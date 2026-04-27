use super::command::{parse_decision, parse_money, parse_rate_number};
use super::render::*;
use super::rivals::public_acquisition_range;
use super::*;

pub(super) struct PreviewSegment {
    command: String,
    tokens: Vec<String>,
}

pub(super) fn preview_command(game: &Game, parts: &[&str]) -> CommandResult {
    if parts.len() <= 1 {
        return CommandResult::Preview(vec![
            styled(BOLD_CYAN, "Preview only; no action taken."),
            "Use preview build gen 400, preview debt 20000, preview diligence 2 buy 2, or preview maint 6000.".to_string(),
        ]);
    }

    let command_parts = &parts[1..];
    let command = command_parts.join(" ");
    let segments = match preview_segments(command_parts) {
        Ok(segments) => segments,
        Err(message) => {
            return CommandResult::Preview(vec![
                styled(BOLD_RED, "Cannot preview command."),
                message,
                "Try preview build gen 400, preview debt 20000, or preview diligence 1 buy 1."
                    .to_string(),
            ]);
        }
    };

    CommandResult::Preview(preview_lines(game, &segments, &command))
}

pub(super) fn preview_lines(
    game: &Game,
    segments: &[PreviewSegment],
    command: &str,
) -> Vec<String> {
    let mut lines = vec![
        styled(BOLD_CYAN, "Preview only; no action taken."),
        format!("{} {}", muted("Command:"), styled(BOLD, command)),
    ];

    let mut simulated = game.clone();
    let multi_step = segments.len() > 1;
    let mut failed = false;

    for (index, segment) in segments.iter().enumerate() {
        if multi_step {
            lines.push(format!(
                "{} {}",
                muted(format!("Step {}:", index + 1)),
                styled(BOLD, &segment.command)
            ));
        }

        let token_refs = segment
            .tokens
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>();
        let decision = match parse_decision(&simulated, &token_refs) {
            Ok(decision) => decision,
            Err(message) => {
                push_labeled_wrapped(&mut lines, "Cannot preview:", BOLD_RED, &message);
                failed = true;
                break;
            }
        };

        lines.extend(decision_preview_notes(&simulated, &decision));
        match simulated.apply_decision(decision) {
            Ok(message) => push_labeled_wrapped(&mut lines, "Would succeed:", BOLD_GREEN, &message),
            Err(error) => {
                push_labeled_wrapped(&mut lines, "Would fail:", BOLD_RED, &error);
                failed = true;
                break;
            }
        }
    }

    let effect_lines = immediate_effect_lines(game, &simulated);
    if !failed || !only_no_immediate_effects(&effect_lines) {
        lines.push(styled(
            BOLD_CYAN,
            if failed {
                "Effects Through Successful Steps"
            } else {
                "Immediate Effects"
            },
        ));
        lines.extend(effect_lines);
        push_labeled_wrapped(
            &mut lines,
            "Note:",
            DIM,
            "Quarter-end demand, churn, rival response, macro movement, and project completion are not simulated.",
        );
    }

    lines
}

pub(super) fn preview_segments(parts: &[&str]) -> Result<Vec<PreviewSegment>, String> {
    let mut segments = Vec::new();
    let mut index = 0;

    while index < parts.len() {
        let length = preview_segment_len(&parts[index..])?;
        let tokens = parts[index..index + length]
            .iter()
            .map(|token| (*token).to_string())
            .collect::<Vec<_>>();
        segments.push(PreviewSegment {
            command: tokens.join(" "),
            tokens,
        });
        index += length;
    }

    Ok(segments)
}

pub(super) fn preview_segment_len(parts: &[&str]) -> Result<usize, String> {
    let Some(first) = parts.first().copied() else {
        return Err("Use 'preview <command>' or enter a command.".to_string());
    };

    match first {
        "build" => {
            let mut length = 1;
            if parts.get(1).is_some() {
                length = 2;
            }
            if parts
                .get(2)
                .is_some_and(|value| !is_preview_action_start(value))
            {
                length = 3;
            }
            Ok(length)
        }
        "marketing" | "market" | "advertise" | "issue" | "buyback" | "repurchase" | "expand"
        | "adjacent" | "territory" | "maintenance" | "maint" | "maintain" | "reliability" => {
            Ok(1 + optional_command_argument(parts.get(1).copied()))
        }
        "repay" | "paydown" => Ok(1 + optional_command_argument(parts.get(1).copied())),
        "stock" | "equity" => match parts.get(1).copied() {
            Some("issue" | "sell" | "buyback" | "repurchase" | "buy") => {
                Ok(2 + optional_command_argument(parts.get(2).copied()))
            }
            Some(value) if parse_money(value).is_some() => Ok(2),
            Some(value) if is_preview_action_start(value) => Ok(1),
            Some(_) => Ok(2),
            None => Ok(1),
        },
        "debt" => match parts.get(1).copied() {
            Some("repay" | "pay" | "down") => {
                Ok(2 + optional_command_argument(parts.get(2).copied()))
            }
            Some(value) if is_money_or_max_argument(value) => Ok(2),
            Some(value) if is_preview_action_start(value) => Ok(1),
            Some(_) => Ok(2),
            None => Ok(1),
        },
        "borrow" | "loan" => Ok(1 + optional_command_argument(parts.get(1).copied())),
        "buy" | "acquire" | "diligence" | "dilig" | "inspect" => {
            Ok(if parts.get(1).is_some() { 2 } else { 1 })
        }
        "rate" => match parts.get(1).copied() {
            Some("up" | "down" | "+" | "-") => {
                Ok(2 + optional_rate_argument(parts.get(2).copied()))
            }
            Some(value) if is_rate_argument(value) => Ok(2),
            Some(value) if is_preview_action_start(value) => Ok(1),
            Some(_) => Ok(2),
            None => Ok(1),
        },
        _ => Err(format!("Unknown command '{first}'. Type 'help'.")),
    }
}

pub(super) fn optional_command_argument(value: Option<&str>) -> usize {
    value
        .filter(|value| !is_preview_action_start(value))
        .map_or(0, |_| 1)
}

pub(super) fn is_money_or_max_argument(value: &str) -> bool {
    value == "max" || parse_money(value).is_some()
}

pub(super) fn optional_rate_argument(value: Option<&str>) -> usize {
    value
        .filter(|value| is_rate_argument(value))
        .map_or(0, |_| 1)
}

pub(super) fn is_rate_argument(value: &str) -> bool {
    if (value.starts_with('+') || value.starts_with('-')) && value.len() > 1 {
        parse_rate_number(&value[1..]).is_ok()
    } else {
        parse_rate_number(value).is_ok()
    }
}

pub(super) fn is_preview_action_start(value: &str) -> bool {
    matches!(
        value,
        "build"
            | "marketing"
            | "market"
            | "advertise"
            | "issue"
            | "stock"
            | "equity"
            | "buyback"
            | "repurchase"
            | "debt"
            | "borrow"
            | "loan"
            | "repay"
            | "paydown"
            | "buy"
            | "acquire"
            | "diligence"
            | "dilig"
            | "inspect"
            | "expand"
            | "adjacent"
            | "territory"
            | "rate"
            | "maintenance"
            | "maint"
            | "maintain"
            | "reliability"
    )
}

pub(super) fn decision_preview_notes(game: &Game, decision: &Decision) -> Vec<String> {
    match decision {
        Decision::BuildGeneration { capacity_mwh } => {
            let size = capacity_mwh.clamp(MIN_GENERATION_PROJECT_MWH, MAX_GENERATION_PROJECT_MWH);
            let cost = generation_project_cost(size);
            let reliability_after = generation_reliability_after_new_capacity(
                game.player.reliability,
                game.player.generation_capacity_mwh,
                size,
            );
            vec![format!(
                "{} generation: {} for {:.0} MWh/q, {}q, {} per MWh/q; reliability {:.0}% -> {:.0}% when online.",
                muted("Quote:"),
                styled(BOLD_YELLOW, money(cost)),
                size,
                generation_project_duration(size),
                money(cost / size),
                game.player.reliability * 100.0,
                reliability_after * 100.0
            )]
        }
        Decision::BuildDistribution { customer_capacity } => {
            let size = customer_capacity.clamp(
                MIN_DISTRIBUTION_PROJECT_CUSTOMERS,
                MAX_DISTRIBUTION_PROJECT_CUSTOMERS,
            );
            let cost = distribution_project_cost(size);
            vec![format!(
                "{} distribution: {} for {:.0} customers, {}q, {} per customer.",
                muted("Quote:"),
                styled(BOLD_YELLOW, money(cost)),
                size,
                distribution_project_duration(size),
                money(cost / size)
            )]
        }
        Decision::EnterAdjacentMarket => {
            let cost = game.adjacent_expansion_cost();
            let duration = game.adjacent_expansion_duration();
            let mut lines = vec![format!(
                "{} adjacent territory: {} for {:.0} addressable customers, {:.0} launch customers, {}q.",
                muted("Quote:"),
                styled(BOLD_YELLOW, money(cost)),
                ADJACENT_EXPANSION_ADDRESSABLE_CUSTOMERS,
                ADJACENT_EXPANSION_INITIAL_CUSTOMERS,
                duration
            )];
            if let Some(blocker) = game.adjacent_expansion_blocker() {
                lines.push(format!("{} {blocker}", styled(BOLD_RED, "Gate:")));
            } else if game.player.cash < cost {
                lines.push(format!(
                    "{} raise {} more before committing, or preview a financing chain first.",
                    styled(BOLD_YELLOW, "Financing:"),
                    money(cost - game.player.cash)
                ));
            }
            lines.push(format!(
                "{} opening the territory creates a temporary regional integration burden; marketing and maintenance work it down.",
                muted("Follow-up:")
            ));
            lines
        }
        Decision::Marketing { spend } => vec![format!(
            "{} {} spend; clamped to {}-{} and mostly affects next quarter's customer capture.",
            muted("Quote:"),
            styled(BOLD_YELLOW, money(*spend)),
            money(1_000.0),
            money(25_000.0)
        )],
        Decision::IssueStock { amount } => vec![format!(
            "{} issue {}; post-money value reflects discounted pre-money plus net proceeds.",
            muted("Quote:"),
            styled(BOLD_YELLOW, money(*amount))
        )],
        Decision::BuyBackStock { amount } => vec![format!(
            "{} repurchase request {}; post-buyback value reflects transaction value less cash spent.",
            muted("Quote:"),
            styled(BOLD_YELLOW, money(*amount))
        )],
        Decision::Borrow { amount } => vec![format!(
            "{} borrow {}; current room {} at {:.1}% floating.",
            muted("Quote:"),
            styled(BOLD_YELLOW, money(*amount)),
            styled(
                borrowing_room_tone(game.borrowing_room()),
                money(game.borrowing_room())
            ),
            game.player_annual_interest_rate() * 100.0
        )],
        Decision::RepayDebt { amount } => vec![format!(
            "{} repay {}; limited by cash {} and debt {}.",
            muted("Quote:"),
            styled(BOLD_YELLOW, money(*amount)),
            money(game.player.cash),
            money(game.player.debt)
        )],
        Decision::Diligence { competitor_index } => {
            if let Some(competitor) = game.competitors.get(*competitor_index) {
                let cost = game.diligence_cost(*competitor_index).unwrap_or(0.0);
                let mut lines = vec![format!(
                    "{} inspect {} for {}; exact deal terms for 3q, but the target reacts before the quote is frozen.",
                    muted("Quote:"),
                    styled(BOLD, shorten_plain(&competitor.name, 16)),
                    styled(BOLD_YELLOW, money(cost)),
                )];
                if let Some(terms) = game.acquisition_terms(*competitor_index) {
                    lines.push(format!(
                        "{} buy at {}; net cost {}, assumes debt {}, post debt/assets {:.0}%; +{:.0} cust.",
                        muted("Will lock:"),
                        styled(BOLD_YELLOW, money(terms.price)),
                        styled(cash_tone(terms.post_cash), money(terms.net_cash_cost)),
                        styled(YELLOW, money(terms.assumed_debt)),
                        terms.post_debt_to_assets * 100.0,
                        terms.acquired_customers
                    ));
                    lines.push(acquisition_service_line(game, competitor, &terms));
                }
                lines
            } else {
                vec![format!(
                    "{} No rival has that number.",
                    styled(BOLD_RED, "Quote:")
                )]
            }
        }
        Decision::Acquire { competitor_index } => {
            if let Some(competitor) = game.competitors.get(*competitor_index) {
                if game.has_diligence(*competitor_index) {
                    let terms = game
                        .acquisition_terms(*competitor_index)
                        .expect("competitor exists for acquisition quote");
                    let mut lines = vec![format!(
                        "{} buy {} for {}; net cost {}, assumes debt {}, adds {:.0} customers.",
                        muted("Quote:"),
                        styled(BOLD, shorten_plain(&competitor.name, 16)),
                        styled(BOLD_YELLOW, money(terms.price)),
                        styled(cash_tone(terms.post_cash), money(terms.net_cash_cost)),
                        styled(YELLOW, money(terms.assumed_debt)),
                        terms.acquired_customers
                    )];
                    lines.push(acquisition_risk_line(
                        game,
                        *competitor_index,
                        competitor,
                        &terms,
                    ));
                    lines.push(acquisition_service_line(game, competitor, &terms));
                    lines
                } else {
                    let cost = game.diligence_cost(*competitor_index).unwrap_or(0.0);
                    let (low, high) = public_acquisition_range(game, *competitor_index);
                    let mut lines = vec![format!(
                        "{} public estimate {}-{}; buy now accepts close risk, or {} diligence reveals exact terms for {}.",
                        muted("Quote:"),
                        styled(YELLOW, money(low)),
                        styled(YELLOW, money(high)),
                        styled(BOLD_YELLOW, money(cost)),
                        styled(BOLD, shorten_plain(&competitor.name, 16))
                    )];
                    lines.push(public_acquisition_underwriting_line(
                        game,
                        *competitor_index,
                    ));
                    lines
                }
            } else {
                vec![format!(
                    "{} No rival has that number.",
                    styled(BOLD_RED, "Quote:")
                )]
            }
        }
        Decision::AdjustRate { delta_cents } => {
            let target = game.player.rate_cents + delta_cents;
            if target <= 0.0 {
                return vec![format!(
                    "{} rates must stay above 0.0c/kWh.",
                    styled(BOLD_RED, "Quote:")
                )];
            }
            let tolerance = public_rate_tolerance(&game.market);
            let public_ceiling = tolerance + MAX_PUBLIC_RATE_PREMIUM_CENTS;
            if *delta_cents > 0.0 && target > public_ceiling {
                return vec![format!(
                    "{} {:.1}c would be rejected; public ceiling is {:.1}c/kWh.",
                    styled(BOLD_RED, "Quote:"),
                    target,
                    public_ceiling
                )];
            }
            let mut lines = vec![format!(
                "{} rate would move from {:.1}c to {:.1}c/kWh.",
                muted("Quote:"),
                game.player.rate_cents,
                target
            )];
            if target > tolerance {
                lines.push(format!(
                    "{} above the {:.1}c public tolerance; expect heavier churn, reputation pressure, startup risk, and rate-freeze risk.",
                    styled(BOLD_YELLOW, "Pressure:"),
                    tolerance
                ));
            }
            let break_even = game.player_break_even_rate_cents();
            let support_ratio = if break_even <= 0.0 {
                f64::INFINITY
            } else {
                target / break_even
            };
            if support_ratio < REVIEW_MIN_RATE_SUPPORT_RATIO {
                lines.push(format!(
                    "{} target is only {:.0}% of {:.1}c break-even; review sustainability and financing pressure will suffer.",
                    styled(BOLD_RED, "Sustainability:"),
                    support_ratio * 100.0,
                    break_even
                ));
            } else if support_ratio < 1.0 {
                lines.push(format!(
                    "{} target is below {:.1}c break-even; margins will be thin.",
                    styled(BOLD_YELLOW, "Sustainability:"),
                    break_even
                ));
            }
            lines
        }
        Decision::Maintenance { spend } => {
            if game.player.reliability >= 0.9795 {
                vec![format!(
                    "{} reliability is already at 98%; maintenance would not be accepted.",
                    styled(BOLD_RED, "Quote:")
                )]
            } else {
                vec![format!(
                    "{} {} maintenance; immediate reliability gain with scale diminishing returns.",
                    muted("Quote:"),
                    styled(BOLD_YELLOW, money(*spend))
                )]
            }
        }
    }
}

fn acquisition_risk_line(
    game: &Game,
    competitor_index: usize,
    competitor: &Utility,
    terms: &AcquisitionTerms,
) -> String {
    let deal_share =
        terms.acquired_customers / (game.player.customers + terms.acquired_customers).max(1.0);
    let service_drag = (0.82 - competitor.reliability).max(0.0);
    let leverage_drag = (terms.post_debt_to_assets - 0.72).max(0.0);
    let risk_score = deal_share * 1.10 + service_drag * 1.40 + leverage_drag * 0.80;
    let stress_score = game
        .acquisition_stress_score(competitor_index)
        .unwrap_or(0.0);
    let stress_label = Game::acquisition_stress_label(stress_score);
    let (tone, label) = if risk_score >= 0.42 {
        (BOLD_RED, "high")
    } else if risk_score >= 0.24 {
        (BOLD_YELLOW, "medium")
    } else {
        (BOLD_GREEN, "low")
    };

    format!(
        "{} {} integration risk; underwriting {}, deal size {:.0}% of post-close customers, post debt/assets {:.0}%, target reliability {:.0}%.",
        styled(tone, "Risk:"),
        styled(tone, label),
        styled(underwriting_tone(stress_score), stress_label),
        deal_share * 100.0,
        terms.post_debt_to_assets * 100.0,
        competitor.reliability * 100.0
    )
}

fn acquisition_service_line(game: &Game, competitor: &Utility, terms: &AcquisitionTerms) -> String {
    let projected_reliability = acquisition_projected_reliability(game, competitor, terms);
    let burden = acquisition_projected_integration_burden(game, terms);
    let tone = acquisition_service_tone(projected_reliability);

    format!(
        "{} post-close reliability about {}; service {} versus review standard; integration burden {}.",
        styled(tone, "Service:"),
        styled(tone, format!("{:.0}%", projected_reliability * 100.0)),
        styled(
            tone,
            acquisition_service_cushion_text(projected_reliability)
        ),
        styled(
            acquisition_integration_burden_tone(burden),
            acquisition_integration_burden_label(burden)
        )
    )
}

fn public_acquisition_underwriting_line(game: &Game, competitor_index: usize) -> String {
    let stress_score = game
        .acquisition_stress_score(competitor_index)
        .unwrap_or(0.0);
    let label = Game::acquisition_stress_label(stress_score);
    format!(
        "{} public file leaves lender posture {}; watch service quality, liquidity buffer, and leverage before closing without diligence.",
        styled(underwriting_tone(stress_score), "Underwriting:"),
        styled(underwriting_tone(stress_score), label)
    )
}

fn underwriting_tone(score: f64) -> &'static str {
    match Game::acquisition_stress_label(score) {
        "distressed" => BOLD_RED,
        "strained" => BOLD_YELLOW,
        "guarded" => YELLOW,
        _ => GREEN,
    }
}

pub(super) fn immediate_effect_lines(before: &Game, after: &Game) -> Vec<String> {
    let mut lines = Vec::new();
    push_money_delta(&mut lines, "Cash", before.player.cash, after.player.cash);
    push_money_delta(&mut lines, "Debt", before.player.debt, after.player.debt);
    push_percent_delta(
        &mut lines,
        "Debt/assets",
        before.player.debt_to_assets(),
        after.player.debt_to_assets(),
    );
    push_money_delta(
        &mut lines,
        "Interest/q",
        quarterly_interest_payment(before),
        quarterly_interest_payment(after),
    );
    push_rate_delta(
        &mut lines,
        "Rate",
        before.player.rate_cents,
        after.player.rate_cents,
    );
    push_percent_delta(
        &mut lines,
        "Reliability",
        before.player.reliability,
        after.player.reliability,
    );
    push_number_delta(
        &mut lines,
        "Reputation",
        before.player.reputation,
        after.player.reputation,
    );
    push_number_delta(
        &mut lines,
        "Customers",
        before.player.customers,
        after.player.customers,
    );
    push_number_delta(
        &mut lines,
        "Customer capacity",
        before.player.customer_capacity(&before.market),
        after.player.customer_capacity(&after.market),
    );
    push_mwh_delta(
        &mut lines,
        "Firm reserve",
        before.player.firm_generation_reserve_mwh(&before.market),
        after.player.firm_generation_reserve_mwh(&after.market),
    );
    push_number_delta(
        &mut lines,
        "Shares",
        before.player.shares,
        after.player.shares,
    );
    push_stock_delta(
        &mut lines,
        "Stock price",
        before.player.stock_price,
        after.player.stock_price,
    );

    let project_delta = after.pending_projects.len() as i32 - before.pending_projects.len() as i32;
    if project_delta != 0 {
        lines.push(format!(
            "{} {} -> {} ({:+})",
            muted("Projects"),
            before.pending_projects.len(),
            after.pending_projects.len(),
            project_delta
        ));
    }

    let rival_delta = after.competitors.len() as i32 - before.competitors.len() as i32;
    if rival_delta != 0 {
        lines.push(format!(
            "{} {} -> {} ({:+})",
            muted("Rivals"),
            before.competitors.len(),
            after.competitors.len(),
            rival_delta
        ));
    }

    if lines.is_empty() {
        lines.push("No immediate dashboard metrics would change.".to_string());
    }
    lines
}

pub(super) fn only_no_immediate_effects(lines: &[String]) -> bool {
    matches!(lines, [line] if line == "No immediate dashboard metrics would change.")
}

pub(super) fn quarterly_interest_payment(game: &Game) -> f64 {
    game.player.debt * game.player_annual_interest_rate() / 4.0
}

pub(super) fn push_money_delta(lines: &mut Vec<String>, label: &str, before: f64, after: f64) {
    if changed(before, after, 0.5) {
        lines.push(format!(
            "{} {} -> {} ({})",
            muted(label),
            money(before),
            money(after),
            signed_money(after - before)
        ));
    }
}

pub(super) fn push_number_delta(lines: &mut Vec<String>, label: &str, before: f64, after: f64) {
    if changed(before, after, 0.05) {
        let delta = after - before;
        lines.push(format!(
            "{} {:.0} -> {:.0} ({:+.0})",
            muted(label),
            before,
            after,
            if delta.abs() < 0.5 { 0.0 } else { delta }
        ));
    }
}

pub(super) fn push_percent_delta(lines: &mut Vec<String>, label: &str, before: f64, after: f64) {
    if changed(before, after, 0.0005) {
        lines.push(format!(
            "{} {:.1}% -> {:.1}% ({:+.1} pts)",
            muted(label),
            before * 100.0,
            after * 100.0,
            (after - before) * 100.0
        ));
    }
}

pub(super) fn push_rate_delta(lines: &mut Vec<String>, label: &str, before: f64, after: f64) {
    if changed(before, after, 0.005) {
        lines.push(format!(
            "{} {:.1}c -> {:.1}c ({:+.1}c)",
            muted(label),
            before,
            after,
            after - before
        ));
    }
}

pub(super) fn push_mwh_delta(lines: &mut Vec<String>, label: &str, before: f64, after: f64) {
    if changed(before, after, 0.5) {
        lines.push(format!(
            "{} {:+.0} MWh -> {:+.0} MWh ({:+.0})",
            muted(label),
            before,
            after,
            after - before
        ));
    }
}

pub(super) fn push_stock_delta(lines: &mut Vec<String>, label: &str, before: f64, after: f64) {
    if changed(before, after, 0.005) {
        lines.push(format!(
            "{} ${:.2} -> ${:.2} ({:+.2})",
            muted(label),
            before,
            after,
            after - before
        ));
    }
}

pub(super) fn signed_money(value: f64) -> String {
    if value >= 0.0 {
        format!("+{}", money(value))
    } else {
        format!("-{}", money(value.abs()))
    }
}

pub(super) fn changed(before: f64, after: f64, epsilon: f64) -> bool {
    (after - before).abs() > epsilon
}

pub(super) fn push_labeled_wrapped(lines: &mut Vec<String>, label: &str, style: &str, text: &str) {
    let label_width = label.chars().count();
    let body_width = content_width().saturating_sub(label_width + 1).max(16);

    for (index, wrapped) in wrap_plain_text(text, body_width).iter().enumerate() {
        if index == 0 {
            lines.push(format!("{} {wrapped}", styled(style, label)));
        } else {
            lines.push(format!("{} {wrapped}", " ".repeat(label_width)));
        }
    }
}
