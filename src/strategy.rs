use crate::sim::{
    MAX_DISTRIBUTION_PROJECT_CUSTOMERS, MAX_GENERATION_PROJECT_MWH, MAX_PUBLIC_RATE_PREMIUM_CENTS,
    MIN_DISTRIBUTION_PROJECT_CUSTOMERS, MIN_GENERATION_PROJECT_MWH,
    REGIONAL_MANDATE_EXPANSION_TARGET, REGIONAL_MANDATE_LEVERAGE_LIMIT,
    REGIONAL_MANDATE_RELIABILITY_TARGET, REGIONAL_MANDATE_SHARE_TARGET, public_rate_tolerance,
};
use crate::{
    AcquisitionTerms, DISTRIBUTION_PROJECT_CAPACITY, Decision, GENERATION_PROJECT_CAPACITY_MWH,
    Game, InitialVariance, OutcomeKind, QuarterReport, distribution_project_cost,
    generation_project_cost,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Strategy {
    Naive,
    Organic,
    Balanced,
    Regional,
    Mna,
    Raider,
    Glonzo,
}

impl Strategy {
    pub fn all() -> [Strategy; 7] {
        [
            Strategy::Naive,
            Strategy::Organic,
            Strategy::Balanced,
            Strategy::Regional,
            Strategy::Mna,
            Strategy::Raider,
            Strategy::Glonzo,
        ]
    }

    pub fn name(self) -> &'static str {
        match self {
            Strategy::Naive => "naive",
            Strategy::Organic => "organic",
            Strategy::Balanced => "balanced",
            Strategy::Regional => "regional",
            Strategy::Mna => "mna",
            Strategy::Raider => "raider",
            Strategy::Glonzo => "glonzo",
        }
    }

    fn continues_to_regional_mandate(self) -> bool {
        matches!(self, Strategy::Regional)
    }
}

#[derive(Clone, Debug, Default)]
pub struct StrategySummary {
    pub runs: u32,
    pub victories: u32,
    pub board_defeats: u32,
    pub receivership_defeats: u32,
    pub market_access_defeats: u32,
    pub board_defeat_quarters: f64,
    pub receivership_defeat_quarters: f64,
    pub market_access_defeat_quarters: f64,
    pub defeat_quarters: f64,
    pub defeat_quarter_range: Range,
    pub start_share: f64,
    pub start_cash: f64,
    pub start_debt: f64,
    pub start_customers: f64,
    pub start_reliability: f64,
    pub start_rate: f64,
    pub start_headroom: f64,
    pub start_share_range: Range,
    pub start_cash_range: Range,
    pub start_debt_range: Range,
    pub start_customers_range: Range,
    pub start_reliability_range: Range,
    pub start_rate_range: Range,
    pub start_headroom_range: Range,
    pub finish_share: f64,
    pub finish_share_range: Range,
    pub finish_share_histogram: ShareHistogram,
    pub finish_cash: f64,
    pub finish_debt: f64,
    pub finish_customers: f64,
    pub finish_reliability: f64,
    pub peak_acquisition_stress: f64,
    pub peak_acquisition_stress_range: Range,
    pub acquisition_stress_defeats: u32,
    pub acquisition_stress_receiverships: u32,
}

impl StrategySummary {
    pub fn win_rate(&self) -> f64 {
        if self.runs == 0 {
            0.0
        } else {
            self.victories as f64 / self.runs as f64
        }
    }

    pub fn defeats(&self) -> u32 {
        self.board_defeats + self.receivership_defeats + self.market_access_defeats
    }

    pub fn average_finish_share(&self) -> f64 {
        if self.runs == 0 {
            0.0
        } else {
            self.finish_share / self.runs as f64
        }
    }

    pub fn average_peak_acquisition_stress(&self) -> f64 {
        if self.runs == 0 {
            0.0
        } else {
            self.peak_acquisition_stress / self.runs as f64
        }
    }

    fn record_defeat(&mut self, headline: &str, quarter: u32) {
        let quarter = quarter as f64;
        self.defeat_quarters += quarter;
        self.defeat_quarter_range.observe(quarter);
        if headline.contains("Market Access") {
            self.market_access_defeats += 1;
            self.market_access_defeat_quarters += quarter;
        } else if headline.contains("Receivership") {
            self.receivership_defeats += 1;
            self.receivership_defeat_quarters += quarter;
        } else {
            self.board_defeats += 1;
            self.board_defeat_quarters += quarter;
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct ShareHistogram {
    bins: [u32; 7],
}

impl ShareHistogram {
    fn observe(&mut self, share: f64) {
        let pct = share * 100.0;
        let index = if pct < 35.0 {
            0
        } else if pct < 40.0 {
            1
        } else if pct < 45.0 {
            2
        } else if pct < 50.0 {
            3
        } else if pct < 55.0 {
            4
        } else if pct < 60.0 {
            5
        } else {
            6
        };
        self.bins[index] += 1;
    }

    pub fn render(&self) -> String {
        let labels = [
            "<35%", "35-40%", "40-45%", "45-50%", "50-55%", "55-60%", "60%+",
        ];
        let mut parts = Vec::with_capacity(labels.len());
        for (label, count) in labels.iter().zip(self.bins.iter()) {
            parts.push(format!("{label}: {count}"));
        }
        parts.join(" | ")
    }
}

#[derive(Clone, Debug)]
pub struct Range {
    pub min: f64,
    pub max: f64,
}

impl Range {
    fn observe(&mut self, value: f64) {
        self.min = self.min.min(value);
        self.max = self.max.max(value);
    }
}

impl Default for Range {
    fn default() -> Self {
        Self {
            min: f64::INFINITY,
            max: f64::NEG_INFINITY,
        }
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
    const MATERIAL_ACQUISITION_STRESS: f64 = 0.62;
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
        Strategy::Mna => mna_policy(game),
        Strategy::Raider => raider_policy(game),
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

fn naive_policy(game: &mut Game) {
    if game.player.cash < 8_000.0 && game.player.debt_to_assets() < 0.60 {
        borrow_up_to(game, 15_000.0, 0.60);
    }

    if game.player.cash < 6_000.0 {
        let _ = game.apply_decision(Decision::IssueStock { amount: 12_000.0 });
    }

    if game.player.capacity_headroom(&game.market) < 80.0 {
        let cost = distribution_project_cost(DISTRIBUTION_PROJECT_CAPACITY);
        if game.player.cash > cost + 5_000.0 {
            let _ = game.apply_decision(Decision::BuildDistribution {
                customer_capacity: DISTRIBUTION_PROJECT_CAPACITY,
            });
        }
    }

    if game.player.firm_generation_reserve_mwh(&game.market) < 45.0 {
        let cost = generation_project_cost(GENERATION_PROJECT_CAPACITY_MWH);
        if game.player.cash > cost + 6_000.0 {
            let _ = game.apply_decision(Decision::BuildGeneration {
                capacity_mwh: GENERATION_PROJECT_CAPACITY_MWH,
            });
        }
    }

    if game.quarter.is_multiple_of(3) && game.player.cash > 8_000.0 {
        let _ = game.apply_decision(Decision::Marketing { spend: 3_500.0 });
    }

    if game.player.reliability < 0.80 && game.player.cash > 8_500.0 {
        let _ = game.apply_decision(Decision::Maintenance { spend: 5_000.0 });
    }

    if game.market_share() < 0.30 && game.player.rate_cents > 9.8 {
        let _ = game.apply_decision(Decision::AdjustRate { delta_cents: -0.25 });
    }
}

fn organic_policy(game: &mut Game) {
    run_organic_program(
        game,
        OrganicPlan {
            target_rate_cents: 7.95,
            marketing_spend: 11_250.0,
            marketing_interval: 1,
            reliability_floor: 0.84,
            headroom_target: 400.0,
            firm_reserve_target_mwh: 135.0,
            distribution_project_customers: 760.0,
            generation_project_mwh: 530.0,
            cash_reserve: 10_000.0,
            max_debt_to_assets: 0.78,
        },
    );
}

fn balanced_policy(game: &mut Game) {
    run_organic_program(
        game,
        OrganicPlan {
            target_rate_cents: 7.90,
            marketing_spend: 12_500.0,
            marketing_interval: 1,
            reliability_floor: 0.84,
            headroom_target: 420.0,
            firm_reserve_target_mwh: 145.0,
            distribution_project_customers: 790.0,
            generation_project_mwh: 550.0,
            cash_reserve: 10_000.0,
            max_debt_to_assets: 0.80,
        },
    );
    monetize_established_share(game, 0.42);

    let acquisition_share_cap = if game.quarter < 8 { 0.36 } else { 0.48 };
    if game.quarter >= 3 && game.market_share() < acquisition_share_cap {
        let max_deal_share = if game.quarter < 8 { 0.20 } else { 0.28 };
        let max_execution_risk = if game.quarter < 8 { 0.22 } else { 0.28 };
        let _ = try_acquire_opportunistically(
            game,
            14_000.0,
            0.80,
            0.90,
            max_deal_share,
            max_execution_risk,
        );
    }
}

fn regional_policy(game: &mut Game) {
    let post_review = game.review_completed;
    let plan = if post_review {
        OrganicPlan {
            target_rate_cents: 7.80,
            marketing_spend: 16_000.0,
            marketing_interval: 1,
            reliability_floor: REGIONAL_MANDATE_RELIABILITY_TARGET + 0.04,
            headroom_target: 720.0,
            firm_reserve_target_mwh: 250.0,
            distribution_project_customers: 1_100.0,
            generation_project_mwh: 780.0,
            cash_reserve: 20_000.0,
            max_debt_to_assets: 0.84,
        }
    } else {
        OrganicPlan {
            target_rate_cents: 8.00,
            marketing_spend: 12_250.0,
            marketing_interval: 1,
            reliability_floor: 0.84,
            headroom_target: 450.0,
            firm_reserve_target_mwh: 155.0,
            distribution_project_customers: 820.0,
            generation_project_mwh: 575.0,
            cash_reserve: 12_000.0,
            max_debt_to_assets: 0.78,
        }
    };

    run_organic_program(game, plan);
    monetize_established_share(game, 0.46);

    if game.quarter >= 8 || post_review {
        try_enter_adjacent_market(game, 24_000.0, 0.86);
    }

    if game.regional_integration > 0.10 && game.player.cash > 16_000.0 {
        let _ = game.apply_decision(Decision::Marketing { spend: 8_000.0 });
    }

    if game.player.debt_to_assets() > REGIONAL_MANDATE_LEVERAGE_LIMIT - 0.03 {
        repay_excess_cash(game, 24_000.0);
    }

    if post_review && game.market_share() < REGIONAL_MANDATE_SHARE_TARGET - 0.005 {
        let _ = try_acquire_opportunistically(game, 18_000.0, 0.84, 0.96, 0.24, 0.26);
    }
}

fn mna_policy(game: &mut Game) {
    if let Some((index, price)) = cheapest_competitor(game) {
        let reserve = if game.quarter < 4 { 5_000.0 } else { 10_000.0 };
        let diligence_cost = if game.has_diligence(index) {
            0.0
        } else {
            game.diligence_cost(index).unwrap_or(0.0)
        };
        if game.player.cash < price + reserve + diligence_cost
            && game.player.debt_to_assets() < 0.66
        {
            let _ = game.apply_decision(Decision::Borrow {
                amount: (price + reserve + diligence_cost - game.player.cash)
                    .clamp(8_000.0, 42_000.0),
            });
        }

        if game.player.cash < price + reserve + diligence_cost
            && game.player.debt_to_assets() >= 0.60
        {
            let _ = game.apply_decision(Decision::IssueStock {
                amount: (price + reserve + diligence_cost - game.player.cash)
                    .clamp(12_000.0, 52_000.0),
            });
        }

        if !game.has_diligence(index) && game.player.cash > diligence_cost + reserve * 0.25 {
            let _ = game.apply_decision(Decision::Diligence {
                competitor_index: index,
            });
        }

        if game.player.cash > price + reserve && game.player.debt_to_assets() < 0.98 {
            let _ = game.apply_decision(Decision::Acquire {
                competitor_index: index,
            });
        }
    }

    if game.player.cash < 12_000.0 && game.player.debt_to_assets() < 0.72 {
        let _ = game.apply_decision(Decision::Borrow { amount: 20_000.0 });
    }

    if game.player.cash < 10_000.0 {
        let _ = game.apply_decision(Decision::IssueStock { amount: 24_000.0 });
    }

    let generation_headroom = game.player.generation_capacity_mwh
        - game.player.customers * game.market.avg_mwh_per_customer;
    if generation_headroom < 70.0 && game.player.cash > 24_000.0 {
        let _ = game.apply_decision(Decision::BuildGeneration {
            capacity_mwh: GENERATION_PROJECT_CAPACITY_MWH,
        });
    }

    if game.player.capacity_headroom(&game.market) < 95.0 && game.player.cash > 14_000.0 {
        let _ = game.apply_decision(Decision::BuildDistribution {
            customer_capacity: DISTRIBUTION_PROJECT_CAPACITY,
        });
    }

    if game.quarter.is_multiple_of(3) && game.player.cash > 8_000.0 {
        let _ = game.apply_decision(Decision::Marketing { spend: 5_500.0 });
    }

    if game.market_share() < 0.34 && game.player.rate_cents > 9.4 {
        let _ = game.apply_decision(Decision::AdjustRate { delta_cents: -0.5 });
    }

    if game.player.reliability < 0.80 && game.player.cash > 7_500.0 {
        let spend = (5_500.0 + game.player.asset_base * 0.018).clamp(7_000.0, 20_000.0);
        let _ = game.apply_decision(Decision::Maintenance { spend });
    }

    if let Some((index, price)) = cheapest_competitor(game) {
        let can_absorb_debt = game.player.debt_to_assets() < 0.82;
        if !game.has_diligence(index)
            && let Some(cost) = game.diligence_cost(index)
            && game.player.cash > cost + 5_000.0
        {
            let _ = game.apply_decision(Decision::Diligence {
                competitor_index: index,
            });
        }
        if game.player.cash > price + 10_000.0 && can_absorb_debt {
            let _ = game.apply_decision(Decision::Acquire {
                competitor_index: index,
            });
        }
    }
}

fn raider_policy(game: &mut Game) {
    if game.player.cash < 18_000.0 && game.borrowing_room() > 1_000.0 {
        let _ = game.apply_decision(Decision::Borrow {
            amount: game.borrowing_room().min(60_000.0),
        });
    }

    if game.player.cash < 14_000.0 {
        let _ = game.apply_decision(Decision::IssueStock { amount: 36_000.0 });
    }

    if game.competitors.len() > 1
        && game.acquisition_cooldown == 0
        && let Some((index, terms)) = riskiest_raider_target(game)
    {
        let reserve = if game.quarter < 8 { 2_500.0 } else { 5_000.0 };
        raider_finance_to_cash(game, terms.price + reserve);

        if game.player.cash > terms.price + reserve {
            let _ = game.apply_decision(Decision::Acquire {
                competitor_index: index,
            });
        }
    }

    let generation_headroom = game.player.generation_capacity_mwh
        - game.player.customers * game.market.avg_mwh_per_customer;
    if generation_headroom < 35.0 && game.player.cash > 20_000.0 {
        let _ = game.apply_decision(Decision::BuildGeneration {
            capacity_mwh: GENERATION_PROJECT_CAPACITY_MWH,
        });
    }

    if game.player.capacity_headroom(&game.market) < 70.0 && game.player.cash > 12_000.0 {
        let _ = game.apply_decision(Decision::BuildDistribution {
            customer_capacity: DISTRIBUTION_PROJECT_CAPACITY,
        });
    }

    if game.quarter.is_multiple_of(2) && game.player.cash > 6_000.0 {
        let _ = game.apply_decision(Decision::Marketing { spend: 4_500.0 });
    }

    if game.player.reliability < 0.72 && game.player.cash > 5_000.0 {
        let _ = game.apply_decision(Decision::Maintenance { spend: 4_500.0 });
    }

    if game.market_share() < 0.44 && game.player.rate_cents > 8.6 {
        let _ = game.apply_decision(Decision::AdjustRate { delta_cents: -0.45 });
    }
}

fn glonzo_policy(game: &mut Game, rng: &mut GlonzoRng) {
    let actions = rng.u32_inclusive(2, 7);
    for _ in 0..actions {
        match rng.u32_inclusive(0, 99) {
            0..=10 => glonzo_borrow(game, rng),
            11..=20 => glonzo_repay(game, rng),
            21..=30 => glonzo_issue(game, rng),
            31..=39 => glonzo_buyback(game, rng),
            40..=51 => glonzo_rate(game, rng),
            52..=63 => glonzo_marketing(game, rng),
            64..=74 => glonzo_maintenance(game, rng),
            75..=83 => glonzo_build_distribution(game, rng),
            84..=91 => glonzo_build_generation(game, rng),
            92..=97 => glonzo_deal(game, rng),
            _ => glonzo_expand(game, rng),
        }
    }
}

fn glonzo_borrow(game: &mut Game, rng: &mut GlonzoRng) {
    let room = game.borrowing_room();
    if room < 1_000.0 {
        return;
    }
    let amount = rng.money_between(1_000.0, room.min(85_000.0));
    let _ = game.apply_decision(Decision::Borrow { amount });
}

fn glonzo_repay(game: &mut Game, rng: &mut GlonzoRng) {
    let available = glonzo_available_cash(game, rng.range(0.0, 12_000.0));
    let amount = game.player.debt.min(available);
    if amount < 1_000.0 {
        return;
    }
    let payment = if rng.chance(0.12) {
        amount
    } else {
        rng.money_between(1_000.0, amount)
    };
    let _ = game.apply_decision(Decision::RepayDebt { amount: payment });
}

fn glonzo_issue(game: &mut Game, rng: &mut GlonzoRng) {
    let financing_base = game
        .player
        .market_cap()
        .max(game.player.asset_base * 0.45)
        .max(8_000.0);
    let pressure = rng.range(0.12, 1.10);
    let amount = rng.money_between(1_000.0, (financing_base * pressure).max(1_000.0));
    let _ = game.apply_decision(Decision::IssueStock { amount });
}

fn glonzo_buyback(game: &mut Game, rng: &mut GlonzoRng) {
    let available = glonzo_available_cash(game, rng.range(0.0, 18_000.0));
    if available < 1_000.0 {
        return;
    }
    let cap_pressure = (game.player.market_cap() * rng.range(0.04, 0.35)).max(1_000.0);
    let amount = rng.money_between(1_000.0, available.min(cap_pressure));
    let _ = game.apply_decision(Decision::BuyBackStock { amount });
}

fn glonzo_rate(game: &mut Game, rng: &mut GlonzoRng) {
    let tolerance = public_rate_tolerance(&game.market);
    let high = (tolerance + MAX_PUBLIC_RATE_PREMIUM_CENTS).clamp(1.0, 24.0);
    let target = if rng.chance(0.18) {
        rng.range(0.25, 24.0)
    } else {
        rng.range(0.25, high)
    };
    let delta = target - game.player.rate_cents;
    if delta.abs() >= 0.05 {
        let _ = game.apply_decision(Decision::AdjustRate { delta_cents: delta });
    }
}

fn glonzo_marketing(game: &mut Game, rng: &mut GlonzoRng) {
    let available = glonzo_available_cash(game, rng.range(0.0, 10_000.0));
    if available < 1_000.0 {
        return;
    }
    let spend = rng.money_between(1_000.0, available.min(55_000.0));
    let _ = game.apply_decision(Decision::Marketing { spend });
}

fn glonzo_maintenance(game: &mut Game, rng: &mut GlonzoRng) {
    if game.player.reliability >= 0.979 {
        return;
    }
    let available = glonzo_available_cash(game, rng.range(0.0, 12_000.0));
    if available < 1_000.0 {
        return;
    }
    let spend = rng.money_between(1_000.0, available.min(45_000.0));
    let _ = game.apply_decision(Decision::Maintenance { spend });
}

fn glonzo_build_distribution(game: &mut Game, rng: &mut GlonzoRng) {
    let size = rng.range(
        MIN_DISTRIBUTION_PROJECT_CUSTOMERS,
        MAX_DISTRIBUTION_PROJECT_CUSTOMERS,
    );
    let cost = distribution_project_cost(size);
    glonzo_finance_toward(game, cost + rng.range(0.0, 15_000.0), rng);
    if game.player.cash >= cost {
        let _ = game.apply_decision(Decision::BuildDistribution {
            customer_capacity: size,
        });
    }
}

fn glonzo_build_generation(game: &mut Game, rng: &mut GlonzoRng) {
    let size = rng.range(MIN_GENERATION_PROJECT_MWH, MAX_GENERATION_PROJECT_MWH);
    let cost = generation_project_cost(size);
    glonzo_finance_toward(game, cost + rng.range(0.0, 18_000.0), rng);
    if game.player.cash >= cost {
        let _ = game.apply_decision(Decision::BuildGeneration { capacity_mwh: size });
    }
}

fn glonzo_deal(game: &mut Game, rng: &mut GlonzoRng) {
    if game.competitors.len() <= 1 {
        return;
    }
    let index = rng.usize_below(game.competitors.len());
    if !game.has_diligence(index)
        && let Some(cost) = game.diligence_cost(index)
    {
        glonzo_finance_toward(game, cost + rng.range(0.0, 10_000.0), rng);
        if game.player.cash >= cost {
            let _ = game.apply_decision(Decision::Diligence {
                competitor_index: index,
            });
        }
    }
    if !rng.chance(0.58) {
        return;
    }
    if let Some(terms) = estimated_acquisition_terms(game, index) {
        glonzo_finance_toward(game, terms.price + rng.range(0.0, 18_000.0), rng);
        if game.player.cash >= terms.price {
            let _ = game.apply_decision(Decision::Acquire {
                competitor_index: index,
            });
        }
    }
}

fn glonzo_expand(game: &mut Game, rng: &mut GlonzoRng) {
    if game.adjacent_expansion_blocker().is_some() {
        return;
    }
    let cost = game.adjacent_expansion_cost();
    glonzo_finance_toward(game, cost + rng.range(0.0, 25_000.0), rng);
    if game.player.cash >= cost {
        let _ = game.apply_decision(Decision::EnterAdjacentMarket);
    }
}

fn glonzo_finance_toward(game: &mut Game, target_cash: f64, rng: &mut GlonzoRng) {
    if game.player.cash >= target_cash {
        return;
    }
    let need = target_cash - game.player.cash;
    if rng.chance(0.68) {
        let room = game.borrowing_room();
        if room >= 1_000.0 {
            let amount = need.min(room).max(1_000.0);
            let _ = game.apply_decision(Decision::Borrow { amount });
        }
    }
    if game.player.cash < target_cash && rng.chance(0.74) {
        let amount = ((target_cash - game.player.cash) * rng.range(0.8, 1.6)).max(1_000.0);
        let _ = game.apply_decision(Decision::IssueStock { amount });
    }
}

fn glonzo_available_cash(game: &Game, reserve: f64) -> f64 {
    (game.player.cash - reserve.max(0.0)).max(0.0)
}

#[derive(Clone, Copy)]
struct OrganicPlan {
    target_rate_cents: f64,
    marketing_spend: f64,
    marketing_interval: u32,
    reliability_floor: f64,
    headroom_target: f64,
    firm_reserve_target_mwh: f64,
    distribution_project_customers: f64,
    generation_project_mwh: f64,
    cash_reserve: f64,
    max_debt_to_assets: f64,
}

fn run_organic_program(game: &mut Game, plan: OrganicPlan) {
    cut_rate_toward(game, plan.target_rate_cents);
    build_distribution_to_headroom(game, plan);
    build_generation_to_reserve(game, plan);

    if plan.marketing_interval > 0 && game.quarter.is_multiple_of(plan.marketing_interval) {
        let marketing_buffer = plan.cash_reserve * 0.75;
        finance_to_cash(
            game,
            plan.marketing_spend + marketing_buffer,
            plan.max_debt_to_assets,
        );
        if game.player.cash > plan.marketing_spend + project_cash_buffer(plan) {
            let _ = game.apply_decision(Decision::Marketing {
                spend: plan.marketing_spend,
            });
        }
    }

    maintain_reliability(game, plan);
    if game.player.cash < plan.cash_reserve {
        finance_to_cash(game, plan.cash_reserve + 10_000.0, plan.max_debt_to_assets);
    }
}

fn build_generation_to_reserve(game: &mut Game, plan: OrganicPlan) {
    let reserve = game.player.firm_generation_reserve_mwh(&game.market);
    if reserve >= plan.firm_reserve_target_mwh {
        return;
    }

    let project_size = (plan.generation_project_mwh
        + (plan.firm_reserve_target_mwh - reserve).max(0.0) * 0.35)
        .clamp(250.0, 900.0);
    let cost = generation_project_cost(project_size);
    finance_to_cash(game, cost + plan.cash_reserve, plan.max_debt_to_assets);
    if game.player.cash > cost + project_cash_buffer(plan) {
        let _ = game.apply_decision(Decision::BuildGeneration {
            capacity_mwh: project_size,
        });
    }
}

fn build_distribution_to_headroom(game: &mut Game, plan: OrganicPlan) {
    let headroom = game.player.capacity_headroom(&game.market);
    if headroom >= plan.headroom_target {
        return;
    }

    let project_size = (plan.distribution_project_customers
        + (plan.headroom_target - headroom).max(0.0) * 0.45)
        .clamp(300.0, 1_200.0);
    let cost = distribution_project_cost(project_size);
    finance_to_cash(game, cost + plan.cash_reserve, plan.max_debt_to_assets);
    if game.player.cash > cost + project_cash_buffer(plan) {
        let _ = game.apply_decision(Decision::BuildDistribution {
            customer_capacity: project_size,
        });
    }
}

fn maintain_reliability(game: &mut Game, plan: OrganicPlan) {
    if game.player.reliability >= plan.reliability_floor {
        return;
    }

    let spend = (8_000.0 + game.player.asset_base * 0.012).clamp(8_000.0, 20_000.0);
    finance_to_cash(
        game,
        spend + plan.cash_reserve * 0.5,
        plan.max_debt_to_assets,
    );
    if game.player.cash > spend + plan.cash_reserve * 0.375 {
        let _ = game.apply_decision(Decision::Maintenance { spend });
    }
}

fn cut_rate_toward(game: &mut Game, target_rate_cents: f64) {
    while game.player.rate_cents > target_rate_cents + 0.05 {
        let cut = (game.player.rate_cents - target_rate_cents).min(0.5);
        let _ = game.apply_decision(Decision::AdjustRate { delta_cents: -cut });
    }
}

fn monetize_established_share(game: &mut Game, share_floor: f64) {
    if game.quarter < game.campaign_quarters.saturating_sub(5) || game.market_share() < share_floor
    {
        return;
    }

    let break_even_target = game.player_break_even_rate_cents() * 1.04;
    let public_ceiling = public_rate_tolerance(&game.market) + MAX_PUBLIC_RATE_PREMIUM_CENTS - 0.10;
    let target = break_even_target
        .min(public_ceiling)
        .max(game.player.rate_cents);
    if target <= game.player.rate_cents + 0.05 {
        return;
    }

    let delta = (target - game.player.rate_cents).min(0.50);
    let _ = game.apply_decision(Decision::AdjustRate { delta_cents: delta });
}

fn try_enter_adjacent_market(game: &mut Game, reserve: f64, max_debt_to_assets: f64) -> bool {
    if game.adjacent_expansions >= REGIONAL_MANDATE_EXPANSION_TARGET {
        let can_absorb_optional_expansion = game.market_share()
            >= REGIONAL_MANDATE_SHARE_TARGET + 0.02
            && game.player.debt_to_assets() <= REGIONAL_MANDATE_LEVERAGE_LIMIT - 0.12;
        if !can_absorb_optional_expansion {
            return false;
        }
    }
    if game.adjacent_expansion_blocker().is_some() {
        return false;
    }

    let cost = game.adjacent_expansion_cost();
    finance_to_cash(game, cost + reserve, max_debt_to_assets);
    if game.player.cash <= cost + reserve * 0.50 {
        return false;
    }

    game.apply_decision(Decision::EnterAdjacentMarket).is_ok()
}

fn repay_excess_cash(game: &mut Game, reserve: f64) {
    let excess_cash = (game.player.cash - reserve).max(0.0);
    let amount = excess_cash.min(game.player.debt);
    if amount >= 1_000.0 {
        let _ = game.apply_decision(Decision::RepayDebt { amount });
    }
}

fn project_cash_buffer(plan: OrganicPlan) -> f64 {
    (plan.cash_reserve * 0.625).max(3_000.0)
}

fn try_acquire_opportunistically(
    game: &mut Game,
    reserve: f64,
    max_debt_to_assets: f64,
    max_price_to_assets: f64,
    max_deal_share: f64,
    max_execution_risk: f64,
) -> bool {
    if game.acquisition_cooldown > 0 || game.competitors.len() <= 1 {
        return false;
    }

    let Some((index, terms)) = best_opportunistic_competitor(
        game,
        max_price_to_assets,
        max_deal_share,
        max_execution_risk,
    ) else {
        return false;
    };

    let diligence_cost = if game.has_diligence(index) {
        0.0
    } else {
        game.diligence_cost(index).unwrap_or(0.0)
    };
    finance_to_cash(
        game,
        terms.price + reserve + diligence_cost,
        max_debt_to_assets,
    );
    if !game.has_diligence(index) {
        if game.player.cash <= diligence_cost + reserve * 0.25 {
            return false;
        }
        if game
            .apply_decision(Decision::Diligence {
                competitor_index: index,
            })
            .is_err()
        {
            return false;
        }
    }
    if game.player.cash <= terms.price + reserve
        || game.player.debt_to_assets() > max_debt_to_assets + 0.08
    {
        return false;
    }

    game.apply_decision(Decision::Acquire {
        competitor_index: index,
    })
    .is_ok()
}

fn best_opportunistic_competitor(
    game: &Game,
    max_price_to_assets: f64,
    max_deal_share: f64,
    max_execution_risk: f64,
) -> Option<(usize, AcquisitionTerms)> {
    (0..game.competitors.len())
        .filter_map(|index| {
            let competitor = &game.competitors[index];
            let terms = estimated_acquisition_terms(game, index)?;
            if terms.price > game.player.asset_base * max_price_to_assets {
                return None;
            }
            let deal_share = terms.acquired_customers
                / (game.player.customers + terms.acquired_customers).max(1.0);
            if deal_share > max_deal_share || terms.post_debt_to_assets > 0.82 {
                return None;
            }
            if competitor.reliability < 0.76 || competitor.reputation < 46.0 {
                return None;
            }
            let execution_risk = balanced_execution_risk(game, competitor, &terms);
            if execution_risk > max_execution_risk {
                return None;
            }
            let strategic_fit = terms.acquired_customers / terms.price.max(1.0);
            let service_fit = (competitor.reliability - 0.78).max(0.0) * 0.20
                + (competitor.reputation - 52.0).max(0.0) / 600.0;
            let score = strategic_fit + service_fit - execution_risk * 0.004;
            Some((index, terms, score))
        })
        .max_by(|left, right| left.2.total_cmp(&right.2))
        .map(|(index, terms, _)| (index, terms))
}

fn balanced_execution_risk(
    game: &Game,
    competitor: &crate::Utility,
    terms: &AcquisitionTerms,
) -> f64 {
    let deal_share =
        terms.acquired_customers / (game.player.customers + terms.acquired_customers).max(1.0);
    let target_service_risk = (0.84 - competitor.reliability).max(0.0) * 0.70;
    let target_reputation_risk = (58.0 - competitor.reputation).max(0.0) / 320.0;
    let leverage_risk = (terms.post_debt_to_assets - 0.68).max(0.0) * 0.34;
    let serial_rollup_risk = game.integration_strain.max(0.0) * 0.14;
    let dominance_risk = (terms.post_market_share - 0.42).max(0.0) * 0.12;

    (0.035
        + deal_share * 0.29
        + target_service_risk
        + target_reputation_risk
        + leverage_risk
        + serial_rollup_risk
        + dominance_risk)
        .clamp(0.06, 0.46)
}

fn finance_to_cash(game: &mut Game, target_cash: f64, max_debt_to_assets: f64) {
    if game.player.cash >= target_cash {
        return;
    }

    let need = target_cash - game.player.cash;
    borrow_up_to(game, need, max_debt_to_assets);

    if game.player.cash < target_cash {
        let amount = (target_cash - game.player.cash) * 1.15;
        if amount >= 1_000.0 {
            let _ = game.apply_decision(Decision::IssueStock { amount });
        }
    }
}

fn borrow_up_to(game: &mut Game, target_amount: f64, max_debt_to_assets: f64) {
    let leverage_room = (game.player.asset_base * max_debt_to_assets - game.player.debt).max(0.0);
    let amount = target_amount.min(game.borrowing_room()).min(leverage_room);
    if amount >= 1_000.0 {
        let _ = game.apply_decision(Decision::Borrow { amount });
    }
}

fn raider_finance_to_cash(game: &mut Game, target_cash: f64) {
    if game.player.cash >= target_cash {
        return;
    }

    let need = target_cash - game.player.cash;
    let room = game.borrowing_room();
    if room >= 1_000.0 {
        let amount = need.min(room).max(1_000.0);
        let _ = game.apply_decision(Decision::Borrow { amount });
    }
    if game.player.cash < target_cash {
        let amount = ((target_cash - game.player.cash) * 1.20).max(5_000.0);
        let _ = game.apply_decision(Decision::IssueStock { amount });
    }
}

fn riskiest_raider_target(game: &Game) -> Option<(usize, AcquisitionTerms)> {
    (0..game.competitors.len())
        .filter_map(|index| {
            let competitor = &game.competitors[index];
            let terms = estimated_acquisition_terms(game, index)?;
            if terms.price > game.player.asset_base.max(40_000.0) * 2.35 {
                return None;
            }
            if terms.post_debt_to_assets > 1.14 {
                return None;
            }
            let deal_share = terms.acquired_customers
                / (game.player.customers + terms.acquired_customers).max(1.0);
            let service_gap = (0.82 - competitor.reliability).max(0.0);
            let reputation_gap = (56.0 - competitor.reputation).max(0.0) / 100.0;
            let stress = game.acquisition_stress_score(index).unwrap_or(0.0);
            let price_pull = terms.acquired_customers / terms.price.max(1.0);
            let score = deal_share * 2.2 + service_gap + reputation_gap + stress + price_pull;
            Some((index, terms, score))
        })
        .max_by(|left, right| left.2.total_cmp(&right.2))
        .map(|(index, terms, _)| (index, terms))
}

fn cheapest_competitor(game: &Game) -> Option<(usize, f64)> {
    (0..game.competitors.len())
        .filter_map(|index| {
            estimated_acquisition_terms(game, index).map(|terms| (index, terms.price))
        })
        .min_by(|left, right| left.1.total_cmp(&right.1))
}

fn estimated_acquisition_terms(game: &Game, index: usize) -> Option<AcquisitionTerms> {
    if game.has_diligence(index) {
        game.acquisition_terms(index)
    } else {
        game.public_acquisition_estimate(index)
    }
}

struct GlonzoRng {
    state: u64,
}

impl GlonzoRng {
    fn from_seed(game_seed: u64, variance: InitialVariance) -> Self {
        let mut state = 0xB10B_1234_CAFE_D00D_u64;
        state = glonzo_mix(state, game_seed);
        state = glonzo_mix(state, variance.amplitude.to_bits());
        Self { state }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut value = self.state;
        value = (value ^ (value >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        value ^ (value >> 31)
    }

    fn unit(&mut self) -> f64 {
        let value = self.next_u64() >> 11;
        value as f64 / ((1_u64 << 53) as f64)
    }

    fn range(&mut self, min: f64, max: f64) -> f64 {
        if min >= max {
            return min;
        }
        min + (max - min) * self.unit()
    }

    fn u32_inclusive(&mut self, min: u32, max: u32) -> u32 {
        if min >= max {
            return min;
        }
        min + (self.next_u64() % u64::from(max - min + 1)) as u32
    }

    fn usize_below(&mut self, max: usize) -> usize {
        if max <= 1 {
            return 0;
        }
        (self.next_u64() % max as u64) as usize
    }

    fn chance(&mut self, probability: f64) -> bool {
        self.unit() < probability.clamp(0.0, 1.0)
    }

    fn money_between(&mut self, min: f64, max: f64) -> f64 {
        if max <= min {
            return min.max(0.0);
        }
        (self.range(min, max) / 100.0).round() * 100.0
    }
}

fn glonzo_mix(state: u64, value: u64) -> u64 {
    let mut mixed = value.wrapping_add(0x9E37_79B9_7F4A_7C15);
    mixed = (mixed ^ (mixed >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    mixed = (mixed ^ (mixed >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    state ^ (mixed ^ (mixed >> 31))
}
