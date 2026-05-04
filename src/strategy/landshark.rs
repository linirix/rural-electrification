//! Visible-mechanics optimizer.
//!
//! `landshark` is the adversarial test player that uses only player-visible
//! state, but squeezes those signals aggressively: rates, financing, deal math,
//! and equity actions are selected for expected value within the public model.

use crate::sim::{
    MAX_PUBLIC_RATE_PREMIUM_CENTS, REVIEW_MIN_RATE_SUPPORT_RATIO, public_rate_tolerance,
};
use crate::{Decision, Game, Utility};

use super::organic::balanced_policy;
use super::shared::{estimated_acquisition_terms, finance_to_cash};

pub(super) fn landshark_policy(game: &mut Game) {
    balanced_policy(game);
    if game.quarter >= game.campaign_quarters.saturating_sub(6) {
        optimize_visible_rate(game);
    }
    pursue_best_visible_deal(game);
    arbitrage_visible_equity(game);
    repay_idle_debt(game);
}

fn optimize_visible_rate(game: &mut Game) {
    let break_even = game.player_break_even_rate_cents().max(0.25);
    let public_ceiling = public_rate_tolerance(&game.market) + MAX_PUBLIC_RATE_PREMIUM_CENTS - 0.05;
    let sustainability_floor = if game.quarter >= game.campaign_quarters.saturating_sub(6) {
        break_even * 1.03
    } else if game.market_share() >= 0.50 {
        break_even * 0.96
    } else {
        break_even * 0.82
    };
    let target = visible_rate_candidates(game)
        .into_iter()
        .filter(|rate| rate.is_finite() && *rate > 0.0)
        .max_by(|left, right| {
            visible_rate_score(game, *left).total_cmp(&visible_rate_score(game, *right))
        })
        .unwrap_or(game.player.rate_cents)
        .max(sustainability_floor)
        .min(public_ceiling.max(0.25));
    adjust_rate_toward(game, target);
}

fn visible_rate_candidates(game: &Game) -> Vec<f64> {
    let break_even = game.player_break_even_rate_cents().max(0.25);
    let public_ceiling = public_rate_tolerance(&game.market) + MAX_PUBLIC_RATE_PREMIUM_CENTS - 0.05;
    let rival_rate = visible_weighted_rival_rate(game).unwrap_or(game.market.standard_rate_cents);
    vec![
        game.player.rate_cents,
        break_even * REVIEW_MIN_RATE_SUPPORT_RATIO + 0.05,
        break_even * 0.78,
        break_even * 0.90,
        break_even * 1.02,
        rival_rate - 0.75,
        rival_rate - 0.25,
        rival_rate + 0.25,
        public_rate_tolerance(&game.market) - 0.50,
        public_ceiling,
    ]
}

fn visible_rate_score(game: &Game, rate: f64) -> f64 {
    let break_even = game.player_break_even_rate_cents().max(0.25);
    let rival_rate = visible_weighted_rival_rate(game).unwrap_or(game.market.standard_rate_cents);
    let tolerance = public_rate_tolerance(&game.market);
    let support_ratio = rate / break_even;
    let margin = (support_ratio - REVIEW_MIN_RATE_SUPPORT_RATIO).clamp(-0.50, 0.75);
    let price_edge = (rival_rate - rate).clamp(-3.0, 3.0) * 0.22;
    let public_penalty = (rate - tolerance).max(0.0).powf(1.2) * 0.35;
    let sustainability_penalty = (REVIEW_MIN_RATE_SUPPORT_RATIO - support_ratio).max(0.0) * 3.0;
    let share_need = (0.47 - game.market_share()).max(0.0) * 1.35;

    margin * 1.05 + price_edge * (1.0 + share_need) - public_penalty - sustainability_penalty
}

fn visible_weighted_rival_rate(game: &Game) -> Option<f64> {
    let total = game
        .competitors
        .iter()
        .map(|rival| rival.customers)
        .sum::<f64>();
    if total <= 0.0 {
        return None;
    }
    Some(
        game.competitors
            .iter()
            .map(|rival| rival.rate_cents * rival.customers)
            .sum::<f64>()
            / total,
    )
}

fn adjust_rate_toward(game: &mut Game, target: f64) {
    let delta = (target - game.player.rate_cents).clamp(-0.75, 0.75);
    if delta.abs() >= 0.05 {
        let _ = game.apply_decision(Decision::AdjustRate { delta_cents: delta });
    }
}

fn pursue_best_visible_deal(game: &mut Game) {
    if game.acquisition_cooldown > 0 || game.competitors.len() <= 1 {
        return;
    }
    if game.market_share() >= 0.46 {
        return;
    }
    let Some((index, score)) = best_visible_deal(game) else {
        return;
    };
    if score < 0.0 {
        return;
    }

    let Some(terms) = estimated_acquisition_terms(game, index) else {
        return;
    };
    let stress = game.acquisition_stress_score(index).unwrap_or(1.0);
    let small_bolt_on = terms.acquired_customers <= game.player.customers.max(1.0) * 0.16;
    let uses_surprise_close = small_bolt_on && stress < 0.38;
    let diligence_cost = if game.has_diligence(index) || uses_surprise_close {
        0.0
    } else {
        game.diligence_cost(index).unwrap_or(0.0)
    };

    finance_to_cash(game, terms.net_cash_cost + diligence_cost + 16_000.0, 0.82);
    if !uses_surprise_close && !game.has_diligence(index) {
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

    let Some(close_terms) = estimated_acquisition_terms(game, index) else {
        return;
    };
    if game.player.cash > close_terms.net_cash_cost + 8_000.0
        && close_terms.post_debt_to_assets < 0.90
    {
        let _ = game.apply_decision(Decision::Acquire {
            competitor_index: index,
        });
    }
}

fn best_visible_deal(game: &Game) -> Option<(usize, f64)> {
    (0..game.competitors.len())
        .filter_map(|index| {
            let terms = estimated_acquisition_terms(game, index)?;
            let rival = &game.competitors[index];
            let stress = game.acquisition_stress_score(index).unwrap_or(1.0);
            let deal_share = terms.acquired_customers
                / (game.player.customers + terms.acquired_customers).max(1.0);
            if terms.post_debt_to_assets > 0.92 || deal_share > 0.24 {
                return None;
            }
            let customer_yield = terms.acquired_customers / terms.net_cash_cost.max(1.0);
            let rate_arbitrage = (rival.rate_cents - game.player.rate_cents).max(-1.0) * 0.002;
            let service_penalty = visible_target_service_penalty(rival);
            let score = customer_yield * 100.0 + rate_arbitrage - stress * 0.85 - service_penalty;
            Some((index, score))
        })
        .max_by(|left, right| left.1.total_cmp(&right.1))
}

fn visible_target_service_penalty(rival: &Utility) -> f64 {
    (0.80 - rival.reliability).max(0.0) * 0.75 + (48.0 - rival.reputation).max(0.0) / 180.0
}

fn arbitrage_visible_equity(game: &mut Game) {
    let book_equity = (game.player.asset_base + game.player.cash - game.player.debt).max(1.0);
    let market_cap = game.player.market_cap().max(1.0);
    if market_cap < book_equity * 0.55
        && game.player.cash > 60_000.0
        && game.player.debt_to_assets() < 0.72
    {
        let amount = (market_cap * 0.10).min(game.player.cash - 35_000.0);
        if amount >= 2_000.0 {
            let _ = game.apply_decision(Decision::BuyBackStock { amount });
        }
    } else if market_cap > book_equity * 1.25 && game.player.cash < 45_000.0 {
        let _ = game.apply_decision(Decision::IssueStock { amount: 24_000.0 });
    }
}

fn repay_idle_debt(game: &mut Game) {
    if game.player.cash <= 70_000.0 || game.player.debt_to_assets() <= 0.62 {
        return;
    }
    let amount = (game.player.cash - 45_000.0).min(game.player.debt);
    if amount >= 2_000.0 {
        let _ = game.apply_decision(Decision::RepayDebt { amount });
    }
}
