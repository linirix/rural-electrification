//! Reckless roll-up stress strategy.
//!
//! `raider` intentionally pushes visible acquisition mechanics hard enough to
//! exercise stress, market-access, hostile-takeover, and high-share edge cases.
//! It is a probe for failure-mode coverage, not a model of good play.

use crate::{
    AcquisitionTerms, DISTRIBUTION_PROJECT_CAPACITY, Decision, GENERATION_PROJECT_CAPACITY_MWH,
    Game,
};

use super::shared::estimated_acquisition_terms;

pub(super) fn raider_policy(game: &mut Game) {
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
