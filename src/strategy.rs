mod costanza;
mod glonzo;
mod landshark;
mod ma;
mod naive;
mod organic;
mod raider;
mod shared;
mod summary;

use crate::{ACQUISITION_STRESS_STRAINED, Game, InitialVariance, OutcomeKind, QuarterReport};
use costanza::costanza_policy;
use glonzo::{GlonzoRng, glonzo_policy};
use landshark::landshark_policy;
use ma::ma_policy;
use naive::naive_policy;
use organic::{balanced_policy, organic_policy, regional_policy};
use raider::raider_policy;

pub use summary::{Range, ShareHistogram, StrategySummary};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Strategy {
    Naive,
    Organic,
    Balanced,
    Regional,
    Ma,
    Raider,
    Landshark,
    Costanza,
    Glonzo,
}

impl Strategy {
    pub fn all() -> [Strategy; 9] {
        [
            Strategy::Naive,
            Strategy::Organic,
            Strategy::Balanced,
            Strategy::Regional,
            Strategy::Ma,
            Strategy::Raider,
            Strategy::Landshark,
            Strategy::Costanza,
            Strategy::Glonzo,
        ]
    }

    pub fn name(self) -> &'static str {
        match self {
            Strategy::Naive => "naive",
            Strategy::Organic => "organic",
            Strategy::Balanced => "balanced",
            Strategy::Regional => "regional",
            Strategy::Ma => "ma",
            Strategy::Raider => "raider",
            Strategy::Landshark => "landshark",
            Strategy::Costanza => "costanza",
            Strategy::Glonzo => "glonzo",
        }
    }

    fn continues_to_regional_mandate(self) -> bool {
        matches!(self, Strategy::Regional)
    }
}

pub fn run_strategy_batch(
    seeds: u32,
    variance: InitialVariance,
    strategy: Strategy,
) -> StrategySummary {
    run_strategy_batch_with_observer(seeds, variance, strategy, |_, _, _| {})
}

pub fn run_strategy_batch_with_observer<F>(
    seeds: u32,
    variance: InitialVariance,
    strategy: Strategy,
    mut after_quarter: F,
) -> StrategySummary
where
    F: FnMut(u32, &Game, &QuarterReport),
{
    let seeds = seeds.max(1);
    const MATERIAL_ACQUISITION_STRESS: f64 = ACQUISITION_STRESS_STRAINED;
    let mut summary = StrategySummary {
        runs: seeds,
        ..StrategySummary::default()
    };

    for seed in 1..=seeds {
        let game_seed = seed as u64 * 1_003;
        let mut game = Game::with_seed_and_initial_variance(game_seed, variance);
        let mut state = StrategyState::new(strategy, game_seed, variance);
        let start_share = game.market_share();
        let start_cash = game.player.cash;
        let start_debt = game.player.debt;
        let start_customers = game.player.customers;
        let start_reliability = game.player.reliability;
        let start_rate = game.player.rate_cents;
        let start_headroom = game.player.capacity_headroom(&game.market);
        summary.start_share += start_share;
        summary.start_cash += start_cash;
        summary.start_debt += start_debt;
        summary.start_customers += start_customers;
        summary.start_reliability += start_reliability;
        summary.start_rate += start_rate;
        summary.start_headroom += start_headroom;
        summary.start_share_range.observe(start_share);
        summary.start_cash_range.observe(start_cash);
        summary.start_debt_range.observe(start_debt);
        summary.start_customers_range.observe(start_customers);
        summary.start_reliability_range.observe(start_reliability);
        summary.start_rate_range.observe(start_rate);
        summary.start_headroom_range.observe(start_headroom);

        let mut run_peak_acquisition_stress = game.acquisition_stress;
        loop {
            if let Some(outcome) = &game.outcome {
                if strategy.continues_to_regional_mandate()
                    && outcome.can_continue
                    && !game.regional_mandate_completed
                {
                    let _ = game.continue_after_review();
                } else {
                    break;
                }
            }

            run_policy(strategy, &mut game, &mut state);
            run_peak_acquisition_stress = run_peak_acquisition_stress.max(game.acquisition_stress);
            let report = game.advance_quarter();
            run_peak_acquisition_stress = run_peak_acquisition_stress.max(game.acquisition_stress);
            after_quarter(seed, &game, &report);
        }

        if matches!(
            game.outcome.as_ref().map(|outcome| &outcome.kind),
            Some(OutcomeKind::Victory)
        ) {
            summary.victories += 1;
        } else if let Some(outcome) = &game.outcome {
            summary.record_defeat(&outcome.headline, game.quarter);
            if run_peak_acquisition_stress >= MATERIAL_ACQUISITION_STRESS {
                summary.acquisition_stress_defeats += 1;
                if outcome.headline.contains("Receivership") {
                    summary.acquisition_stress_receiverships += 1;
                }
            }
        }

        let finish_share = game.market_share();
        summary.peak_acquisition_stress += run_peak_acquisition_stress;
        summary
            .peak_acquisition_stress_range
            .observe(run_peak_acquisition_stress);
        summary.finish_share += finish_share;
        summary.finish_cash += game.player.cash;
        summary.finish_debt += game.player.debt;
        summary.finish_customers += game.player.customers;
        summary.finish_reliability += game.player.reliability;
        summary.finish_share_range.observe(finish_share);
        summary.finish_share_histogram.observe(finish_share);
    }

    summary
}

fn run_policy(strategy: Strategy, game: &mut Game, state: &mut StrategyState) {
    match strategy {
        Strategy::Naive => naive_policy(game),
        Strategy::Organic => organic_policy(game),
        Strategy::Balanced => balanced_policy(game),
        Strategy::Regional => regional_policy(game),
        Strategy::Ma => ma_policy(game),
        Strategy::Raider => raider_policy(game),
        Strategy::Landshark => landshark_policy(game),
        Strategy::Costanza => costanza_policy(game),
        Strategy::Glonzo => {
            if let StrategyState::Glonzo(rng) = state {
                glonzo_policy(game, rng);
            }
        }
    }
}

enum StrategyState {
    None,
    Glonzo(GlonzoRng),
}

impl StrategyState {
    fn new(strategy: Strategy, game_seed: u64, variance: InitialVariance) -> Self {
        match strategy {
            Strategy::Glonzo => Self::Glonzo(GlonzoRng::from_seed(game_seed, variance)),
            _ => Self::None,
        }
    }
}
