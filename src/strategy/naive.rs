//! Passive baseline strategy.
//!
//! `naive` represents a cautious first-time player who keeps the lights on,
//! reacts late to capacity problems, and uses only light marketing. It should
//! usually lose, but it should lose cleanly rather than by crashing the sim.

use crate::{
    DISTRIBUTION_PROJECT_CAPACITY, Decision, GENERATION_PROJECT_CAPACITY_MWH, Game,
    distribution_project_cost, generation_project_cost,
};

use super::shared::borrow_up_to;

pub(super) fn naive_policy(game: &mut Game) {
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
