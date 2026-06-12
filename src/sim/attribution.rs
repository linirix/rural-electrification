use super::{ActiveShock, FirmFinances, shock_kind_label};

pub(super) struct PointInTime {
    pub(super) customers: f64,
    pub(super) market_share: f64,
    pub(super) total_connected: f64,
    pub(super) reliability: f64,
    pub(super) headroom: f64,
}

pub(super) struct AttributionContext<'a> {
    pub(super) starting: PointInTime,
    pub(super) ending: PointInTime,
    pub(super) lost_customer_rate: f64,
    pub(super) rate_gap_to_rivals: f64,
    pub(super) finances: &'a FirmFinances,
    pub(super) active_shocks: &'a [ActiveShock],
}

/// Salience ranks for attribution lines. Routine lines keep their emission
/// order (share, churn, margin, capacity, reliability, shocks) through the
/// stable sort; crisis lines float to the front so the report leads with the
/// cause and the dashboard's single-driver slot inherits it.
const ROUTINE_ATTRIBUTION: u8 = 10;
const CRISIS_UNSERVED_DEMAND: u8 = 0;
const CRISIS_RELIABILITY_SLIP: u8 = 1;
const SEVERE_UNSERVED_DEMAND_RATIO: f64 = 0.08;
const SEVERE_RELIABILITY_SLIP: f64 = 0.02;

pub(super) fn quarter_attributions(context: AttributionContext<'_>) -> Vec<String> {
    let mut lines: Vec<(u8, String)> = Vec::new();
    let net_customers = context.ending.customers - context.starting.customers;
    let market_customer_change = context.ending.total_connected - context.starting.total_connected;
    let share_change = context.ending.market_share - context.starting.market_share;

    if share_change.abs() >= 0.001 {
        let direction = if share_change > 0.0 { "rose" } else { "fell" };
        lines.push((ROUTINE_ATTRIBUTION, format!(
            "Share {direction} {} as Metro added {} net customers while connected accounts in the market changed by {}.",
            percentage_points(share_change.abs()),
            signed_whole(net_customers),
            signed_whole(market_customer_change)
        )));
    } else {
        lines.push((ROUTINE_ATTRIBUTION, format!(
            "Share held roughly flat while Metro added {} net customers and the total market changed by {}.",
            signed_whole(net_customers),
            signed_whole(market_customer_change)
        )));
    }

    let churn_reason = if context.rate_gap_to_rivals >= 0.4 && context.ending.reliability < 0.74 {
        "premium pricing and service pressure"
    } else if context.rate_gap_to_rivals >= 0.4 {
        "premium pricing"
    } else if context.ending.reliability < 0.74 {
        "service pressure"
    } else if context.rate_gap_to_rivals <= -0.4 {
        "normal switching despite a price advantage"
    } else {
        "normal switching in a competitive market"
    };
    lines.push((
        ROUTINE_ATTRIBUTION,
        format!(
            "Churn was {:.2}% with your rate {}c versus rivals; main pressure was {churn_reason}.",
            context.lost_customer_rate * 100.0,
            signed_decimal(context.rate_gap_to_rivals)
        ),
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
    lines.push((ROUTINE_ATTRIBUTION, format!(
        "Profit margin was {:.1}%; operating costs used {:.0}% of revenue and interest used {:.0}%.",
        margin * 100.0,
        cost_load * 100.0,
        interest_load * 100.0
    )));

    let headroom_change = context.ending.headroom - context.starting.headroom;
    if context.ending.headroom < 60.0 {
        lines.push((ROUTINE_ATTRIBUTION, format!(
            "Capacity is tight: headroom moved from {:.0} to {:.0}, so growth spending may outrun the network.",
            context.starting.headroom.max(0.0),
            context.ending.headroom.max(0.0)
        )));
    } else if headroom_change.abs() >= 35.0 {
        let direction = if headroom_change > 0.0 {
            "opened"
        } else {
            "narrowed"
        };
        lines.push((
            ROUTINE_ATTRIBUTION,
            format!(
                "Capacity headroom {direction} by {:.0} customers, ending at {:.0}.",
                headroom_change.abs(),
                context.ending.headroom.max(0.0)
            ),
        ));
    }

    let overloaded = context.finances.unmet_demand_ratio > 0.02;
    let reliability_change = context.ending.reliability - context.starting.reliability;
    if reliability_change <= -0.006 {
        let severity = if reliability_change <= -SEVERE_RELIABILITY_SLIP {
            CRISIS_RELIABILITY_SLIP
        } else {
            ROUTINE_ATTRIBUTION
        };
        // The prescription depends on the cause: maintenance counters wear, but
        // an overloaded fleet only recovers through added generation capacity.
        let text = if overloaded {
            format!(
                "Reliability slipped {} under unserved-demand strain; only added generation capacity relieves the overload.",
                percentage_points(reliability_change.abs())
            )
        } else {
            format!(
                "Reliability slipped {} from operating wear and load; maintenance spending is the main way to counter it.",
                percentage_points(reliability_change.abs())
            )
        };
        lines.push((severity, text));
    } else if reliability_change >= 0.006 {
        lines.push((
            ROUTINE_ATTRIBUTION,
            format!(
                "Reliability improved {} after service work and completed capacity relief.",
                percentage_points(reliability_change)
            ),
        ));
    }

    if overloaded {
        let severity = if context.finances.unmet_demand_ratio >= SEVERE_UNSERVED_DEMAND_RATIO {
            CRISIS_UNSERVED_DEMAND
        } else {
            ROUTINE_ATTRIBUTION
        };
        lines.push((
            severity,
            format!(
                "Unserved demand reached {:.0}%, hurting reliability and reputation.",
                context.finances.unmet_demand_ratio * 100.0
            ),
        ));
    }

    if !context.active_shocks.is_empty() {
        let shocks = context
            .active_shocks
            .iter()
            .map(|shock| shock_kind_label(&shock.kind))
            .collect::<Vec<_>>()
            .join(", ");
        lines.push((
            ROUTINE_ATTRIBUTION,
            format!("Active macro shock affecting the quarter: {shocks}."),
        ));
    }

    lines.sort_by_key(|(severity, _)| *severity);
    let mut lines: Vec<String> = lines.into_iter().map(|(_, line)| line).collect();
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
