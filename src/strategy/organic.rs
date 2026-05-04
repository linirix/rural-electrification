//! Disciplined growth strategies.
//!
//! This module holds the organic, balanced, and regional policies. Organic is
//! the low-variance proof that pricing, service quality, and capital projects
//! can win without deals. Balanced layers opportunistic acquisitions on top.
//! Regional extends that operating posture into the post-review Y10 mandate.

use crate::sim::{
    MAX_PUBLIC_RATE_PREMIUM_CENTS, REGIONAL_MANDATE_EXPANSION_TARGET,
    REGIONAL_MANDATE_LEVERAGE_LIMIT, REGIONAL_MANDATE_RELIABILITY_TARGET,
    REGIONAL_MANDATE_SHARE_TARGET, public_rate_tolerance,
};
use crate::{AcquisitionTerms, Decision, Game, distribution_project_cost, generation_project_cost};

use super::shared::{estimated_acquisition_terms, finance_to_cash};

pub(super) fn organic_policy(game: &mut Game) {
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

pub(super) fn balanced_policy(game: &mut Game) {
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

pub(super) fn regional_policy(game: &mut Game) {
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
