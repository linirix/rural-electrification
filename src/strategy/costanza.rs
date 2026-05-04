//! Contrarian poor-instinct strategy.
//!
//! `costanza` identifies the kind of late, under-capitalized, under-maintained
//! play that a weak player might attempt, then does the opposite: finance early,
//! build before constraints bind, maintain first, and avoid panic pricing.

use crate::sim::{
    MAX_PUBLIC_RATE_PREMIUM_CENTS, REVIEW_MIN_RATE_SUPPORT_RATIO, public_rate_tolerance,
};
use crate::{Decision, Game, distribution_project_cost, generation_project_cost};

use super::shared::{borrow_up_to, estimated_acquisition_terms, finance_to_cash};

pub(super) fn costanza_policy(game: &mut Game) {
    finance_before_crisis(game);
    build_before_constraints(game);
    maintain_before_outages(game);
    market_before_share_is_lost(game);
    price_against_break_even_not_panic(game);
    buy_only_when_naive_would_not(game);
    clean_up_balance_sheet(game);
}

fn finance_before_crisis(game: &mut Game) {
    let target_cash = if game.quarter < 6 { 42_000.0 } else { 24_000.0 };
    if game.player.cash < target_cash && game.player.debt_to_assets() < 0.74 {
        borrow_up_to(game, target_cash - game.player.cash, 0.74);
    }
    if game.player.cash < 9_000.0 {
        let _ = game.apply_decision(Decision::IssueStock { amount: 18_000.0 });
    }
}

fn build_before_constraints(game: &mut Game) {
    if game.player.capacity_headroom(&game.market) < 300.0 {
        let size = (620.0 + (300.0 - game.player.capacity_headroom(&game.market)).max(0.0) * 0.30)
            .clamp(360.0, 900.0);
        let cost = distribution_project_cost(size);
        finance_to_cash(game, cost + 10_000.0, 0.76);
        if game.player.cash > cost + 6_000.0 {
            let _ = game.apply_decision(Decision::BuildDistribution {
                customer_capacity: size,
            });
        }
    }

    if game.player.firm_generation_reserve_mwh(&game.market) < 110.0 {
        let size = (460.0
            + (110.0 - game.player.firm_generation_reserve_mwh(&game.market)).max(0.0))
        .clamp(260.0, 700.0);
        let cost = generation_project_cost(size);
        finance_to_cash(game, cost + 10_000.0, 0.76);
        if game.player.cash > cost + 6_000.0 {
            let _ = game.apply_decision(Decision::BuildGeneration { capacity_mwh: size });
        }
    }
}

fn maintain_before_outages(game: &mut Game) {
    if game.player.reliability >= 0.855 {
        return;
    }
    let spend = (8_000.0 + game.player.asset_base * 0.011).clamp(8_000.0, 22_000.0);
    finance_to_cash(game, spend + 9_000.0, 0.76);
    if game.player.cash > spend + 5_000.0 {
        let _ = game.apply_decision(Decision::Maintenance { spend });
    }
}

fn market_before_share_is_lost(game: &mut Game) {
    let spend = if game.market_share() < 0.42 {
        13_500.0
    } else {
        7_500.0
    };
    finance_to_cash(game, spend + 9_000.0, 0.76);
    if game.player.cash > spend + 6_000.0 {
        let _ = game.apply_decision(Decision::Marketing { spend });
    }
}

fn price_against_break_even_not_panic(game: &mut Game) {
    let break_even = game.player_break_even_rate_cents().max(0.25);
    let public_ceiling = public_rate_tolerance(&game.market) + MAX_PUBLIC_RATE_PREMIUM_CENTS - 0.25;
    let growth_price = break_even * (REVIEW_MIN_RATE_SUPPORT_RATIO + 0.08);
    let sustainable_price = break_even * 1.03;
    let target = if game.market_share() < 0.42 {
        growth_price
    } else {
        sustainable_price
    }
    .min(public_ceiling)
    .max(0.25);
    let delta = (target - game.player.rate_cents).clamp(-0.50, 0.50);
    if delta.abs() >= 0.05 {
        let _ = game.apply_decision(Decision::AdjustRate { delta_cents: delta });
    }
}

fn buy_only_when_naive_would_not(game: &mut Game) {
    if game.quarter < 7 || game.acquisition_cooldown > 0 || game.competitors.len() <= 1 {
        return;
    }
    let Some((index, terms, score)) = best_contrarian_target(game) else {
        return;
    };
    if score < 0.0 {
        return;
    }

    let diligence_cost = if game.has_diligence(index) {
        0.0
    } else {
        game.diligence_cost(index).unwrap_or(0.0)
    };
    finance_to_cash(game, terms.net_cash_cost + diligence_cost + 14_000.0, 0.80);
    if !game.has_diligence(index) {
        if game.player.cash <= diligence_cost + 4_000.0 {
            return;
        }
        if game
            .apply_decision(Decision::Diligence {
                competitor_index: index,
            })
            .is_err()
        {
            return;
        }
    }

    let Some(current_terms) = estimated_acquisition_terms(game, index) else {
        return;
    };
    if current_terms.post_debt_to_assets < 0.86
        && game.player.cash > current_terms.net_cash_cost + 8_000.0
    {
        let _ = game.apply_decision(Decision::Acquire {
            competitor_index: index,
        });
    }
}

fn best_contrarian_target(game: &Game) -> Option<(usize, crate::AcquisitionTerms, f64)> {
    (0..game.competitors.len())
        .filter_map(|index| {
            let terms = estimated_acquisition_terms(game, index)?;
            let stress = game.acquisition_stress_score(index).unwrap_or(1.0);
            let deal_share = terms.acquired_customers
                / (game.player.customers + terms.acquired_customers).max(1.0);
            if terms.post_debt_to_assets > 0.86 || deal_share > 0.24 || stress > 0.44 {
                return None;
            }
            let score = terms.acquired_customers / terms.net_cash_cost.max(1.0) * 85.0
                + (0.49 - game.market_share()).max(0.0)
                - stress;
            Some((index, terms, score))
        })
        .max_by(|left, right| left.2.total_cmp(&right.2))
}

fn clean_up_balance_sheet(game: &mut Game) {
    if game.player.debt_to_assets() < 0.68 || game.player.cash < 40_000.0 {
        return;
    }
    let amount = (game.player.cash - 24_000.0).min(game.player.debt);
    if amount >= 2_000.0 {
        let _ = game.apply_decision(Decision::RepayDebt { amount });
    }
}
