use super::{
    DISTRIBUTION_PROJECT_CAPACITY, DISTRIBUTION_PROJECT_COST, FirmFinances,
    GENERATION_PROJECT_CAPACITY_MWH, GENERATION_PROJECT_COST, MAINTENANCE_REFERENCE_ASSET_BASE,
    MAX_DISTRIBUTION_PROJECT_CUSTOMERS, MAX_GENERATION_PROJECT_MWH, MAX_RELIABILITY,
    MIN_DISTRIBUTION_PROJECT_CUSTOMERS, MIN_GENERATION_PROJECT_MWH, MacroEnvironment, Market,
    Utility,
};

const SERVICE_REPUTATION_RELIABILITY_THRESHOLD: f64 = 0.84;
const SERVICE_REPUTATION_UTILIZATION_LIMIT: f64 = 0.82;
const MIN_REPUTATION_EVENT_POINTS: f64 = 0.05;

pub(super) fn maintenance_reliability_gain(utility: &Utility, spend: f64) -> f64 {
    (spend / 22_000.0)
        * (1.05_f64 - utility.reliability).max(0.05)
        * maintenance_asset_scale(utility)
}

pub(super) fn maintenance_spend_for_reliability_target(utility: &Utility, target: f64) -> f64 {
    let target = target.clamp(0.35, MAX_RELIABILITY);
    if target <= utility.reliability {
        return 0.0;
    }
    let response =
        ((1.05_f64 - utility.reliability).max(0.05) * maintenance_asset_scale(utility)).max(0.001);
    ((target - utility.reliability) * 22_000.0 / response).max(0.0)
}

pub(super) fn maintenance_reputation_gain(utility: &Utility, spend: f64) -> f64 {
    spend / 5_500.0 * maintenance_asset_scale(utility)
}

fn maintenance_asset_scale(utility: &Utility) -> f64 {
    (MAINTENANCE_REFERENCE_ASSET_BASE / utility.asset_base.max(20_000.0))
        .sqrt()
        .clamp(0.35, 1.45)
}

pub(super) fn settle_utility(
    utility: &mut Utility,
    market: &Market,
    macro_state: &MacroEnvironment,
    cost_multiplier: f64,
    is_player: bool,
    events: &mut Vec<String>,
) -> FirmFinances {
    let demanded_mwh = utility.customers * market.avg_mwh_per_customer;
    let served_mwh =
        demanded_mwh.min(utility.generation_capacity_mwh * utility.reliability.max(0.45));
    let unmet_demand_ratio = if demanded_mwh <= 0.0 {
        0.0
    } else {
        1.0 - served_mwh / demanded_mwh
    };

    let revenue = served_mwh * utility.rate_cents * 10.0;
    let operating_cost = operating_cost_for(utility, market, served_mwh, cost_multiplier);
    let interest = utility.debt * macro_state.annual_interest_rate_for(utility) / 4.0;
    let profit = revenue - operating_cost - interest;

    utility.cash += profit;
    if utility.cash < 0.0 {
        let overdraft = -utility.cash;
        utility.debt += overdraft;
        utility.cash = 0.0;
        utility.reputation = (utility.reputation - 1.8).clamp(0.0, 100.0);
        if is_player {
            events.push(format!(
                "Cash crunch: {} overdraft converted to debt; reputation -1.8 pts.",
                money(overdraft)
            ));
        }
    }

    utility.reliability = (utility.reliability - 0.014).clamp(0.35, 0.98);

    if unmet_demand_ratio > 0.04 {
        utility.reliability = (utility.reliability - unmet_demand_ratio * 0.22).clamp(0.35, 0.98);
        utility.reputation = (utility.reputation - unmet_demand_ratio * 10.0).clamp(0.0, 100.0);
        if is_player {
            events.push(format!(
                "Overloaded generation left {:.0}% of demand unmet and hurt public confidence.",
                unmet_demand_ratio * 100.0
            ));
        }
    } else {
        utility.reliability = (utility.reliability + 0.004).clamp(0.35, 0.98);
    }

    let rate_excess = high_rate_excess(utility, market);
    if rate_excess > 0.0 {
        let impatience = 1.0 - market.civic_patience;
        let reputation_damage = (rate_excess * (0.45 + impatience * 0.35)
            + rate_excess * rate_excess * 0.08)
            .clamp(0.0, 8.0);
        utility.reputation = (utility.reputation - reputation_damage).clamp(0.0, 100.0);
        if is_player {
            events.push(format!(
                "Rates above public tolerance damaged reputation by {:.1} points.",
                reputation_damage
            ));
        }
    }

    let service_reputation_gain = service_reputation_gain(utility, market, unmet_demand_ratio);
    if service_reputation_gain > 0.0 {
        let old_reputation = utility.reputation;
        utility.reputation = (old_reputation + service_reputation_gain).clamp(0.0, 100.0);
        let actual_gain = utility.reputation - old_reputation;
        if is_player && actual_gain >= MIN_REPUTATION_EVENT_POINTS {
            events.push(format!(
                "Reliable service with manageable utilization lifted Metro's reputation by {:.1} points.",
                actual_gain
            ));
        }
    }

    FirmFinances {
        revenue,
        operating_cost,
        interest,
        profit,
        served_mwh,
        unmet_demand_ratio,
    }
}

pub(super) fn break_even_rate_cents(
    utility: &Utility,
    market: &Market,
    macro_state: &MacroEnvironment,
    cost_multiplier: f64,
) -> f64 {
    let demanded_mwh = utility.customers * market.avg_mwh_per_customer;
    let served_mwh =
        demanded_mwh.min(utility.generation_capacity_mwh * utility.reliability.max(0.45));
    if served_mwh <= 0.0 {
        return 0.0;
    }

    let operating_cost = operating_cost_for(utility, market, served_mwh, cost_multiplier);
    let interest = utility.debt * macro_state.annual_interest_rate_for(utility) / 4.0;
    ((operating_cost + interest) / served_mwh / 10.0).max(0.0)
}

pub(super) fn defensive_rate_floor(
    utility: &Utility,
    market: &Market,
    macro_state: &MacroEnvironment,
    cost_multiplier: f64,
) -> f64 {
    let break_even = break_even_rate_cents(utility, market, macro_state, cost_multiplier);
    let variable_cost_floor = market.variable_cost_per_mwh / 10.0;
    (break_even * 0.88)
        .max(variable_cost_floor * 1.05)
        .max(0.25)
}

pub(super) fn bounded_rate_target(target: f64, floor: f64, ceiling: f64) -> f64 {
    let floor = if floor.is_finite() {
        floor.max(0.25)
    } else {
        0.25
    };
    let ceiling = if ceiling.is_finite() {
        ceiling.max(floor)
    } else {
        floor
    };
    if target.is_finite() {
        target.clamp(floor, ceiling)
    } else {
        floor
    }
}

fn operating_cost_for(
    utility: &Utility,
    market: &Market,
    served_mwh: f64,
    cost_multiplier: f64,
) -> f64 {
    let maintenance_load = if utility.reliability < 0.72 {
        1.14
    } else {
        1.0
    };
    (served_mwh * market.variable_cost_per_mwh * maintenance_load
        + 850.0
        + utility.customers * 2.65
        + utility.asset_base * 0.0105)
        * cost_multiplier
}

fn service_reputation_gain(utility: &Utility, market: &Market, unmet_demand_ratio: f64) -> f64 {
    if unmet_demand_ratio > 0.01
        || utility.reliability < SERVICE_REPUTATION_RELIABILITY_THRESHOLD
        || utility.utilization(market) > SERVICE_REPUTATION_UTILIZATION_LIMIT
    {
        return 0.0;
    }

    let reliability_quality =
        ((utility.reliability - SERVICE_REPUTATION_RELIABILITY_THRESHOLD) / 0.12).clamp(0.0, 1.0);
    let utilization_cushion = ((SERVICE_REPUTATION_UTILIZATION_LIMIT
        - utility.utilization(market))
        / SERVICE_REPUTATION_UTILIZATION_LIMIT)
        .clamp(0.0, 1.0);
    let reputation_headroom = ((88.0 - utility.reputation) / 33.0).clamp(0.10, 1.0);

    (0.20 + reliability_quality * 0.35 + utilization_cushion * 0.15).min(0.70) * reputation_headroom
}

pub(super) fn churn_for_market_with_floor(
    utility: &Utility,
    average_rate: f64,
    market: &Market,
    natural_churn_floor: f64,
) -> f64 {
    utility.customers
        * churn_rate_for_market_with_floor(utility, average_rate, market, natural_churn_floor)
}

#[cfg(test)]
pub(super) fn churn_rate_for(utility: &Utility, average_rate: f64) -> f64 {
    base_churn_rate_for(utility, average_rate, 0.0, 0.004)
}

#[cfg(test)]
pub(super) fn churn_rate_for_market(utility: &Utility, average_rate: f64, market: &Market) -> f64 {
    churn_rate_for_market_with_floor(utility, average_rate, market, 0.004)
}

fn churn_rate_for_market_with_floor(
    utility: &Utility,
    average_rate: f64,
    market: &Market,
    natural_churn_floor: f64,
) -> f64 {
    base_churn_rate_for(
        utility,
        average_rate,
        high_rate_excess(utility, market),
        natural_churn_floor,
    )
}

fn base_churn_rate_for(
    utility: &Utility,
    average_rate: f64,
    rate_excess: f64,
    natural_churn_floor: f64,
) -> f64 {
    let rate_gap = utility.rate_cents - average_rate;
    let rate_pressure = rate_gap.max(0.0) * 0.006;
    let rate_retention = (-rate_gap).max(0.0) * 0.0031;
    let public_rate_pressure = rate_excess * 0.012 + rate_excess * rate_excess * 0.004;
    let outage_pressure = (0.78 - utility.reliability).max(0.0) * 0.055;
    let reliability_retention = (utility.reliability - 0.84).max(0.0) * 0.026;
    let reputation_pressure = (48.0 - utility.reputation).max(0.0) * 0.0008;
    let reputation_retention = (utility.reputation - 62.0).max(0.0) * 0.00012;
    let churn_rate =
        0.010 + rate_pressure + public_rate_pressure + outage_pressure + reputation_pressure;
    let churn_cap = (0.075 + rate_excess * 0.035).clamp(0.075, 0.50);
    let churn_floor = natural_churn_floor.clamp(0.0024, 0.0065);
    (churn_rate - rate_retention - reliability_retention - reputation_retention)
        .clamp(churn_floor, churn_cap)
}

pub fn public_rate_tolerance(market: &Market) -> f64 {
    market.standard_rate_cents
        + 2.2
        + market.civic_patience * 1.2
        + market.rate_tolerance_adjustment_cents
}

pub(super) fn high_rate_excess(utility: &Utility, market: &Market) -> f64 {
    (utility.rate_cents - public_rate_tolerance(market)).max(0.0)
}

pub(super) fn weighted_rate(rates: impl Iterator<Item = (f64, f64)>, fallback: f64) -> f64 {
    let mut weighted = 0.0;
    let mut customers = 0.0;
    for (rate, customer_count) in rates {
        weighted += rate * customer_count;
        customers += customer_count;
    }
    if customers <= 0.0 {
        fallback
    } else {
        weighted / customers
    }
}

pub(super) fn weighted_average(left: f64, left_weight: f64, right: f64, right_weight: f64) -> f64 {
    let total = left_weight + right_weight;
    if total <= 0.0 {
        left
    } else {
        (left * left_weight + right * right_weight) / total
    }
}

pub(super) fn positive_amount(amount: f64, label: &str) -> Result<f64, String> {
    if amount.is_finite() && amount > 0.0 {
        Ok(amount)
    } else {
        Err(format!("The {label} amount must be positive."))
    }
}

pub fn money(value: f64) -> String {
    if value.abs() >= 1_000.0 {
        format!("${:.1}k", value / 1_000.0)
    } else {
        format!("${value:.0}")
    }
}

pub fn generation_project_cost(capacity_mwh: f64) -> f64 {
    scaled_project_cost(
        capacity_mwh,
        GENERATION_PROJECT_CAPACITY_MWH,
        GENERATION_PROJECT_COST,
        0.82,
        MIN_GENERATION_PROJECT_MWH,
        MAX_GENERATION_PROJECT_MWH,
    )
}

pub fn distribution_project_cost(customer_capacity: f64) -> f64 {
    scaled_project_cost(
        customer_capacity,
        DISTRIBUTION_PROJECT_CAPACITY,
        DISTRIBUTION_PROJECT_COST,
        0.84,
        MIN_DISTRIBUTION_PROJECT_CUSTOMERS,
        MAX_DISTRIBUTION_PROJECT_CUSTOMERS,
    )
}

pub fn generation_project_duration(capacity_mwh: f64) -> u32 {
    let capacity_mwh = capacity_mwh.clamp(MIN_GENERATION_PROJECT_MWH, MAX_GENERATION_PROJECT_MWH);
    if capacity_mwh <= GENERATION_PROJECT_CAPACITY_MWH * 1.35 {
        2
    } else if capacity_mwh <= GENERATION_PROJECT_CAPACITY_MWH * 2.40 {
        3
    } else {
        4
    }
}

pub fn distribution_project_duration(customer_capacity: f64) -> u32 {
    let customer_capacity = customer_capacity.clamp(
        MIN_DISTRIBUTION_PROJECT_CUSTOMERS,
        MAX_DISTRIBUTION_PROJECT_CUSTOMERS,
    );
    if customer_capacity <= DISTRIBUTION_PROJECT_CAPACITY * 1.50 {
        1
    } else if customer_capacity <= DISTRIBUTION_PROJECT_CAPACITY * 2.75 {
        2
    } else {
        3
    }
}

fn scaled_project_cost(
    requested: f64,
    default_size: f64,
    default_cost: f64,
    scale_exponent: f64,
    min_size: f64,
    max_size: f64,
) -> f64 {
    let requested = requested.clamp(min_size, max_size);
    default_cost * (requested / default_size).powf(scale_exponent)
}
