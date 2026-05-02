use super::economics::{bounded_rate_target, defensive_rate_floor};
use super::{
    Game, MacroEnvironment, Market, Rng, Utility, high_rate_excess, maintenance_reliability_gain,
    public_balance_sheet_multipliers, public_rate_tolerance,
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

        let raw_rate_target = match reason {
            StartupEntryReason::ConcentratedMarket => market_avg_rate - 0.7,
            StartupEntryReason::HighRates => {
                if high_rate_excess(&self.player, &self.market) > 0.75 {
                    self.player.rate_cents - 1.2
                } else {
                    market_avg_rate - 0.8
                }
            }
            StartupEntryReason::StagnantMarket => self.market.standard_rate_cents - 0.4,
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
            StartupEntryReason::ConcentratedMarket => {
                1.15 + ((self.market_share() - 0.45) * 2.0).clamp(0.0, 0.70)
            }
            StartupEntryReason::HighRates => {
                1.0 + high_rate_excess(&self.player, &self.market).clamp(0.0, 1.6) * 0.18
            }
        };
        let starting_customers = (30.0 + self.rng.range(0.0, 25.0)) * scale;
        let (public_cash_multiplier, public_debt_multiplier) =
            public_balance_sheet_multipliers(&mut self.rng);
        let mut startup = Utility {
            name: name.clone(),
            cash: 6_000.0 + self.rng.range(0.0, 2_500.0) * scale,
            debt: 3_500.0 + self.rng.range(0.0, 2_000.0),
            shares: 0.0,
            stock_price: 0.0,
            customers: starting_customers,
            generation_capacity_mwh: (26.0 + self.rng.range(0.0, 14.0)) * scale,
            distribution_capacity: (90.0 + self.rng.range(0.0, 40.0)) * scale,
            rate_cents: raw_rate_target.max(0.25),
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
            public_cash_multiplier,
            public_debt_multiplier,
        };
        let defensive_floor = defensive_rate_floor(
            &startup,
            &self.market,
            &self.macro_state,
            self.cost_shock_multiplier(),
        );
        let public_ceiling =
            public_rate_tolerance(&self.market).min(self.market.standard_rate_cents + 2.0);
        startup.rate_cents = bounded_rate_target(raw_rate_target, defensive_floor, public_ceiling);
        let undercut = startup.rate_cents;
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

    pub(super) fn maybe_merge_rivals(&mut self, events: &mut Vec<String>) {
        if self.quarter < 8 || self.competitors.len() < 2 {
            return;
        }

        let player_share = self.market_share();
        if player_share < 0.52 {
            return;
        }

        let probability = (0.10 + (player_share - 0.52) * 0.85).clamp(0.08, 0.34);
        if self.rng.chance(probability) {
            self.merge_largest_rivals(events);
        }
    }

    pub(super) fn merge_largest_rivals(&mut self, events: &mut Vec<String>) -> bool {
        if self.competitors.len() < 2 {
            return false;
        }

        let mut indexes = (0..self.competitors.len()).collect::<Vec<_>>();
        indexes.sort_by(|left, right| {
            self.competitors[*right]
                .customers
                .total_cmp(&self.competitors[*left].customers)
        });
        let first = indexes[0];
        let second = indexes[1];
        let (higher, lower) = if first > second {
            (first, second)
        } else {
            (second, first)
        };
        let a = self.competitors.remove(higher);
        let b = self.competitors.remove(lower);
        let a_name = a.name.clone();
        let b_name = b.name.clone();
        let combined_customers = (a.customers + b.customers).max(1.0);
        let total_generation = a.generation_capacity_mwh + b.generation_capacity_mwh;
        let weighted_rate =
            (a.rate_cents * a.customers + b.rate_cents * b.customers) / combined_customers;
        let weighted_reputation =
            (a.reputation * a.customers + b.reputation * b.customers) / combined_customers;
        let weighted_reliability = if total_generation <= 0.0 {
            (a.reliability + b.reliability) / 2.0
        } else {
            (a.reliability * a.generation_capacity_mwh + b.reliability * b.generation_capacity_mwh)
                / total_generation
        };
        let (public_cash_multiplier, public_debt_multiplier) =
            public_balance_sheet_multipliers(&mut self.rng);

        self.startup_index += 1;
        let mut used_names = vec![self.player.name.clone()];
        used_names.extend(
            self.competitors
                .iter()
                .map(|competitor| competitor.name.clone()),
        );
        let name = draw_competitor_name(&mut self.rng, &used_names, self.startup_index);
        let merged = Utility {
            name: name.clone(),
            cash: a.cash + b.cash + 12_000.0,
            debt: a.debt + b.debt + 12_000.0,
            shares: 0.0,
            stock_price: 0.0,
            customers: combined_customers * 0.99,
            generation_capacity_mwh: total_generation * 0.97,
            distribution_capacity: (a.distribution_capacity + b.distribution_capacity) * 0.98,
            rate_cents: weighted_rate,
            reputation: (weighted_reputation + 4.0).clamp(0.0, 100.0),
            reliability: (weighted_reliability + 0.015).clamp(0.35, 0.96),
            marketing_momentum: a.marketing_momentum + b.marketing_momentum + 0.20,
            asset_base: (a.asset_base + b.asset_base) * 0.94,
            last_quarter_customers: combined_customers,
            public_cash_multiplier,
            public_debt_multiplier,
        };
        let defensive_floor = defensive_rate_floor(
            &merged,
            &self.market,
            &self.macro_state,
            self.cost_shock_multiplier(),
        );
        let public_ceiling = public_rate_tolerance(&self.market).min(weighted_rate + 1.5);
        let mut merged = merged;
        merged.rate_cents =
            bounded_rate_target(weighted_rate - 0.25, defensive_floor, public_ceiling);
        self.competitors.push(merged);
        events.push(format!(
            "{name} formed from {a_name} and {b_name}, creating a stronger rival to challenge Metro's lead."
        ));
        true
    }

    pub(super) fn maybe_rival_counteroffensive(&mut self, events: &mut Vec<String>) {
        if self.quarter < 6 || self.competitors.is_empty() {
            return;
        }

        let player_share = self.market_share();
        if player_share < 0.48 {
            return;
        }

        let probability = (0.08 + (player_share - 0.48) * 0.90).clamp(0.08, 0.42);
        if self.rng.chance(probability) {
            self.rival_counteroffensive(events);
        }
    }

    pub(super) fn rival_counteroffensive(&mut self, events: &mut Vec<String>) -> bool {
        let Some(index) = self.largest_competitor_index() else {
            return false;
        };

        let player_rate = self.player.rate_cents;
        let market = self.market.clone();
        let macro_state = self.macro_state.clone();
        let cost_multiplier = self.cost_shock_multiplier();
        let competitor = &mut self.competitors[index];
        let capital = (12_000.0 + competitor.asset_base * 0.055).clamp(12_000.0, 36_000.0);
        let debt_raise = capital * 0.30;
        competitor.debt += debt_raise;
        competitor.asset_base += capital;
        competitor.generation_capacity_mwh += capital / 360.0;
        competitor.distribution_capacity += capital / 70.0;
        competitor.marketing_momentum = (competitor.marketing_momentum + 0.34).clamp(0.0, 1.30);
        competitor.reputation = (competitor.reputation + 4.5).clamp(0.0, 100.0);
        competitor.reliability = (competitor.reliability + 0.018).clamp(0.35, 0.97);

        if competitor.rate_cents > player_rate - 0.15 {
            let defensive_floor =
                defensive_rate_floor(competitor, &market, &macro_state, cost_multiplier);
            let target = (player_rate - 0.20).max(defensive_floor).max(0.25);
            competitor.rate_cents = target.min(competitor.rate_cents);
        }

        events.push(format!(
            "{} secured outside financing for a counteroffensive: lower rates, new capacity, and heavier customer acquisition.",
            competitor.name
        ));
        true
    }

    fn largest_competitor_index(&self) -> Option<usize> {
        self.competitors
            .iter()
            .enumerate()
            .max_by(|(_, left), (_, right)| left.customers.total_cmp(&right.customers))
            .map(|(index, _)| index)
    }

    pub(super) fn competitor_plans(&mut self, events: &mut Vec<String>) {
        let context = self.competitor_plan_context();
        let competitor_count = self.competitors.len();

        for index in 0..competitor_count {
            let rolls = self.competitor_plan_rolls();

            if self.competitor_under_active_diligence(index)
                && let Some(event) =
                    self.competitor_diligence_window_response(index, &context, &rolls)
            {
                events.push(event);
            }
            if let Some(event) = self.competitor_rate_decision(index, &context) {
                events.push(event);
            }
            if let Some(event) = self.competitor_marketing_decision(index, &context, &rolls) {
                events.push(event);
            }
            if let Some(event) = self.competitor_capacity_decision(index, &context, &rolls) {
                events.push(event);
            }
            if let Some(event) = self.competitor_capital_decision(index, &context, &rolls) {
                events.push(event);
            }
            self.competitor_maintenance_decision(index);
        }
    }

    fn competitor_diligence_window_response(
        &mut self,
        index: usize,
        context: &CompetitorPlanContext,
        rolls: &CompetitorPlanRolls,
    ) -> Option<String> {
        let competitor = &mut self.competitors[index];
        let mut acted = false;

        if competitor.cash > 3_500.0 {
            let spend = (competitor.cash * 0.04).clamp(600.0, 1_800.0);
            competitor.cash -= spend;
            competitor.marketing_momentum =
                (competitor.marketing_momentum + spend / 30_000.0).clamp(0.0, 1.75);
            competitor.reputation = (competitor.reputation + spend / 20_000.0).clamp(0.0, 100.0);
            acted = true;
        }

        if !context.rate_frozen && competitor.rate_cents > context.player_rate + 0.25 {
            let defensive_floor = defensive_rate_floor(
                competitor,
                &context.market,
                &context.macro_state,
                context.cost_multiplier,
            );
            let target = (context.player_rate + 0.25).max(defensive_floor);
            let cut = 0.10_f64.min((competitor.rate_cents - target).max(0.0));
            if cut > 0.0 {
                competitor.rate_cents -= cut;
                acted = true;
            }
        }

        if rolls.proactive_expansion_chance && competitor.cash > 14_000.0 {
            let capex = (1_400.0 + rolls.expansion_cost_jitter * 0.35).min(competitor.cash * 0.07);
            competitor.cash -= capex;
            competitor.asset_base += capex;
            competitor.distribution_capacity += capex / 95.0;
            competitor.generation_capacity_mwh += capex / 460.0;
            acted = true;
        }

        acted.then(|| {
            format!(
                "{} accelerated retention and defenses while under diligence.",
                competitor.name
            )
        })
    }

    fn competitor_plan_context(&self) -> CompetitorPlanContext {
        let total_last = (self.player.last_quarter_customers
            + self
                .competitors
                .iter()
                .map(|competitor| competitor.last_quarter_customers)
                .sum::<f64>())
        .max(1.0);

        CompetitorPlanContext {
            player_rate: self.player.rate_cents,
            player_share: self.market_share(),
            demand_signal: self.macro_state.demand_index,
            rate_frozen: self.rate_frozen(),
            total_now: self.total_connected_customers().max(1.0),
            total_last,
            market: self.market.clone(),
            macro_state: self.macro_state.clone(),
            cost_multiplier: self.cost_shock_multiplier(),
        }
    }

    fn competitor_plan_rolls(&mut self) -> CompetitorPlanRolls {
        CompetitorPlanRolls {
            routine_marketing_chance: self.rng.chance(0.24),
            routine_marketing_jitter: self.rng.range(0.0, 2_000.0),
            expansion_cost_jitter: self.rng.range(0.0, 2_000.0),
            expansion_lines_jitter: self.rng.range(0.0, 45.0),
            expansion_gen_jitter: self.rng.range(0.0, 30.0),
            proactive_expansion_chance: self.rng.chance(0.22),
            strategic_debt_chance: self.rng.chance(0.32),
        }
    }

    fn competitor_rate_decision(
        &mut self,
        index: usize,
        context: &CompetitorPlanContext,
    ) -> Option<String> {
        let competitor = &mut self.competitors[index];
        let lost_share = competitor_lost_share(competitor, context);
        let player_undercutting = context.player_rate + 0.4 < competitor.rate_cents;
        let player_premium_pricing = context.player_rate > competitor.rate_cents + 0.5;
        let near_capacity =
            competitor.capacity_headroom(&context.market) < competitor.customers * 0.06;

        let defensive_floor = defensive_rate_floor(
            competitor,
            &context.market,
            &context.macro_state,
            context.cost_multiplier,
        );
        let dynamic_ceiling =
            (public_rate_tolerance(&context.market) + 2.0).max(defensive_floor + 0.30);
        if lost_share && player_undercutting && competitor.rate_cents > defensive_floor + 0.05 {
            let target = (context.player_rate + 0.35).max(defensive_floor);
            let cut = 0.4_f64.min((competitor.rate_cents - target).max(0.0));
            if cut > 0.0 {
                competitor.rate_cents -= cut;
                return Some(format!(
                    "{} cut rates to {:.1}c/kWh to defend share.",
                    competitor.name, competitor.rate_cents
                ));
            }
        } else if !context.rate_frozen
            && player_premium_pricing
            && near_capacity
            && competitor.rate_cents < dynamic_ceiling - 0.05
        {
            let raise = 0.3_f64.min((dynamic_ceiling - competitor.rate_cents).max(0.0));
            if raise > 0.0 {
                competitor.rate_cents += raise;
                return Some(format!(
                    "{} raised rates to {:.1}c/kWh, riding strong demand.",
                    competitor.name, competitor.rate_cents
                ));
            }
        }

        None
    }

    fn competitor_marketing_decision(
        &mut self,
        index: usize,
        context: &CompetitorPlanContext,
        rolls: &CompetitorPlanRolls,
    ) -> Option<String> {
        let competitor = &mut self.competitors[index];
        let lost_share = competitor_lost_share(competitor, context);

        if lost_share && competitor.cash > 4_500.0 {
            let spend = (competitor.cash * 0.18).clamp(2_500.0, 6_500.0);
            competitor.cash -= spend;
            competitor.marketing_momentum += spend / 12_000.0;
            competitor.reputation = (competitor.reputation + spend / 5_500.0).clamp(0.0, 100.0);
            Some(format!(
                "{} launched a retention campaign to win back customers.",
                competitor.name
            ))
        } else if rolls.routine_marketing_chance && competitor.cash > 2_500.0 {
            let spend = 1_600.0 + rolls.routine_marketing_jitter;
            competitor.cash -= spend;
            competitor.marketing_momentum += spend / 18_000.0;
            competitor.reputation = (competitor.reputation + spend / 7_000.0).clamp(0.0, 100.0);
            None
        } else {
            None
        }
    }

    fn competitor_capacity_decision(
        &mut self,
        index: usize,
        context: &CompetitorPlanContext,
        rolls: &CompetitorPlanRolls,
    ) -> Option<String> {
        let competitor = &mut self.competitors[index];
        let headroom = competitor.capacity_headroom(&context.market);
        let reactive_pressure = headroom < context.market.addressable_customers * 0.025;
        let proactive_growth = rolls.proactive_expansion_chance
            && (context.demand_signal > 0.10 || context.player_share > 0.40);
        if (reactive_pressure || proactive_growth) && competitor.cash > 8_500.0 {
            let line_cost = 6_000.0 + rolls.expansion_cost_jitter;
            competitor.cash -= line_cost;
            competitor.asset_base += line_cost;
            competitor.distribution_capacity += 115.0 + rolls.expansion_lines_jitter;
            competitor.generation_capacity_mwh += 42.0 + rolls.expansion_gen_jitter;
            Some(format!(
                "{} expanded generation and distribution capacity.",
                competitor.name
            ))
        } else {
            None
        }
    }

    fn competitor_capital_decision(
        &mut self,
        index: usize,
        context: &CompetitorPlanContext,
        rolls: &CompetitorPlanRolls,
    ) -> Option<String> {
        let competitor = &mut self.competitors[index];
        let lost_share = competitor_lost_share(competitor, context);
        let leverage = competitor.debt / competitor.asset_base.max(1.0);
        let losing_to_player = context.player_share > 0.45 && lost_share;
        if losing_to_player && leverage < 0.65 && rolls.strategic_debt_chance {
            let raise = 12_000.0 + competitor.asset_base * 0.05;
            competitor.cash += raise;
            competitor.debt += raise;
            Some(format!(
                "{} raised debt to fund a counter-expansion.",
                competitor.name
            ))
        } else {
            None
        }
    }

    fn competitor_maintenance_decision(&mut self, index: usize) {
        let competitor = &mut self.competitors[index];
        if competitor.reliability < 0.78 && competitor.cash > 3_500.0 {
            let spend = 4_000.0_f64.min(competitor.cash * 0.30);
            competitor.cash -= spend;
            let gain = maintenance_reliability_gain(competitor, spend);
            competitor.reliability = (competitor.reliability + gain).clamp(0.35, 0.98);
        }
    }
}

struct CompetitorPlanContext {
    player_rate: f64,
    player_share: f64,
    demand_signal: f64,
    rate_frozen: bool,
    total_now: f64,
    total_last: f64,
    market: Market,
    macro_state: MacroEnvironment,
    cost_multiplier: f64,
}

struct CompetitorPlanRolls {
    routine_marketing_chance: bool,
    routine_marketing_jitter: f64,
    expansion_cost_jitter: f64,
    expansion_lines_jitter: f64,
    expansion_gen_jitter: f64,
    proactive_expansion_chance: bool,
    strategic_debt_chance: bool,
}

fn competitor_lost_share(competitor: &Utility, context: &CompetitorPlanContext) -> bool {
    let share_now = competitor.customers / context.total_now;
    let share_last = competitor.last_quarter_customers / context.total_last;
    competitor.last_quarter_customers > 0.0 && share_now < share_last - 0.005
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
