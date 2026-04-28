use crate::sim::{
    MAX_DISTRIBUTION_PROJECT_CUSTOMERS, MAX_GENERATION_PROJECT_MWH, MAX_PUBLIC_RATE_PREMIUM_CENTS,
    MIN_DISTRIBUTION_PROJECT_CUSTOMERS, MIN_GENERATION_PROJECT_MWH, public_rate_tolerance,
};
use crate::{Decision, Game, InitialVariance, distribution_project_cost, generation_project_cost};

use super::shared::estimated_acquisition_terms;

pub(super) fn glonzo_policy(game: &mut Game, rng: &mut GlonzoRng) {
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

pub(super) struct GlonzoRng {
    state: u64,
}

impl GlonzoRng {
    pub(super) fn from_seed(game_seed: u64, variance: InitialVariance) -> Self {
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
