//! Acquisition-heavy strategy.
//!
//! `ma` tests whether a deal-first player can still win after integration
//! strain, financing pressure, diligence uncertainty, and acquisition stress.
//! It should have a higher upside tail than organic play and a worse downside.

use crate::{
    ACQUISITION_STRESS_NOTICE, DISTRIBUTION_PROJECT_CAPACITY, Decision,
    GENERATION_PROJECT_CAPACITY_MWH, Game,
};

use super::shared::cheapest_competitor;

pub(super) fn ma_policy(game: &mut Game) {
    if let Some((index, price)) = cheapest_competitor(game) {
        let reserve = if game.quarter < 4 { 5_000.0 } else { 10_000.0 };
        let should_skip_diligence = should_skip_diligence_for_bolt_on(game, index);
        let diligence_cost = if game.has_diligence(index) || should_skip_diligence {
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

        if !should_skip_diligence
            && !game.has_diligence(index)
            && game.player.cash > diligence_cost + reserve * 0.25
        {
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
        let should_skip_diligence = should_skip_diligence_for_bolt_on(game, index);
        if !should_skip_diligence
            && !game.has_diligence(index)
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

fn should_skip_diligence_for_bolt_on(game: &Game, competitor_index: usize) -> bool {
    let Some(target) = game.competitors.get(competitor_index) else {
        return false;
    };
    let small_target = target.customers <= game.player.customers.max(1.0) * 0.18;
    let low_stress = game
        .acquisition_stress_score(competitor_index)
        .map(|score| score < ACQUISITION_STRESS_NOTICE)
        .unwrap_or(false);

    small_target && low_stress
}
