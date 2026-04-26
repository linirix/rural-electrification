use super::{
    DISTRIBUTION_PROJECT_CAPACITY, DISTRIBUTION_PROJECT_COST, FirmFinances,
    GENERATION_PROJECT_CAPACITY_MWH, GENERATION_PROJECT_COST, MAINTENANCE_REFERENCE_ASSET_BASE,
    MAX_DISTRIBUTION_PROJECT_CUSTOMERS, MAX_GENERATION_PROJECT_MWH,
    MIN_DISTRIBUTION_PROJECT_CUSTOMERS, MIN_GENERATION_PROJECT_MWH, MacroEnvironment, Market,
    Utility,
};

const SERVICE_REPUTATION_RELIABILITY_THRESHOLD: f64 = 0.84;
const SERVICE_REPUTATION_UTILIZATION_LIMIT: f64 = 0.82;

pub(super) fn maintenance_reliability_gain(utility: &Utility, spend: f64) -> f64 {
    (spend / 22_000.0)
        * (1.05_f64 - utility.reliability).max(0.05)
        * maintenance_asset_scale(utility)
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
    let maintenance_load = if utility.reliability < 0.72 {
        1.14
    } else {
        1.0
    };
    let operating_cost = (served_mwh * market.variable_cost_per_mwh * maintenance_load
        + 850.0
        + utility.customers * 2.65
        + utility.asset_base * 0.0105)
        * cost_multiplier;
    let interest = utility.debt * macro_state.annual_interest_rate_for(utility) / 4.0;
    let profit = revenue - operating_cost - interest;

    utility.cash += profit;
    if utility.cash < 0.0 {
        utility.debt += -utility.cash;
        utility.cash = 0.0;
        utility.reputation = (utility.reputation - 1.8).clamp(0.0, 100.0);
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
        utility.reputation = (utility.reputation + service_reputation_gain).clamp(0.0, 100.0);
        if is_player {
            events.push(format!(
                "Reliable service with manageable utilization lifted Metro's reputation by {:.1} points.",
                service_reputation_gain
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

pub(super) fn churn_for_market(utility: &Utility, average_rate: f64, market: &Market) -> f64 {
    utility.customers * churn_rate_for_market(utility, average_rate, market)
}

#[cfg(test)]
pub(super) fn churn_rate_for(utility: &Utility, average_rate: f64) -> f64 {
    base_churn_rate_for(utility, average_rate, 0.0)
}

pub(super) fn churn_rate_for_market(utility: &Utility, average_rate: f64, market: &Market) -> f64 {
    base_churn_rate_for(utility, average_rate, high_rate_excess(utility, market))
}

fn base_churn_rate_for(utility: &Utility, average_rate: f64, rate_excess: f64) -> f64 {
    let rate_gap = utility.rate_cents - average_rate;
    let rate_pressure = rate_gap.max(0.0) * 0.006;
    let rate_retention = (-rate_gap).max(0.0) * 0.0025;
    let public_rate_pressure = rate_excess * 0.012 + rate_excess * rate_excess * 0.004;
    let outage_pressure = (0.78 - utility.reliability).max(0.0) * 0.055;
    let reliability_retention = (utility.reliability - 0.84).max(0.0) * 0.020;
    let reputation_pressure = (48.0 - utility.reputation).max(0.0) * 0.0008;
    let reputation_retention = (utility.reputation - 62.0).max(0.0) * 0.00012;
    let churn_rate =
        0.010 + rate_pressure + public_rate_pressure + outage_pressure + reputation_pressure;
    let churn_cap = (0.075 + rate_excess * 0.035).clamp(0.075, 0.50);
    (churn_rate - rate_retention - reliability_retention - reputation_retention)
        .clamp(0.004, churn_cap)
}

pub fn public_rate_tolerance(market: &Market) -> f64 {
    market.standard_rate_cents + 2.2 + market.civic_patience * 1.2
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
