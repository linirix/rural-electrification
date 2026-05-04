//! Diagnostic strategy probes.
//!
//! These policies are not meant to be "main" benchmark personalities. Each one
//! pressures a specific visible mechanic: dividends, managers, capital-light
//! austerity, debt-heavy organic growth, pricing power, below-cost growth, and
//! distressed acquisitions.

use crate::sim::{
    DEFAULT_MAINTENANCE_MANAGER_TARGET, DEFAULT_MARKETING_MANAGER_TARGET,
    MAX_DISTRIBUTION_PROJECT_CUSTOMERS, MAX_GENERATION_PROJECT_MWH, MAX_PUBLIC_RATE_PREMIUM_CENTS,
    MIN_DISTRIBUTION_PROJECT_CUSTOMERS, MIN_GENERATION_PROJECT_MWH, REVIEW_MIN_RATE_SUPPORT_RATIO,
    public_rate_tolerance,
};
use crate::{AcquisitionTerms, Decision, Game, Utility};
use crate::{distribution_project_cost, generation_project_cost};

use super::organic::organic_policy;
use super::shared::{borrow_up_to, estimated_acquisition_terms, finance_to_cash};

pub(super) fn dividend_policy(game: &mut Game) {
    try_extract_dividend(game, 0.75);
    set_rate_toward(game, sustainable_rate_target(game, 0.86, 1.04), 0.45);
    debt_funded_projects(game, 440.0, 160.0, 0.78, 14_000.0);
    if game.market_share() < 0.45 && game.player.cash > 12_000.0 {
        let _ = game.apply_decision(Decision::Marketing { spend: 8_500.0 });
    }
    if game.player.reliability < 0.84 && game.player.cash > 10_000.0 {
        let _ = game.apply_decision(Decision::Maintenance { spend: 7_500.0 });
    }
    if game.player.debt_to_assets() > 0.80 {
        repay_if_cash_rich(game, 20_000.0);
    }
    try_extract_dividend(game, 0.35);
}

fn try_extract_dividend(game: &mut Game, profit_multiple: f64) {
    let profit = prior_profit(game);
    let reserve = if game.quarter < 8 { 8_000.0 } else { 12_000.0 };
    let operating_cushion_needed = game.player.capacity_headroom(&game.market) < 160.0
        || game.player.firm_generation_reserve_mwh(&game.market) < 70.0
        || game.player.reliability < 0.76;
    if profit <= 1_000.0 || operating_cushion_needed || game.player.cash <= reserve {
        return;
    }

    let dividend = (game.player.cash - reserve)
        .min(profit * profit_multiple)
        .min(game.player.cash * 0.45);
    if dividend >= 1_000.0 {
        let _ = game.apply_decision(Decision::DeclareDividend { amount: dividend });
    }
}

pub(super) fn manager_policy(game: &mut Game) {
    if game.maintenance_manager_target.is_none() {
        let _ = game.apply_decision(Decision::HireMaintenanceManager {
            target_reliability: DEFAULT_MAINTENANCE_MANAGER_TARGET,
        });
    }
    if game.marketing_manager_target.is_none() {
        let _ = game.apply_decision(Decision::HireMarketingManager {
            target_reputation: DEFAULT_MARKETING_MANAGER_TARGET,
        });
    }

    set_rate_toward(game, sustainable_rate_target(game, 0.88, 1.04), 0.45);
    finance_core_projects(game, 430.0, 150.0, 0.78, 14_000.0);
    if game.player.reliability < 0.76 && game.player.cash > 18_000.0 {
        let _ = game.apply_decision(Decision::Maintenance { spend: 8_000.0 });
    }
    repay_if_cash_rich(game, 38_000.0);
}

pub(super) fn austerity_policy(game: &mut Game) {
    set_rate_toward(game, sustainable_rate_target(game, 0.98, 1.12), 0.35);
    repay_if_cash_rich(game, 24_000.0);
    cash_funded_projects(game, 210.0, 85.0, 26_000.0);

    let profit = prior_profit(game);
    if profit > 3_000.0 && game.player.cash > 18_000.0 && game.market_share() < 0.42 {
        let spend = (profit * 0.22).clamp(1_000.0, 7_500.0);
        if game.player.cash > spend + 14_000.0 {
            let _ = game.apply_decision(Decision::Marketing { spend });
        }
    }
    if game.player.reliability < 0.84 && game.player.cash > 18_000.0 {
        let spend = (profit.max(4_000.0) * 0.28).clamp(3_000.0, 9_000.0);
        if game.player.cash > spend + 12_000.0 {
            let _ = game.apply_decision(Decision::Maintenance { spend });
        }
    }
}

pub(super) fn junkbond_policy(game: &mut Game) {
    set_rate_toward(game, sustainable_rate_target(game, 0.80, 1.00), 0.55);
    debt_funded_projects(game, 520.0, 190.0, 0.93, 18_000.0);
    if game.market_share() < 0.48 {
        borrow_up_to(game, 18_000.0, 0.93);
        if game.player.cash > 14_000.0 {
            let _ = game.apply_decision(Decision::Marketing { spend: 11_500.0 });
        }
    }
    if game.player.reliability < 0.84 {
        borrow_up_to(game, 10_000.0, 0.93);
        if game.player.cash > 10_000.0 {
            let _ = game.apply_decision(Decision::Maintenance { spend: 8_500.0 });
        }
    }
}

pub(super) fn ratehawk_policy(game: &mut Game) {
    let public_ceiling = public_rate_tolerance(&game.market) + MAX_PUBLIC_RATE_PREMIUM_CENTS - 0.15;
    let break_even = game.player_break_even_rate_cents().max(0.25);
    let target = public_ceiling
        .min(break_even * 1.55)
        .max(break_even * 1.05)
        .max(0.25);
    set_rate_toward(game, target, 0.60);

    finance_core_projects(game, 360.0, 145.0, 0.74, 18_000.0);
    if game.player.reliability < 0.88 && game.player.cash > 14_000.0 {
        let _ = game.apply_decision(Decision::Maintenance { spend: 10_000.0 });
    }
    if game.player.reputation < 78.0 && game.player.cash > 16_000.0 {
        let _ = game.apply_decision(Decision::Marketing { spend: 7_500.0 });
    }
    repay_if_cash_rich(game, 48_000.0);
}

pub(super) fn discountrunner_policy(game: &mut Game) {
    let break_even = game.player_break_even_rate_cents().max(0.25);
    let rival_anchor = visible_weighted_rival_rate(game).unwrap_or(game.market.standard_rate_cents);
    let target = (rival_anchor - 0.75)
        .min(break_even * (REVIEW_MIN_RATE_SUPPORT_RATIO + 0.02))
        .max(0.25);
    set_rate_toward(game, target, 0.70);

    debt_funded_projects(game, 610.0, 215.0, 0.84, 12_000.0);
    if game.player.cash < 24_000.0 {
        borrow_up_to(game, 22_000.0, 0.84);
    }
    if game.player.cash > 14_000.0 {
        let _ = game.apply_decision(Decision::Marketing { spend: 14_500.0 });
    }
    if game.player.reliability < 0.83 && game.player.cash > 11_000.0 {
        let _ = game.apply_decision(Decision::Maintenance { spend: 8_500.0 });
    }
}

pub(super) fn distressedbuyer_policy(game: &mut Game) {
    organic_policy(game);

    if game.quarter < 5 || game.acquisition_cooldown > 0 || game.competitors.len() <= 1 {
        return;
    }
    let Some((index, terms)) = distressed_target(game) else {
        return;
    };

    let diligence_cost = if game.has_diligence(index) {
        0.0
    } else {
        game.diligence_cost(index).unwrap_or(0.0)
    };
    finance_to_cash(game, terms.net_cash_cost + diligence_cost + 16_000.0, 0.84);

    if !game.has_diligence(index) {
        if game.player.cash > diligence_cost + 8_000.0 {
            let _ = game.apply_decision(Decision::Diligence {
                competitor_index: index,
            });
        }
        return;
    }

    if game.player.cash > terms.price + 12_000.0 && terms.post_debt_to_assets <= 0.88 {
        let _ = game.apply_decision(Decision::Acquire {
            competitor_index: index,
        });
    }
}

fn finance_core_projects(
    game: &mut Game,
    headroom_target: f64,
    reserve_target_mwh: f64,
    max_debt_to_assets: f64,
    cash_reserve: f64,
) {
    let headroom_deficit = headroom_target - game.player.capacity_headroom(&game.market);
    if headroom_deficit > 0.0 {
        let size = (420.0 + headroom_deficit * 0.35).clamp(
            MIN_DISTRIBUTION_PROJECT_CUSTOMERS,
            MAX_DISTRIBUTION_PROJECT_CUSTOMERS,
        );
        let cost = distribution_project_cost(size);
        finance_to_cash(game, cost + cash_reserve, max_debt_to_assets);
        if game.player.cash > cost + cash_reserve * 0.40 {
            let _ = game.apply_decision(Decision::BuildDistribution {
                customer_capacity: size,
            });
        }
    }

    let reserve_deficit =
        reserve_target_mwh - game.player.firm_generation_reserve_mwh(&game.market);
    if reserve_deficit > 0.0 {
        let size = (360.0 + reserve_deficit * 0.70)
            .clamp(MIN_GENERATION_PROJECT_MWH, MAX_GENERATION_PROJECT_MWH);
        let cost = generation_project_cost(size);
        finance_to_cash(game, cost + cash_reserve, max_debt_to_assets);
        if game.player.cash > cost + cash_reserve * 0.40 {
            let _ = game.apply_decision(Decision::BuildGeneration { capacity_mwh: size });
        }
    }
}

fn debt_funded_projects(
    game: &mut Game,
    headroom_target: f64,
    reserve_target_mwh: f64,
    max_debt_to_assets: f64,
    cash_reserve: f64,
) {
    let headroom_deficit = headroom_target - game.player.capacity_headroom(&game.market);
    if headroom_deficit > 0.0 {
        let size = (420.0 + headroom_deficit * 0.35).clamp(
            MIN_DISTRIBUTION_PROJECT_CUSTOMERS,
            MAX_DISTRIBUTION_PROJECT_CUSTOMERS,
        );
        let cost = distribution_project_cost(size);
        borrow_up_to(
            game,
            cost + cash_reserve - game.player.cash,
            max_debt_to_assets,
        );
        if game.player.cash > cost + cash_reserve * 0.35 {
            let _ = game.apply_decision(Decision::BuildDistribution {
                customer_capacity: size,
            });
        }
    }

    let reserve_deficit =
        reserve_target_mwh - game.player.firm_generation_reserve_mwh(&game.market);
    if reserve_deficit > 0.0 {
        let size = (360.0 + reserve_deficit * 0.70)
            .clamp(MIN_GENERATION_PROJECT_MWH, MAX_GENERATION_PROJECT_MWH);
        let cost = generation_project_cost(size);
        borrow_up_to(
            game,
            cost + cash_reserve - game.player.cash,
            max_debt_to_assets,
        );
        if game.player.cash > cost + cash_reserve * 0.35 {
            let _ = game.apply_decision(Decision::BuildGeneration { capacity_mwh: size });
        }
    }
}

fn cash_funded_projects(
    game: &mut Game,
    headroom_target: f64,
    reserve_target_mwh: f64,
    cash_reserve: f64,
) {
    if game.player.capacity_headroom(&game.market) < headroom_target {
        let size = 520.0;
        let cost = distribution_project_cost(size);
        if game.player.cash > cost + cash_reserve {
            let _ = game.apply_decision(Decision::BuildDistribution {
                customer_capacity: size,
            });
        }
    }
    if game.player.firm_generation_reserve_mwh(&game.market) < reserve_target_mwh {
        let size = 390.0;
        let cost = generation_project_cost(size);
        if game.player.cash > cost + cash_reserve {
            let _ = game.apply_decision(Decision::BuildGeneration { capacity_mwh: size });
        }
    }
}

fn distressed_target(game: &Game) -> Option<(usize, AcquisitionTerms)> {
    (0..game.competitors.len())
        .filter_map(|index| {
            let rival = &game.competitors[index];
            let terms = estimated_acquisition_terms(game, index)?;
            if !looks_distressed(rival) {
                return None;
            }
            if terms.post_debt_to_assets > 0.92 {
                return None;
            }
            if terms.price > game.player.asset_base.max(45_000.0) * 1.35 {
                return None;
            }
            let deal_share = terms.acquired_customers
                / (game.player.customers + terms.acquired_customers).max(1.0);
            if deal_share > 0.26 {
                return None;
            }
            let debt_discount = rival.debt_to_assets().clamp(0.0, 1.2) * 0.70;
            let reputation_discount = ((58.0 - rival.reputation).max(0.0) / 100.0) * 0.55;
            let service_discount = ((0.78 - rival.reliability).max(0.0)) * 0.90;
            let price_yield = terms.acquired_customers / terms.price.max(1.0) * 130.0;
            let score =
                deal_share + debt_discount + reputation_discount + service_discount + price_yield;
            Some((index, terms, score))
        })
        .max_by(|left, right| left.2.total_cmp(&right.2))
        .map(|(index, terms, _)| (index, terms))
}

fn looks_distressed(rival: &Utility) -> bool {
    rival.reputation < 55.0 || rival.reliability < 0.77 || rival.debt_to_assets() > 0.52
}

fn set_rate_toward(game: &mut Game, target: f64, max_step: f64) {
    let public_ceiling = public_rate_tolerance(&game.market) + MAX_PUBLIC_RATE_PREMIUM_CENTS - 0.05;
    let target = target.min(public_ceiling).max(0.25);
    let delta = (target - game.player.rate_cents).clamp(-max_step, max_step);
    if delta.abs() >= 0.05 {
        let _ = game.apply_decision(Decision::AdjustRate { delta_cents: delta });
    }
}

fn sustainable_rate_target(game: &Game, growth_ratio: f64, mature_ratio: f64) -> f64 {
    let break_even = game.player_break_even_rate_cents().max(0.25);
    let ratio = if game.market_share() < 0.45 {
        growth_ratio
    } else {
        mature_ratio
    };
    break_even * ratio
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

fn repay_if_cash_rich(game: &mut Game, reserve: f64) {
    let excess = (game.player.cash - reserve).max(0.0);
    let payment = game.player.debt.min(excess);
    if payment >= 1_000.0 {
        let _ = game.apply_decision(Decision::RepayDebt { amount: payment });
    }
}

fn prior_profit(game: &Game) -> f64 {
    game.last_report
        .as_ref()
        .map(|report| report.profit)
        .unwrap_or(0.0)
}
