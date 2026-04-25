use super::{ActiveShock, FirmFinances, shock_kind_label};

pub(super) struct AttributionContext<'a> {
    pub(super) starting_customers: f64,
    pub(super) ending_customers: f64,
    pub(super) starting_total_connected: f64,
    pub(super) ending_total_connected: f64,
    pub(super) starting_market_share: f64,
    pub(super) ending_market_share: f64,
    pub(super) starting_reliability: f64,
    pub(super) ending_reliability: f64,
    pub(super) starting_headroom: f64,
    pub(super) ending_headroom: f64,
    pub(super) lost_customer_rate: f64,
    pub(super) rate_gap_to_rivals: f64,
    pub(super) finances: &'a FirmFinances,
    pub(super) active_shocks: &'a [ActiveShock],
}

pub(super) fn quarter_attributions(context: AttributionContext<'_>) -> Vec<String> {
    let mut lines = Vec::new();
    let net_customers = context.ending_customers - context.starting_customers;
    let market_customer_change = context.ending_total_connected - context.starting_total_connected;
    let share_change = context.ending_market_share - context.starting_market_share;

    if share_change.abs() >= 0.001 {
        let direction = if share_change > 0.0 { "rose" } else { "fell" };
        lines.push(format!(
            "Share {direction} {} as Metro added {} net customers while connected accounts in the market changed by {}.",
            percentage_points(share_change.abs()),
            signed_whole(net_customers),
            signed_whole(market_customer_change)
        ));
    } else {
        lines.push(format!(
            "Share held roughly flat while Metro added {} net customers and the total market changed by {}.",
            signed_whole(net_customers),
            signed_whole(market_customer_change)
        ));
    }

    let churn_reason = if context.rate_gap_to_rivals >= 0.4 && context.ending_reliability < 0.74 {
        "premium pricing and service pressure"
    } else if context.rate_gap_to_rivals >= 0.4 {
        "premium pricing"
    } else if context.ending_reliability < 0.74 {
        "service pressure"
    } else if context.rate_gap_to_rivals <= -0.4 {
        "normal switching despite a price advantage"
    } else {
        "normal switching in a competitive market"
    };
    lines.push(format!(
        "Churn was {:.2}% with your rate {}c versus rivals; main pressure was {churn_reason}.",
        context.lost_customer_rate * 100.0,
        signed_decimal(context.rate_gap_to_rivals)
    ));

    let margin = if context.finances.revenue > 0.0 {
        context.finances.profit / context.finances.revenue
    } else {
        0.0
    };
    let cost_load = if context.finances.revenue > 0.0 {
        context.finances.operating_cost / context.finances.revenue
    } else {
        0.0
    };
    let interest_load = if context.finances.revenue > 0.0 {
        context.finances.interest / context.finances.revenue
    } else {
        0.0
    };
    lines.push(format!(
        "Profit margin was {:.1}%; operating costs used {:.0}% of revenue and interest used {:.0}%.",
        margin * 100.0,
        cost_load * 100.0,
        interest_load * 100.0
    ));

    let headroom_change = context.ending_headroom - context.starting_headroom;
    if context.ending_headroom < 60.0 {
        lines.push(format!(
            "Capacity is tight: headroom moved from {:.0} to {:.0}, so growth spending may outrun the network.",
            context.starting_headroom.max(0.0),
            context.ending_headroom.max(0.0)
        ));
    } else if headroom_change.abs() >= 35.0 {
        let direction = if headroom_change > 0.0 {
            "opened"
        } else {
            "narrowed"
        };
        lines.push(format!(
            "Capacity headroom {direction} by {:.0} customers, ending at {:.0}.",
            headroom_change.abs(),
            context.ending_headroom.max(0.0)
        ));
    }

    let reliability_change = context.ending_reliability - context.starting_reliability;
    if reliability_change <= -0.006 {
        lines.push(format!(
            "Reliability slipped {} from operating wear and load; maintenance is the direct offset.",
            percentage_points(reliability_change.abs())
        ));
    } else if reliability_change >= 0.006 {
        lines.push(format!(
            "Reliability improved {} after service work and completed capacity relief.",
            percentage_points(reliability_change)
        ));
    }

    if context.finances.unmet_demand_ratio > 0.02 {
        lines.push(format!(
            "Unserved demand reached {:.0}%, hurting reliability and reputation.",
            context.finances.unmet_demand_ratio * 100.0
        ));
    }

    if !context.active_shocks.is_empty() {
        let shocks = context
            .active_shocks
            .iter()
            .map(|shock| shock_kind_label(&shock.kind))
            .collect::<Vec<_>>()
            .join(", ");
        lines.push(format!(
            "Active macro shock affecting the quarter: {shocks}."
        ));
    }

    lines.truncate(6);
    lines
}

fn percentage_points(value: f64) -> String {
    format!("{:.1} pts", value * 100.0)
}

fn signed_whole(value: f64) -> String {
    format!("{:+.0}", value)
}

fn signed_decimal(value: f64) -> String {
    format!("{:+.1}", value)
}
