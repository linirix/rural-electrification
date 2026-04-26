use super::{
    Game, Rng, Utility, high_rate_excess, maintenance_reliability_gain, public_rate_tolerance,
};

pub(super) const COMPETITOR_NAME_POOL: &[&str] = &[
    "Arc Light Power",
    "Blue River Electric",
    "Bright Path Power",
    "Civic Current",
    "Clear Grid Partners",
    "Commonwealth Power",
    "Core Line Energy",
    "Cross-Town Electric",
    "District Current",
    "First Service Power",
    "Harbor Grid",
    "Keystone Electric",
    "Metro Light",
    "New Current Co.",
    "North Loop Power",
    "Pioneer Utility",
    "Public Circuit",
    "Reliant Service",
    "Signal Electric",
    "Switchback Energy",
    "Union Grid",
    "Vector Power",
    "Voltaic Cooperative",
    "Westside Current",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum StartupEntryReason {
    ConcentratedMarket,
    HighRates,
    StagnantMarket,
}

impl Game {
    pub(super) fn maybe_spawn_startup(&mut self, events: &mut Vec<String>) {
        if self.competitors.len() >= 5 {
            return;
        }

        let market_avg_rate = self.average_rate();
        let Some((reason, probability)) = self.startup_entry_signal() else {
            return;
        };

        if !self.rng.chance(probability) {
            return;
        }

        let undercut = match reason {
            StartupEntryReason::ConcentratedMarket => (market_avg_rate - 0.7).clamp(8.4, 12.8),
            StartupEntryReason::HighRates => {
                let target = if high_rate_excess(&self.player, &self.market) > 0.75 {
                    self.player.rate_cents - 1.2
                } else {
                    market_avg_rate - 0.8
                };
                target.clamp(8.5, 12.9)
            }
            StartupEntryReason::StagnantMarket => {
                (self.market.standard_rate_cents - 0.4).clamp(8.6, 11.6)
            }
        };
        self.startup_index += 1;
        let mut used_names = vec![self.player.name.clone()];
        used_names.extend(
            self.competitors
                .iter()
                .map(|competitor| competitor.name.clone()),
        );
        let name = draw_competitor_name(&mut self.rng, &used_names, self.startup_index);
        let scale = match reason {
            StartupEntryReason::StagnantMarket => 1.35,
            StartupEntryReason::ConcentratedMarket => 1.15,
            StartupEntryReason::HighRates => 1.0,
        };
        let starting_customers = (30.0 + self.rng.range(0.0, 25.0)) * scale;
        let startup = Utility {
            name: name.clone(),
            cash: 6_000.0 + self.rng.range(0.0, 2_500.0) * scale,
            debt: 3_500.0 + self.rng.range(0.0, 2_000.0),
            shares: 0.0,
            stock_price: 0.0,
            customers: starting_customers,
            generation_capacity_mwh: (26.0 + self.rng.range(0.0, 14.0)) * scale,
            distribution_capacity: (90.0 + self.rng.range(0.0, 40.0)) * scale,
            rate_cents: undercut,
            reputation: 47.0 + self.rng.range(0.0, 6.0) + if scale > 1.0 { 2.5 } else { 0.0 },
            reliability: 0.74 + self.rng.range(0.0, 0.05) + if scale > 1.0 { 0.02 } else { 0.0 },
            marketing_momentum: 0.10
                + if reason == StartupEntryReason::StagnantMarket {
                    0.08
                } else {
                    0.0
                },
            asset_base: (22_000.0 + self.rng.range(0.0, 5_000.0)) * scale,
            last_quarter_customers: starting_customers,
        };
        self.competitors.push(startup);
        events.push(format!(
            "{} {} at {:.1}c/kWh.",
            name,
            startup_entry_message(reason),
            undercut
        ));
    }

    pub(super) fn startup_entry_signal(&self) -> Option<(StartupEntryReason, f64)> {
        let player_share = self.market_share();
        let market_avg_rate = self.average_rate();
        let player_rate_excess = high_rate_excess(&self.player, &self.market);
        let market_rate_excess = (market_avg_rate - public_rate_tolerance(&self.market)).max(0.0);
        let total_connected = self.total_connected_customers();
        let addressable = self.market.addressable_customers.max(1.0);
        let serviceable_gap = (self.market.serviceable_customers() - total_connected).max(0.0);
        let addressable_gap = (addressable - total_connected).max(0.0);
        let connected_penetration = total_connected / addressable;
        let expected_penetration = (0.115 + self.quarter as f64 * 0.010).clamp(0.12, 0.38);
        let average_reliability = self.market_average_reliability();

        let concentrated = player_share > 0.55
            || self.competitors.len() <= 2 && player_share > 0.42
            || self.competitors.len() <= 1;
        let high_rates = market_avg_rate > 11.2
            || market_avg_rate > self.market.standard_rate_cents + 0.75
            || player_rate_excess > 0.40
            || market_rate_excess > 0.20;
        let stagnant = self.quarter >= 4
            && connected_penetration < expected_penetration
            && addressable_gap > addressable * 0.45
            && (self.macro_state.demand_index < -0.15
                || average_reliability < 0.77
                || serviceable_gap < addressable * 0.015);
        let open_opportunity = serviceable_gap > 45.0
            || addressable_gap > addressable * 0.35
            || high_rates
            || concentrated;

        if !open_opportunity || !(concentrated || high_rates || stagnant) {
            return None;
        }

        let mut probability: f64 = 0.03;
        if concentrated {
            probability += 0.08;
        }
        if high_rates {
            probability += 0.06;
            probability += player_rate_excess * 0.025 + market_rate_excess * 0.020;
        }
        if stagnant {
            probability += 0.09;
        }
        if serviceable_gap > addressable * 0.05 {
            probability += 0.04;
        }
        if self.competitors.len() <= 1 {
            probability += 0.05;
        }

        let reason = if stagnant {
            StartupEntryReason::StagnantMarket
        } else if high_rates && player_rate_excess > 0.75 {
            StartupEntryReason::HighRates
        } else if concentrated {
            StartupEntryReason::ConcentratedMarket
        } else {
            StartupEntryReason::HighRates
        };

        Some((reason, probability.clamp(0.04, 0.36)))
    }

    fn market_average_reliability(&self) -> f64 {
        let player_weight = self.player.customers.max(0.0);
        let mut weighted = self.player.reliability * player_weight;
        let mut customers = player_weight;
        for competitor in &self.competitors {
            let weight = competitor.customers.max(0.0);
            weighted += competitor.reliability * weight;
            customers += weight;
        }

        if customers <= 0.0 {
            self.player.reliability
        } else {
            weighted / customers
        }
    }

    pub(super) fn competitor_plans(&mut self, events: &mut Vec<String>) {
        let player_rate = self.player.rate_cents;
        let player_share = self.market_share();
        let market = self.market.clone();
        let demand_signal = self.macro_state.demand_index;
        let competitor_count = self.competitors.len();
        let rate_frozen = self.rate_frozen();

        let total_now = self.total_connected_customers().max(1.0);
        let total_last = (self.player.last_quarter_customers
            + self
                .competitors
                .iter()
                .map(|competitor| competitor.last_quarter_customers)
                .sum::<f64>())
        .max(1.0);

        for index in 0..competitor_count {
            let routine_marketing_chance = self.rng.chance(0.24);
            let routine_marketing_jitter = self.rng.range(0.0, 2_000.0);
            let expansion_cost_jitter = self.rng.range(0.0, 2_000.0);
            let expansion_lines_jitter = self.rng.range(0.0, 45.0);
            let expansion_gen_jitter = self.rng.range(0.0, 30.0);
            let proactive_expansion_chance = self.rng.chance(0.22);
            let strategic_debt_chance = self.rng.chance(0.32);

            let competitor = &mut self.competitors[index];

            let share_now = competitor.customers / total_now;
            let share_last = competitor.last_quarter_customers / total_last;
            let lost_share =
                competitor.last_quarter_customers > 0.0 && share_now < share_last - 0.005;
            let player_undercutting = player_rate + 0.4 < competitor.rate_cents;
            let player_premium_pricing = player_rate > competitor.rate_cents + 0.5;
            let near_capacity = competitor.capacity_headroom(&market) < competitor.customers * 0.06;

            if lost_share && player_undercutting && competitor.rate_cents > 8.4 {
                let cut = 0.4_f64.min(competitor.rate_cents - 8.4);
                if cut > 0.0 {
                    competitor.rate_cents -= cut;
                    events.push(format!(
                        "{} cut rates to {:.1}c/kWh to defend share.",
                        competitor.name, competitor.rate_cents
                    ));
                }
            } else if !rate_frozen
                && player_premium_pricing
                && near_capacity
                && competitor.rate_cents < 13.4
            {
                let raise = 0.3_f64.min(13.5 - competitor.rate_cents);
                if raise > 0.0 {
                    competitor.rate_cents += raise;
                    events.push(format!(
                        "{} raised rates to {:.1}c/kWh, riding strong demand.",
                        competitor.name, competitor.rate_cents
                    ));
                }
            }

            if lost_share && competitor.cash > 4_500.0 {
                let spend = (competitor.cash * 0.18).clamp(2_500.0, 6_500.0);
                competitor.cash -= spend;
                competitor.marketing_momentum += spend / 12_000.0;
                competitor.reputation = (competitor.reputation + spend / 5_500.0).clamp(0.0, 100.0);
                events.push(format!(
                    "{} launched a retention campaign to win back customers.",
                    competitor.name
                ));
            } else if routine_marketing_chance && competitor.cash > 2_500.0 {
                let spend = 1_600.0 + routine_marketing_jitter;
                competitor.cash -= spend;
                competitor.marketing_momentum += spend / 18_000.0;
                competitor.reputation = (competitor.reputation + spend / 7_000.0).clamp(0.0, 100.0);
            }

            let headroom = competitor.capacity_headroom(&market);
            let reactive_pressure = headroom < market.addressable_customers * 0.025;
            let proactive_growth =
                proactive_expansion_chance && (demand_signal > 0.10 || player_share > 0.40);
            if (reactive_pressure || proactive_growth) && competitor.cash > 8_500.0 {
                let line_cost = 6_000.0 + expansion_cost_jitter;
                competitor.cash -= line_cost;
                competitor.asset_base += line_cost;
                competitor.distribution_capacity += 115.0 + expansion_lines_jitter;
                competitor.generation_capacity_mwh += 42.0 + expansion_gen_jitter;
                events.push(format!(
                    "{} expanded generation and distribution capacity.",
                    competitor.name
                ));
            }

            let leverage = competitor.debt / competitor.asset_base.max(1.0);
            let losing_to_player = player_share > 0.45 && lost_share;
            if losing_to_player && leverage < 0.65 && strategic_debt_chance {
                let raise = 12_000.0 + competitor.asset_base * 0.05;
                competitor.cash += raise;
                competitor.debt += raise;
                events.push(format!(
                    "{} raised debt to fund a counter-expansion.",
                    competitor.name
                ));
            }

            if competitor.reliability < 0.78 && competitor.cash > 3_500.0 {
                let spend = 4_000.0_f64.min(competitor.cash * 0.30);
                competitor.cash -= spend;
                let gain = maintenance_reliability_gain(competitor, spend);
                competitor.reliability = (competitor.reliability + gain).clamp(0.35, 0.98);
            }
        }
    }
}

fn startup_entry_message(reason: StartupEntryReason) -> &'static str {
    match reason {
        StartupEntryReason::ConcentratedMarket => {
            "formed to challenge a concentrated market and entered"
        }
        StartupEntryReason::HighRates => "entered to chase rate-sensitive customers",
        StartupEntryReason::StagnantMarket => {
            "formed around under-served districts in a stagnant market and entered"
        }
    }
}

pub(super) fn draw_competitor_name(
    rng: &mut Rng,
    used_names: &[String],
    fallback_index: u32,
) -> String {
    let available = COMPETITOR_NAME_POOL
        .iter()
        .copied()
        .filter(|candidate| !used_names.iter().any(|used| used == candidate))
        .collect::<Vec<_>>();

    if !available.is_empty() {
        let index = rng.next_index(available.len());
        return available[index].to_string();
    }

    let base = COMPETITOR_NAME_POOL[rng.next_index(COMPETITOR_NAME_POOL.len())];
    let mut suffix = fallback_index.max(1);
    loop {
        let candidate = format!("{base} {suffix}");
        if !used_names.iter().any(|used| used == &candidate) {
            return candidate;
        }
        suffix += 1;
    }
}
