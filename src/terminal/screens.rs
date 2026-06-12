use super::board::*;
use super::render::*;
use super::rivals::{dashboard_competitor_lines, rival_detail_lines, rival_overview_lines};
use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum FramedScreen {
    Report,
    Rivals,
    Board,
    Help,
}

impl FramedScreen {
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::Report => "Report",
            Self::Rivals => "Rivals",
            Self::Board => "Board",
            Self::Help => "Help",
        }
    }
}

pub(super) fn print_status(game: &Game) {
    print_box(
        &dashboard_title(game),
        &stable_panel_lines(scorecard_lines(game), 5, "board for full milestone path"),
    );
    print_box_pair(
        "Capital",
        &stable_panel_lines(financial_lines(game), 6, "board for financing context"),
        "Operations",
        &stable_panel_lines(operation_lines(game), 6, "report for operating detail"),
    );
    print_box_pair(
        "Last Quarter",
        &stable_panel_lines(last_quarter_lines(game), 6, "report for full detail"),
        "Signals",
        &stable_panel_lines(signal_lines(game), 6, "board for remaining risks"),
    );
    print_box_pair(
        "Competition",
        &stable_panel_lines(dashboard_competitor_lines(game), 4, "rivals for full list"),
        "Milestones",
        &stable_panel_lines(milestone_snapshot_lines(game), 4, "board for full path"),
    );
    print_box_pair(
        "Pipeline",
        &stable_panel_lines(project_lines(game), 6, "board for planning context"),
        "Commands",
        &stable_panel_lines(compact_command_lines(game), 6, "help for all commands"),
    );
}

pub(super) fn print_competitors(game: &Game) {
    print_box("Rival Overview", &rival_overview_lines(game));
    print_box("Rival Detail", &rival_detail_lines(game));
}

pub(super) fn print_rivals_screen(game: &Game) {
    print_subscreen_frame(game, FramedScreen::Rivals);
    print_competitors(game);
}

pub(super) fn print_board(game: &Game) {
    print_box("Board Objectives", &board_overview_lines(game));
    print_box_pair(
        "Next Milestone",
        &board_milestone_lines(game),
        "Risk Notes",
        &board_risk_lines(game),
    );
    print_box("Milestone Path", &board_path_lines(game));
    if should_show_regional_status(game) {
        print_box("Regional Mandate", &regional_mandate_lines(game));
    }
}

pub(super) fn print_board_screen(game: &Game) {
    print_subscreen_frame(game, FramedScreen::Board);
    print_board(game);
}

pub(super) fn print_subscreen_frame(game: &Game, active: FramedScreen) {
    print_box(
        &subscreen_title(game, active),
        &subscreen_status_lines(game),
    );
    print_box("Navigation", &subscreen_navigation_lines(active));
}

pub(super) fn dashboard_title(game: &Game) -> String {
    format!("{} | {} | Dashboard", game.date_label(), game.player.name)
}

pub(super) fn subscreen_title(game: &Game, active: FramedScreen) -> String {
    format!(
        "{} | {} | {}",
        game.date_label(),
        game.player.name,
        active.label()
    )
}

pub(super) fn subscreen_status_lines(game: &Game) -> Vec<String> {
    vec![
        format!(
            "{} {} | {} {} | {} {} | {} {} | {} {}",
            muted("Cash"),
            styled(cash_tone(game.player.cash), money(game.player.cash)),
            muted("Share"),
            styled(
                share_tone(game.market_share()),
                format!("{:.0}%", game.market_share() * 100.0)
            ),
            muted("Reliability"),
            styled(
                reliability_tone(game.player.reliability),
                format!("{:.0}%", game.player.reliability * 100.0)
            ),
            muted("Debt/assets"),
            styled(
                leverage_tone(game.player.debt_to_assets()),
                format!("{:.0}%", game.player.debt_to_assets() * 100.0)
            ),
            muted("Rate"),
            styled(BOLD_CYAN, format!("{:.1}c", game.player.rate_cents))
        ),
        format!(
            "{} {} | {} {} | {} {}",
            muted("Review"),
            styled(BOLD_CYAN, subscreen_review_label(game)),
            muted("Avg rate"),
            styled(CYAN, format!("{:.1}c", market_average_rate(game))),
            muted("Tolerance"),
            styled(
                YELLOW,
                format!("{:.1}c", public_rate_tolerance(&game.market))
            )
        ),
    ]
}

pub(super) fn subscreen_navigation_lines(active: FramedScreen) -> Vec<String> {
    vec![
        format!(
            "{}  {}  {}  {}  {}",
            command_label("status"),
            nav_label(active, FramedScreen::Report),
            nav_label(active, FramedScreen::Rivals),
            nav_label(active, FramedScreen::Board),
            nav_label(active, FramedScreen::Help)
        ),
        format!(
            "{} cycles views | {} opens dashboard | direct-opened views return there",
            command_label("Return"),
            command_label("status")
        ),
    ]
}

fn nav_label(active: FramedScreen, screen: FramedScreen) -> String {
    if active == screen {
        styled(
            BOLD_CYAN,
            format!("[{}]", screen.label().to_ascii_lowercase()),
        )
    } else {
        command_label(screen.label().to_ascii_lowercase())
    }
}

fn command_label(label: impl std::fmt::Display) -> String {
    styled(BOLD_CYAN, label)
}

fn subscreen_review_label(game: &Game) -> String {
    if game.sandbox_mode {
        "sandbox, no board reviews".to_string()
    } else {
        next_review_status_label(game)
    }
}

pub(super) fn print_notice(message: &str) {
    println!();
    print_box("Notice", &wrap_plain_text(message, content_width()));
}

pub(super) fn stable_panel_lines(
    mut lines: Vec<String>,
    height: usize,
    overflow_note: &str,
) -> Vec<String> {
    if height == 0 {
        return Vec::new();
    }
    if lines.len() > height {
        lines.truncate(height.saturating_sub(1));
        lines.push(command_hint_line(overflow_note));
    }
    while lines.len() < height {
        lines.push(String::new());
    }
    lines
}

pub(super) fn last_quarter_lines(game: &Game) -> Vec<String> {
    let Some(report) = &game.last_report else {
        return vec![
            format!("{} no completed quarter yet", muted("Results")),
            format!(
                "{} {}   {} {}",
                muted("Rate"),
                styled(BOLD_CYAN, format!("{:.1}c", game.player.rate_cents)),
                muted("Market avg"),
                styled(CYAN, format!("{:.1}c", market_average_rate(game)))
            ),
            format!(
                "{} use {} to complete the first quarter",
                muted("Next"),
                command_label("next")
            ),
        ];
    };

    let net_customers = report.new_customers - report.lost_customers;
    let mut lines = vec![
        format!(
            "{} {}   {} {}   {} {}",
            muted(&report.label),
            styled(profit_tone(report.profit), money(report.profit)),
            muted("margin"),
            styled(
                profit_tone(report_margin(report)),
                format!("{:.1}%", report_margin(report))
            ),
            muted("interest"),
            styled(YELLOW, money(report.interest))
        ),
        format!(
            "{} {}   {} {}   {} {}",
            muted("Customers"),
            styled(
                customer_tone(net_customers),
                format!("{:+.0}", net_customers)
            ),
            muted("churn"),
            styled(
                churn_tone(report.lost_customer_rate),
                format_churn_rate(report.lost_customer_rate)
            ),
            muted("share"),
            styled(
                share_tone(report.market_share),
                format!(
                    "{:.0}% ({:.0}%)",
                    report.market_share * 100.0,
                    report.prior_market_share * 100.0
                )
            )
        ),
    ];

    if let Some(driver) = report.attributions.first() {
        lines.extend(dashboard_note_lines("Driver", driver, 2));
    } else {
        lines.push(format!("{} no unusual operating driver", muted("Driver")));
    }

    if let Some(event) = most_urgent_event(&report.events) {
        lines.extend(dashboard_note_lines("Event", event, 2));
    } else {
        lines.push(format!("{} none", muted("Events")));
    }

    if lines.len() < 6 {
        lines.push(format!(
            "{} {}",
            muted("More"),
            command_hint_line("report for full detail")
        ));
    }

    lines
}

fn dashboard_note_lines(label: &str, text: &str, max_lines: usize) -> Vec<String> {
    let prefix = format!("{} ", muted(label));
    let width = dashboard_pair_content_width();
    let text_width = width.saturating_sub(visible_width(&prefix)).max(16);
    let mut wrapped = wrap_plain_text(text, text_width);
    if wrapped.len() > max_lines {
        wrapped.truncate(max_lines);
        if let Some(last) = wrapped.last_mut() {
            let suffix = " ... report";
            let keep = text_width.saturating_sub(suffix.len()).max(8);
            *last = format!("{}{}", shorten_plain(last, keep), suffix);
        }
    }

    let continuation = " ".repeat(visible_width(&prefix));
    wrapped
        .into_iter()
        .enumerate()
        .map(|(index, line)| {
            if index == 0 {
                format!("{prefix}{line}")
            } else {
                format!("{continuation}{line}")
            }
        })
        .collect()
}

fn dashboard_pair_content_width() -> usize {
    paired_column_widths(dashboard_width()).0.saturating_sub(4)
}

/// Ranks a quarter event for the dashboard's single visible event slot.
///
/// The report screen keeps the full chronological log; this ranking only
/// decides which one event the dashboard surfaces. Without it, rival routine
/// moves (pushed earlier in the quarter than settlement) win the slot even in
/// quarters where the player's own operations are in crisis.
fn dashboard_event_priority(event: &str) -> u8 {
    const PLAYER_CRISIS: [&str; 10] = [
        "Overloaded generation",
        "Cash crunch",
        "Below-cost pricing",
        "Acquisition stress forced",
        "Lenders tightened acquisition covenants",
        "Board confidence weakened",
        "equipment fire at Metro",
        "Rates above public tolerance",
        "Acquisition integration strained",
        "Regional expansion is absorbing",
    ];
    const MARKET_SHIFT_OR_MILESTONE: [&str; 9] = [
        "rate freeze",
        "regional slowdown",
        "surge in industrial demand",
        "costs spiked",
        "Input costs moved",
        "Public rate tolerance",
        "Credit markets",
        "Demand conditions",
        "checkpoint passed",
    ];
    const RIVAL_ROUTINE: [&str; 9] = [
        "expanded generation and distribution capacity",
        "cut rates to",
        "matched Metro's price move",
        "raised rates to",
        "launched a retention campaign",
        "answered Metro's marketing push",
        "raised debt to fund a counter-expansion",
        "arranged financing after Metro",
        "rival generation offline",
    ];

    if PLAYER_CRISIS.iter().any(|pattern| event.contains(pattern)) {
        0
    } else if MARKET_SHIFT_OR_MILESTONE
        .iter()
        .any(|pattern| event.contains(pattern))
    {
        1
    } else if RIVAL_ROUTINE.iter().any(|pattern| event.contains(pattern)) {
        3
    } else {
        2
    }
}

pub(super) fn most_urgent_event(events: &[String]) -> Option<&String> {
    events
        .iter()
        .enumerate()
        .min_by_key(|(index, event)| (dashboard_event_priority(event), *index))
        .map(|(_, event)| event)
}

fn report_margin(report: &QuarterReport) -> f64 {
    if report.revenue > 0.0 {
        report.profit / report.revenue * 100.0
    } else {
        0.0
    }
}

pub(super) fn milestone_snapshot_lines(game: &Game) -> Vec<String> {
    let next = next_board_target(game);
    let coverage = game.player_last_interest_coverage();
    let (coverage_tone, coverage_label) = match coverage {
        Some(coverage) => (coverage_tone(coverage), format_coverage(coverage)),
        None => (DIM, "no completed quarter yet".to_string()),
    };
    vec![
        format!(
            "{} {}   {} {}",
            muted("Y5 review"),
            styled(BOLD_CYAN, quarters_remaining_label(game)),
            muted("Next"),
            styled(BOLD, next.label)
        ),
        {
            // Compare against the targets of the active era: Y5 review targets
            // before the review, Y10 mandate targets after, matching the
            // scorecard so the two panels never show different goalposts.
            let targets = scorecard_review_targets(game);
            format!(
                "{} {:.0}/{:.0}% | {} {:.0}/{:.0}% | {} {:.0}/{:.0}%",
                muted("share"),
                game.market_share() * 100.0,
                targets.share * 100.0,
                muted("reliability"),
                game.player.reliability * 100.0,
                targets.reliability * 100.0,
                muted("debt/assets"),
                game.player.debt_to_assets() * 100.0,
                targets.leverage * 100.0
            )
        },
        format!(
            "{} {}   {} {}",
            muted("Rate support"),
            styled(
                rate_support_tone(game.player_rate_support_ratio()),
                format!("{:.0}%", game.player_rate_support_ratio() * 100.0)
            ),
            muted("Coverage"),
            styled(coverage_tone, coverage_label)
        ),
        format!(
            "{} {}   {} {}",
            muted("Y10 mandate"),
            styled(BOLD_CYAN, regional_due_label(game)),
            muted("Territories"),
            styled(
                expansion_tone(game.adjacent_expansions),
                format!(
                    "{}/{}",
                    game.adjacent_expansions, REGIONAL_MANDATE_EXPANSION_TARGET
                )
            )
        ),
    ]
}

pub(super) fn compact_command_lines(game: &Game) -> Vec<String> {
    let (opportunity_command, opportunity_effect) = compact_opportunity_command(game);

    vec![
        compact_command_pair_line("next / n", "finish", "preview <cmd>", "inspect"),
        compact_command_pair_line(
            "build gen 400",
            project_quote_short("gen", 400.0),
            "build lines 600",
            project_quote_short("lines", 600.0),
        ),
        compact_command_pair_line(
            format!("rate {:.1}", game.market.standard_rate_cents),
            "set price",
            "marketing 4",
            "spend $4k",
        ),
        compact_command_pair_line("maint 6", "spend $6k", "hire maint 95%", "auto upkeep"),
        compact_command_pair_line("hire marketing 90", "auto rep", "fire maint", "stop auto"),
        compact_command_pair_line(
            "borrow max",
            styled(
                borrowing_room_tone(game.borrowing_room()),
                money(game.borrowing_room()),
            ),
            opportunity_command,
            opportunity_effect,
        ),
    ]
}

fn compact_opportunity_command(game: &Game) -> (String, String) {
    if let Some((index, price)) = cheapest_diligenced_competitor(game) {
        let rank = competitor_rank(game, index).unwrap_or(index + 1);
        if game.competitors.len() == 1 {
            ("buy".to_string(), "blocked".to_string())
        } else if game.acquisition_cooldown > 0 {
            (
                format!("buy {rank}"),
                format!("waits {}q", game.acquisition_cooldown),
            )
        } else {
            (format!("buy {rank}"), format!("needs {}", money(price)))
        }
    } else if let Some((index, estimate)) = cheapest_public_acquisition_target(game) {
        let rank = competitor_rank(game, index).unwrap_or(index + 1);
        (
            format!("buy/diligence {rank}"),
            format!("est {}", money(estimate)),
        )
    } else if should_show_adjacent_expansion(game) {
        ("expand".to_string(), adjacent_expansion_footer(game))
    } else {
        ("rivals".to_string(), "details".to_string())
    }
}

fn compact_command_pair_line(
    left_command: impl std::fmt::Display,
    left_effect: impl std::fmt::Display,
    right_command: impl std::fmt::Display,
    right_effect: impl std::fmt::Display,
) -> String {
    format!(
        "{} {} | {} {}",
        command_cell(left_command, 17),
        effect_cell(left_effect, 11),
        command_cell(right_command, 16),
        effect_cell(right_effect, 12)
    )
}

fn command_cell(command: impl std::fmt::Display, width: usize) -> String {
    let command = command.to_string();
    let command = fit_line(&command, width);
    let padding = width.saturating_sub(visible_width(&command));
    format!("{}{}", styled(BOLD_CYAN, command), " ".repeat(padding))
}

fn effect_cell(effect: impl std::fmt::Display, width: usize) -> String {
    let effect = fit_line(&effect.to_string(), width);
    let padding = width.saturating_sub(visible_width(&effect));
    format!("{effect}{}", " ".repeat(padding))
}

pub(super) fn project_quote_short(kind: &str, size: f64) -> String {
    match kind {
        "gen" => {
            let cost = generation_project_cost(size);
            format!(
                "{}/{}q",
                styled(BOLD_YELLOW, money(cost)),
                styled(YELLOW, generation_project_duration(size))
            )
        }
        "lines" => {
            let cost = distribution_project_cost(size);
            format!(
                "{}/{}q",
                styled(BOLD_YELLOW, money(cost)),
                styled(YELLOW, distribution_project_duration(size))
            )
        }
        _ => String::new(),
    }
}

pub(super) fn scorecard_lines(game: &Game) -> Vec<String> {
    if game.sandbox_mode {
        return sandbox_scorecard_lines(game);
    }

    let share = game.market_share();
    let reliability = game.player.reliability;
    let leverage = game.player.debt_to_assets();
    let rate_support = game.player_rate_support_ratio();
    let review_targets = scorecard_review_targets(game);

    let mut lines = vec![
        format!(
            "{} {} | {} {} | {} {} | {} {}",
            muted("Next review"),
            styled(BOLD_CYAN, next_review_status_label(game)),
            muted("Rate"),
            styled(BOLD_CYAN, format!("{:.1}c/kWh", game.player.rate_cents)),
            muted("Avg"),
            styled(CYAN, format!("{:.1}c", market_average_rate(game))),
            muted("Tolerance"),
            styled(
                YELLOW,
                format!("{:.1}c", public_rate_tolerance(&game.market))
            )
        ),
        score_line(
            "Share",
            share,
            review_targets.share,
            share_tone(share),
            format!("{:.0}% target", review_targets.share * 100.0),
            if share >= review_targets.share {
                "target met".to_string()
            } else {
                format!("{:.0} pts to go", (review_targets.share - share) * 100.0)
            },
        ),
        score_line(
            "Reliability",
            reliability,
            review_targets.reliability,
            reliability_tone(reliability),
            format!("{:.0}% target", review_targets.reliability * 100.0),
            if reliability >= review_targets.reliability {
                format!(
                    "{:.0} pts above",
                    (reliability - review_targets.reliability) * 100.0
                )
            } else {
                format!(
                    "{:.0} pts short",
                    (review_targets.reliability - reliability) * 100.0
                )
            },
        ),
        score_line(
            "Debt/assets",
            leverage,
            review_targets.leverage,
            leverage_tone(leverage),
            format!("{:.0}% limit", review_targets.leverage * 100.0),
            if leverage <= review_targets.leverage {
                format!(
                    "{:.0} pts room",
                    (review_targets.leverage - leverage) * 100.0
                )
            } else {
                format!(
                    "{:.0} pts over",
                    (leverage - review_targets.leverage) * 100.0
                )
            },
        ),
    ];

    if review_targets.regional {
        let expansion_progress = (game.adjacent_expansions as f64
            / REGIONAL_MANDATE_EXPANSION_TARGET.max(1) as f64)
            .clamp(0.0, 1.0);
        lines.push(score_line(
            "Territories",
            expansion_progress,
            1.0,
            expansion_tone(game.adjacent_expansions),
            format!("{} needed", REGIONAL_MANDATE_EXPANSION_TARGET),
            if game.adjacent_expansions >= REGIONAL_MANDATE_EXPANSION_TARGET {
                "target met".to_string()
            } else {
                format!(
                    "{} to go",
                    REGIONAL_MANDATE_EXPANSION_TARGET - game.adjacent_expansions
                )
            },
        ));
    } else {
        lines.push(score_line(
            "Rate support",
            rate_support.min(1.35),
            REVIEW_MIN_RATE_SUPPORT_RATIO,
            rate_support_tone(rate_support),
            format!("{:.0}% min", REVIEW_MIN_RATE_SUPPORT_RATIO * 100.0),
            if rate_support >= 1.0 {
                "above break-even".to_string()
            } else {
                format!("{:.0}% of break-even", rate_support * 100.0)
            },
        ));
    }

    lines
}

fn sandbox_scorecard_lines(game: &Game) -> Vec<String> {
    let share = game.market_share();
    let reliability = game.player.reliability;
    let leverage = game.player.debt_to_assets();
    let rate_support = game.player_rate_support_ratio();

    vec![
        format!(
            "{} {} | {} {} | {} {} | {} {}",
            muted("Mode"),
            styled(BOLD_CYAN, "Sandbox, no board reviews"),
            muted("Rate"),
            styled(BOLD_CYAN, format!("{:.1}c/kWh", game.player.rate_cents)),
            muted("Avg"),
            styled(CYAN, format!("{:.1}c", market_average_rate(game))),
            muted("Tolerance"),
            styled(
                YELLOW,
                format!("{:.1}c", public_rate_tolerance(&game.market))
            )
        ),
        score_line(
            "Share",
            share,
            0.50,
            share_tone(share),
            "50% leadership".to_string(),
            if share >= 0.50 {
                "market leader".to_string()
            } else {
                format!("{:.0} pts from lead", (0.50 - share) * 100.0)
            },
        ),
        score_line(
            "Reliability",
            reliability,
            0.80,
            reliability_tone(reliability),
            "80% service marker".to_string(),
            if reliability >= 0.80 {
                format!("{:.0} pts above", (reliability - 0.80) * 100.0)
            } else {
                format!("{:.0} pts short", (0.80 - reliability) * 100.0)
            },
        ),
        score_line(
            "Debt/assets",
            leverage,
            0.90,
            leverage_tone(leverage),
            "90% lender stress".to_string(),
            if leverage <= 0.90 {
                format!("{:.0} pts room", (0.90 - leverage) * 100.0)
            } else {
                format!("{:.0} pts over", (leverage - 0.90) * 100.0)
            },
        ),
        score_line(
            "Rate support",
            rate_support.min(1.35),
            1.0,
            rate_support_tone(rate_support),
            "break-even".to_string(),
            if rate_support >= 1.0 {
                "above break-even".to_string()
            } else {
                format!("{:.0}% of break-even", rate_support * 100.0)
            },
        ),
    ]
}

fn next_review_status_label(game: &Game) -> String {
    if !game.review_completed {
        return review_countdown_label("Y5", game.campaign_quarters.saturating_sub(game.quarter));
    }

    if game.regional_mandate_completed {
        "all reviews complete".to_string()
    } else {
        review_countdown_label("Y10 mandate", game.regional_mandate_due_in())
    }
}

fn review_countdown_label(name: &str, remaining: u32) -> String {
    match remaining {
        0 => format!("{name} due now"),
        1 => format!("{name} in 1 quarter"),
        _ => format!("{name} in {remaining} quarters"),
    }
}

pub(super) fn score_line(
    label: &str,
    current: f64,
    reference: f64,
    tone: &str,
    target: String,
    note: String,
) -> String {
    format!(
        "{} {} / {}  {}  {}",
        styled(BOLD, label),
        styled(tone, format!("{:.0}%", current * 100.0)),
        muted(target),
        progress_meter(current, reference, tone),
        muted(note)
    )
}

pub(super) fn progress_meter(current: f64, reference: f64, tone: &str) -> String {
    let width = 16;
    let filled = ((current / reference.max(0.01)).clamp(0.0, 1.0) * width as f64).round() as usize;
    styled(
        tone,
        format!("[{}{}]", "=".repeat(filled), ".".repeat(width - filled)),
    )
}

fn rate_support_tone(ratio: f64) -> &'static str {
    if ratio < REVIEW_MIN_RATE_SUPPORT_RATIO {
        RED
    } else if ratio < 1.0 {
        YELLOW
    } else {
        GREEN
    }
}

fn coverage_tone(coverage: f64) -> &'static str {
    if coverage < REVIEW_MIN_INTEREST_COVERAGE {
        RED
    } else if coverage < 1.30 {
        YELLOW
    } else {
        GREEN
    }
}

fn manager_tone(active: bool) -> &'static str {
    if active { GREEN } else { DIM }
}

pub(super) fn quarters_remaining_label(game: &Game) -> String {
    if game.review_completed {
        // The caller prefixes this with the row label "Y5 review", so the
        // status must not repeat the word ("Y5 review review complete").
        return "complete".to_string();
    }

    let remaining = game.campaign_quarters.saturating_sub(game.quarter);
    match remaining {
        0 => "review due now".to_string(),
        1 => "1 quarter left".to_string(),
        _ => format!("{remaining} quarters left"),
    }
}

pub(super) fn financial_lines(game: &Game) -> Vec<String> {
    let borrowing_room = game.borrowing_room();
    let leverage = game.player.debt_to_assets();
    let debt_rate = game.player_annual_interest_rate();
    let break_even = game.player_break_even_rate_cents();
    let rate_support = game.player_rate_support_ratio();
    let mut lines = vec![
        format!(
            "{} {}   {} {}",
            muted("Cash"),
            styled(cash_tone(game.player.cash), money(game.player.cash)),
            muted("Debt"),
            styled(leverage_tone(leverage), money(game.player.debt))
        ),
        format!(
            "{} {}   {} {}",
            muted("Borrow room"),
            styled(borrowing_room_tone(borrowing_room), money(borrowing_room)),
            muted("Debt rate"),
            styled(
                interest_rate_tone(debt_rate),
                format!("{:.1}%", debt_rate * 100.0)
            )
        ),
        format!(
            "{} {}   {} {}   {} {}",
            muted("Stock"),
            styled(BOLD_CYAN, format!("${:.2}", game.player.stock_price)),
            muted("Market cap"),
            styled(BOLD, money(game.player.market_cap())),
            muted("Founder"),
            styled(
                ownership_tone(game.player_ownership()),
                format!(
                    "{:.0}/{:.0} ({:.1}%)",
                    game.player_owned_shares,
                    game.player.shares,
                    game.player_ownership() * 100.0
                )
            )
        ),
        format!(
            "{} {}   {} {}",
            muted("Founder value"),
            styled(BOLD_GREEN, money(game.player_wealth())),
            muted("Dividends"),
            styled(GREEN, money(game.player_dividends_received))
        ),
    ];

    if game.macro_state.credit_label() != "normal"
        || borrowing_room < 12_000.0
        || game.borrowing_limit_ratio() < 0.82
    {
        lines.push(format!(
            "{} {}   {} {}",
            muted("Credit"),
            styled(macro_credit_tone(game), game.macro_state.credit_label()),
            muted("Debt cap"),
            styled(
                debt_cap_tone(game.borrowing_limit_ratio()),
                format!("{:.0}%", game.borrowing_limit_ratio() * 100.0)
            )
        ));
    }

    if let Some(report) = &game.last_report {
        let margin = if report.revenue > 0.0 {
            report.profit / report.revenue * 100.0
        } else {
            0.0
        };
        let coverage = game
            .player_last_interest_coverage()
            .unwrap_or(f64::INFINITY);
        lines.push(format!(
            "{} {}   {} {}",
            muted("Last profit"),
            styled(profit_tone(report.profit), money(report.profit)),
            muted("margin"),
            styled(profit_tone(margin), format!("{:.1}%", margin))
        ));
        lines.push(format!(
            "{} {}   {} {}",
            muted("Break-even"),
            styled(
                rate_support_tone(rate_support),
                format!("{:.1}c", break_even)
            ),
            muted("Coverage"),
            styled(coverage_tone(coverage), format_coverage(coverage))
        ));
    } else {
        lines.push(format!("{} no operating report yet", muted("Last q")));
        lines.push(format!(
            "{} {}   {} {}",
            muted("Break-even"),
            styled(
                rate_support_tone(rate_support),
                format!("{:.1}c", break_even)
            ),
            muted("Rate support"),
            styled(
                rate_support_tone(rate_support),
                format!("{:.0}%", rate_support * 100.0)
            )
        ));
    }

    lines
}

pub(super) fn operation_lines(game: &Game) -> Vec<String> {
    let capacity = game.player.customer_capacity(&game.market);
    let headroom = (capacity - game.player.customers).max(0.0);
    let generation_customer_capacity =
        game.player.generation_capacity_mwh / game.market.avg_mwh_per_customer.max(0.05);
    let demanded_mwh = game.player.demanded_mwh(&game.market);
    let firm_generation_reserve = game.player.firm_generation_reserve_mwh(&game.market);
    let limiting = if game.player.distribution_capacity <= generation_customer_capacity {
        "distribution"
    } else {
        "generation"
    };

    let maintenance_manager = game
        .maintenance_manager_target
        .map(|target| format!("maint {:.0}%", target * 100.0))
        .unwrap_or_else(|| "maint off".to_string());
    let marketing_manager = game
        .marketing_manager_target
        .map(|target| format!("market {:.0}", target))
        .unwrap_or_else(|| "market off".to_string());

    vec![
        format!(
            "{} {} / {}   {} {}",
            muted("Customers"),
            styled(BOLD, format!("{:.0}", game.player.customers)),
            styled(BOLD, format!("{:.0}", capacity)),
            muted("Headroom"),
            styled(headroom_tone(headroom), format!("{:.0}", headroom))
        ),
        format!(
            "{} {}   {} {}",
            muted("Generation"),
            styled(
                BOLD,
                format!("{:.0} MWh/q", game.player.generation_capacity_mwh)
            ),
            muted("Demand"),
            styled(BOLD, format!("{:.0}", demanded_mwh)),
        ),
        format!(
            "{} {}   {} {}",
            muted("Firm reserve"),
            styled(
                reserve_tone(firm_generation_reserve),
                format!("{:+.0} MWh", firm_generation_reserve)
            ),
            muted("Limit"),
            styled(
                if limiting == "generation" {
                    YELLOW
                } else {
                    CYAN
                },
                limiting
            )
        ),
        format!(
            "{} {}",
            muted("Utilization"),
            styled(
                utilization_tone(game.player.utilization(&game.market)),
                format!("{:.0}%", game.player.utilization(&game.market) * 100.0)
            ),
        ),
        format!(
            "{} {}   {} {}",
            muted("Reliability"),
            styled(
                reliability_tone(game.player.reliability),
                format!("{:.0}%", game.player.reliability * 100.0)
            ),
            muted("Reputation"),
            styled(
                reputation_tone(game.player.reputation),
                format!("{:.0}", game.player.reputation)
            )
        ),
        format!(
            "{} {}   {}",
            muted("Managers"),
            styled(
                manager_tone(game.maintenance_manager_target.is_some()),
                maintenance_manager
            ),
            styled(
                manager_tone(game.marketing_manager_target.is_some()),
                marketing_manager
            )
        ),
    ]
}

pub(super) fn project_lines(game: &Game) -> Vec<String> {
    let mut lines = if game.pending_projects.is_empty() && game.acquisition_cooldown == 0 {
        vec![styled(DIM, "No active pipeline.")]
    } else if game.pending_projects.is_empty() {
        Vec::new()
    } else {
        game.pending_projects
            .iter()
            .map(|project| {
                let summary = match &project.kind {
                    crate::sim::ProjectKind::Generation { capacity_mwh } => {
                        format!("gen +{capacity_mwh:.0} MWh/q")
                    }
                    crate::sim::ProjectKind::Distribution { customer_capacity } => {
                        format!("lines +{customer_capacity:.0} cust")
                    }
                    crate::sim::ProjectKind::AdjacentTerritory {
                        addressable_customers,
                        ..
                    } => format!("territory +{addressable_customers:.0} addr"),
                };
                format!(
                    "{}   {}",
                    styled(BOLD, summary),
                    styled(YELLOW, format!("{}q left", project.quarters_remaining))
                )
            })
            .collect()
    };

    if game.acquisition_cooldown > 0 {
        lines.push(format!(
            "{} {}",
            muted("Integration"),
            styled(
                integration_strain_tone(game.integration_strain),
                format!(
                    "{}q left | {} burden",
                    game.acquisition_cooldown,
                    integration_strain_label(game.integration_strain)
                )
            )
        ));
    }

    lines
}

pub(super) fn signal_lines(game: &Game) -> Vec<String> {
    #[derive(Clone)]
    struct SignalCandidate {
        priority: u8,
        order: usize,
        line: String,
    }

    let mut lines: Vec<SignalCandidate> = Vec::new();
    let mut push_line = |priority: u8, line: String| {
        let order = lines.len();
        lines.push(SignalCandidate {
            priority,
            order,
            line,
        });
    };

    if !game.active_shocks.is_empty() {
        let descriptors: Vec<String> = game
            .active_shocks
            .iter()
            .map(|shock| format!("{} {}q", shock_label(&shock.kind), shock.quarters_remaining))
            .collect();
        push_line(100, signal_line("Shock", BOLD_RED, descriptors.join(" | ")));
    }

    if let Some(report) = &game.last_report {
        let net_customers = report.new_customers - report.lost_customers;
        push_line(
            45,
            signal_line(
                "Trend",
                CYAN,
                format!(
                    "net {} | churn {}",
                    styled(
                        customer_tone(net_customers),
                        format!("{:+.0}", net_customers)
                    ),
                    styled(
                        churn_tone(report.lost_customer_rate),
                        format_churn_rate(report.lost_customer_rate)
                    )
                ),
            ),
        );
    } else {
        push_line(45, signal_line("Trend", CYAN, "no prior quarter yet"));
    }

    push_line(
        40,
        signal_line(
            "Macro",
            macro_credit_tone(game),
            format!(
                "credit {} | demand {} | costs {}",
                styled(macro_credit_tone(game), game.macro_state.credit_label()),
                styled(
                    demand_tone(game.macro_state.demand_index),
                    game.macro_state.demand_label()
                ),
                styled(
                    cost_pressure_tone(game.macro_state.cost_pressure),
                    game.macro_state.cost_label()
                )
            ),
        ),
    );

    let average_rate = market_average_rate(game);
    let rate_gap = game.player.rate_cents - average_rate;
    let tolerance = public_rate_tolerance(&game.market);
    let break_even = game.player_break_even_rate_cents();
    let rate_support = game.player_rate_support_ratio();
    if game.player.rate_cents > tolerance {
        push_line(
            86,
            signal_line(
                "Rate",
                RED,
                format!(
                    "{} above public tolerance; intervention risk",
                    styled(
                        BOLD_RED,
                        format!("{:.1}c", game.player.rate_cents - tolerance)
                    )
                ),
            ),
        );
    } else if rate_gap >= 0.4 {
        push_line(
            60,
            signal_line(
                "Rate",
                YELLOW,
                format!(
                    "{} above market; churn pressure",
                    styled(BOLD_YELLOW, format!("{:.1}c", rate_gap))
                ),
            ),
        );
    } else if rate_gap <= -0.4 {
        push_line(
            60,
            signal_line(
                "Rate",
                CYAN,
                format!(
                    "{} below market; margin tightens",
                    styled(BOLD_CYAN, format!("{:.1}c", rate_gap.abs()))
                ),
            ),
        );
    } else {
        push_line(35, signal_line("Rate", CYAN, "near market average"));
    }
    if rate_support < REVIEW_MIN_RATE_SUPPORT_RATIO {
        push_line(
            88,
            signal_line(
                "Earnings",
                RED,
                format!(
                    "rate {:.1}c below sustainable {:.1}c break-even",
                    game.player.rate_cents, break_even
                ),
            ),
        );
    } else if rate_support < 1.0 {
        push_line(
            58,
            signal_line("Earnings", YELLOW, "rate is below full break-even"),
        );
    }
    if let Some(coverage) = game.player_last_interest_coverage() {
        if coverage < REVIEW_MIN_INTEREST_COVERAGE {
            push_line(
                83,
                signal_line(
                    "Coverage",
                    RED,
                    format!("debt service not covered ({})", format_coverage(coverage)),
                ),
            );
        } else if coverage < 1.30 {
            push_line(
                56,
                signal_line(
                    "Coverage",
                    YELLOW,
                    format!("thin interest coverage ({})", format_coverage(coverage)),
                ),
            );
        }
    }

    let capacity = game.player.customer_capacity(&game.market);
    let headroom = capacity - game.player.customers;
    let firm_generation_reserve = game.player.firm_generation_reserve_mwh(&game.market);
    if headroom < 60.0 {
        push_line(
            84,
            signal_line("Capacity", RED, "tight; build capacity first"),
        );
    } else if headroom < 140.0 {
        push_line(
            55,
            signal_line("Capacity", YELLOW, "adequate, but narrowing"),
        );
    } else {
        push_line(
            25,
            signal_line("Capacity", GREEN, "growth headroom available"),
        );
    }
    if firm_generation_reserve < 0.0 {
        // A negative firm reserve is the reliability death spiral in motion:
        // unmet demand erodes reliability, which cuts firm capacity further.
        // "Thin" radically undersells it, so escalate above the other signals.
        push_line(
            90,
            signal_line(
                "Generation",
                BOLD_RED,
                "demand exceeds firm capacity; reliability eroding",
            ),
        );
    } else if firm_generation_reserve < 15.0 {
        push_line(
            82,
            signal_line(
                "Generation",
                reserve_tone(firm_generation_reserve),
                "firm reserve thin",
            ),
        );
    }

    let leverage = game.player.debt_to_assets();
    if leverage > 0.85 {
        push_line(80, signal_line("Debt", RED, "high leverage; lender risk"));
    } else if leverage < 0.55 {
        push_line(
            30,
            signal_line("Debt", GREEN, "borrowing capacity available"),
        );
    } else {
        push_line(
            35,
            signal_line("Debt", YELLOW, "usable, but no longer cheap"),
        );
    }
    let valuation_to_assets = game.player.market_cap() / game.player.asset_base.max(1.0);
    if valuation_to_assets < 0.50 && leverage > 0.80 {
        push_line(
            82,
            signal_line("Control", RED, "weak valuation invites hostile bid"),
        );
    }

    if game.acquisition_stress >= ACQUISITION_STRESS_STRAINED {
        push_line(
            81,
            signal_line("M&A", RED, "covenants strained after acquisition"),
        );
    } else if game.acquisition_stress >= ACQUISITION_STRESS_NOTICE {
        push_line(
            57,
            signal_line("M&A", YELLOW, "lenders watching integration"),
        );
    }
    if game.integration_strain >= 0.30 {
        push_line(
            79,
            signal_line(
                "Integration",
                integration_strain_tone(game.integration_strain),
                "heavy operating burden from recent deal",
            ),
        );
    } else if game.integration_strain >= 0.08 {
        push_line(
            54,
            signal_line(
                "Integration",
                integration_strain_tone(game.integration_strain),
                "recent deal still absorbing management attention",
            ),
        );
    }

    if game.player.reliability < 0.50 {
        push_line(91, signal_line("Reliability", RED, "market access at risk"));
    } else if game.player.reliability < 0.74 {
        // Prescribe by cause: maintenance counters wear, but when demand
        // exceeds firm capacity only added generation stops the erosion.
        let prescription = if firm_generation_reserve < 0.0 {
            "below target; overloaded - build generation"
        } else {
            "below target; fund maintenance"
        };
        push_line(78, signal_line("Reliability", RED, prescription));
    } else if game.player.reliability > 0.88 {
        push_line(
            25,
            signal_line("Reliability", GREEN, "strong; expansion can lead"),
        );
    }

    if let Some(warning) = review_gate_warning(game) {
        push_line(76, warning);
    }

    let maintenance_scale = game.player_maintenance_response_scale();
    if maintenance_scale < 0.70 {
        push_line(
            34,
            signal_line(
                "Maintenance",
                YELLOW,
                format!(
                    "large asset base: upkeep dollars work at {:.0}% response",
                    maintenance_scale * 100.0
                ),
            ),
        );
    }

    if lines.len() > 6 {
        lines.sort_by(|left, right| {
            right
                .priority
                .cmp(&left.priority)
                .then_with(|| left.order.cmp(&right.order))
        });
        lines.truncate(6);
        lines.sort_by_key(|candidate| candidate.order);
    }

    lines.into_iter().map(|candidate| candidate.line).collect()
}

pub(super) fn shock_label(kind: &ShockKind) -> &'static str {
    match kind {
        ShockKind::RateFreeze => "rate freeze",
        ShockKind::DemandRecession => "demand recession",
        ShockKind::DemandBoom => "demand boom",
        ShockKind::InputCostShock => "input cost shock",
    }
}

#[cfg(test)]
pub(super) fn command_footer_lines(game: &Game) -> Vec<String> {
    let mut lines = vec![action_line(
        "next/n | preview",
        "finish the quarter or inspect a command before committing",
    )];

    if dashboard_width() >= 112 {
        lines.push(action_line(
            "build gen/lines",
            format!(
                "gen 400 {} | lines 600 {}",
                project_quote_inline("gen", 400.0),
                project_quote_inline("lines", 600.0)
            ),
        ));
    } else {
        lines.push(action_line(
            "build gen 400",
            project_quote_inline("gen", 400.0),
        ));
        lines.push(action_line(
            "build lines 600",
            project_quote_inline("lines", 600.0),
        ));
    }

    lines.push(action_line(
        "marketing/maint",
        if game.regional_integration > 0.12 {
            "manual spend or hire managers; both work down regional integration"
        } else {
            "marketing 4, maint 6, or hire maint/marketing"
        },
    ));
    if should_show_adjacent_expansion(game) {
        lines.push(action_line("expand", adjacent_expansion_footer(game)));
    }
    lines.push(action_line(
        "rate | borrow",
        if dashboard_width() >= 112 {
            format!(
                "rate {:.1} near market; borrow max {} at {}; issue/buyback/dividend",
                game.market.standard_rate_cents,
                styled(
                    borrowing_room_tone(game.borrowing_room()),
                    money(game.borrowing_room())
                ),
                styled(
                    interest_rate_tone(game.player_annual_interest_rate()),
                    format!("{:.1}%", game.player_annual_interest_rate() * 100.0)
                )
            )
        } else {
            format!(
                "rate {:.1}; borrow max {} @ {}; equity/dividend",
                game.market.standard_rate_cents,
                styled(
                    borrowing_room_tone(game.borrowing_room()),
                    money(game.borrowing_room())
                ),
                styled(
                    interest_rate_tone(game.player_annual_interest_rate()),
                    format!("{:.1}%", game.player_annual_interest_rate() * 100.0)
                )
            )
        },
    ));
    lines.push(action_line(
        "rivals | board | save",
        "competitor detail, objectives, and save/load",
    ));

    if let Some((index, price)) = cheapest_diligenced_competitor(game) {
        let rank = competitor_rank(game, index).unwrap_or(index + 1);
        if game.competitors.len() == 1 {
            lines.push(action_line(
                "buy",
                styled(RED, "final rival acquisition blocked by regulator"),
            ));
        } else if game.acquisition_cooldown > 0 {
            lines.push(action_line(
                format!("buy {rank}"),
                format!(
                    "unavailable for {} more quarter(s) during integration",
                    styled(YELLOW, game.acquisition_cooldown)
                ),
            ));
        } else {
            lines.push(action_line(
                format!("buy {rank}"),
                format!(
                    "{} cash plus debt; integration strain follows",
                    styled(BOLD_YELLOW, money(price))
                ),
            ));
        }
    } else if let Some((index, estimate)) = cheapest_public_acquisition_target(game) {
        let rank = competitor_rank(game, index).unwrap_or(index + 1);
        if game.competitors.len() == 1 {
            lines.push(action_line(
                "buy",
                styled(RED, "final rival acquisition blocked by regulator"),
            ));
        } else if game.acquisition_cooldown > 0 {
            lines.push(action_line(
                format!("diligence {rank}"),
                format!(
                    "can inspect now; acquisitions wait {}q",
                    styled(YELLOW, game.acquisition_cooldown)
                ),
            ));
        } else {
            let diligence_note = game
                .diligence_cost(index)
                .map(|cost| {
                    format!(
                        "; diligence {} freezes terms for {}",
                        rank,
                        styled(BOLD_YELLOW, money(cost))
                    )
                })
                .unwrap_or_default();
            lines.push(action_line(
                format!("buy {rank} | diligence {rank}"),
                format!(
                    "public est {}; close risk{}",
                    styled(BOLD_YELLOW, money(estimate)),
                    diligence_note
                ),
            ));
        }
    } else if let Some((index, cost)) = cheapest_diligence_target(game) {
        let rank = competitor_rank(game, index).unwrap_or(index + 1);
        lines.push(action_line(
            format!("diligence {rank}"),
            format!(
                "{} to reveal exact acquisition terms",
                styled(BOLD_YELLOW, money(cost))
            ),
        ));
    }

    lines
}

pub(super) fn should_show_adjacent_expansion(game: &Game) -> bool {
    game.quarter >= 6
        || game.review_completed
        || game.adjacent_expansion_pending()
        || game.adjacent_expansions > 0
}

pub(super) fn adjacent_expansion_footer(game: &Game) -> String {
    let cost = game.adjacent_expansion_cost();
    if game.adjacent_expansion_pending() {
        return styled(YELLOW, "adjacent territory entry already in pipeline");
    }
    if let Some(blocker) = game.adjacent_expansion_blocker() {
        return blocker;
    }
    if game.player.cash < cost {
        return format!(
            "{} | {}q | raise {} more",
            styled(BOLD_YELLOW, money(cost)),
            styled(YELLOW, game.adjacent_expansion_duration()),
            styled(YELLOW, money(cost - game.player.cash))
        );
    }
    format!(
        "{} | {}q | +{:.0} addressable customers",
        styled(BOLD_YELLOW, money(cost)),
        styled(YELLOW, game.adjacent_expansion_duration()),
        ADJACENT_EXPANSION_ADDRESSABLE_CUSTOMERS
    )
}

#[cfg(test)]
pub(super) fn project_quote_inline(kind: &str, size: f64) -> String {
    match kind {
        "gen" => {
            let cost = generation_project_cost(size);
            format!(
                "{} | {} | {}",
                styled(BOLD_YELLOW, money(cost)),
                styled(YELLOW, format!("{}q", generation_project_duration(size))),
                styled(DIM, format!("{}/MWh", money(cost / size)))
            )
        }
        "lines" => {
            let cost = distribution_project_cost(size);
            format!(
                "{} | {} | {}",
                styled(BOLD_YELLOW, money(cost)),
                styled(YELLOW, format!("{}q", distribution_project_duration(size))),
                styled(DIM, format!("{}/cust", money(cost / size)))
            )
        }
        _ => String::new(),
    }
}
