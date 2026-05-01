use super::render::*;
use super::rivals::{dashboard_competitor_lines, rival_detail_lines, rival_overview_lines};
use super::*;

pub(super) struct BoardTarget {
    label: &'static str,
    quarter: u32,
    share: f64,
    reliability: f64,
    leverage: f64,
}

pub(super) fn print_start_screen(game: &Game) {
    print_box(
        &format!("{} | Start", game.player.name),
        &start_screen_overview_lines(game),
    );
    print_box_pair(
        "How To Read It",
        &start_screen_metric_lines(),
        "Useful Commands",
        &start_screen_command_lines(game),
    );
    print_box("Begin", &start_screen_begin_lines());
}

pub(super) fn start_screen_overview_lines(game: &Game) -> Vec<String> {
    vec![
        styled(BOLD_CYAN, "more corgi's rural electrification!"),
        format!(
            "You run {} in {}.",
            styled(BOLD, &game.player.name),
            game.market.territory
        ),
        "Build a durable utility: enough customers, enough capacity, reliable service.".to_string(),
        "Keep a balance sheet that can survive bad quarters.".to_string(),
        "The Year 5 review tests share, reliability, leverage, and sustainable earnings."
            .to_string(),
        "Continued games add a Year 10 regional mandate.".to_string(),
    ]
}

pub(super) fn start_screen_metric_lines() -> Vec<String> {
    vec![
        "Cash: projects and shock cushion.".to_string(),
        "Rates: revenue, churn, public tolerance.".to_string(),
        "Capacity: watts and customer hookups.".to_string(),
        "Reliability: growth, churn, reputation.".to_string(),
        "Debt/assets: lender room and failure risk.".to_string(),
    ]
}

pub(super) fn start_screen_command_lines(game: &Game) -> Vec<String> {
    vec![
        compact_action_line("next / n", "finish the quarter"),
        compact_action_line("preview <cmd>", "inspect before acting"),
        compact_action_line(
            "rate / build",
            format!(
                "set price; add gen/lines near {:.1}c",
                game.market.standard_rate_cents
            ),
        ),
        compact_action_line("marketing / maint", "growth; reliability"),
        compact_action_line("hire / fire", "automate marketing or upkeep"),
        compact_action_line("capital", "debt, equity, buybacks"),
        compact_action_line("rivals / board", "competitors; objectives"),
        compact_action_line("save / load", "keep long campaigns"),
    ]
}

pub(super) fn start_screen_begin_lines() -> Vec<String> {
    vec![
        "Press Return to open the main dashboard.".to_string(),
        "From the dashboard, Return cycles: Report -> Rivals -> Board -> Help -> Dashboard."
            .to_string(),
        "From a screen opened by command, Return goes back to the dashboard.".to_string(),
        "Type help at any time for the full command reference.".to_string(),
    ]
}

pub(super) fn print_help() {
    println!("{}", styled(BOLD_CYAN, "Electrification Command Reference"));
    print_box_pair(
        "Build + Operate",
        &[
            "build gen [MWh]".to_string(),
            "build lines [customers]".to_string(),
            "marketing [amount]".to_string(),
            "maint [amount]       maintenance".to_string(),
            "hire maint 95%       auto service work".to_string(),
            "hire marketing 90    auto reputation work".to_string(),
            "fire maint/marketing dismiss manager".to_string(),
            "expand               adjacent territory".to_string(),
            "rate 10.0            set target".to_string(),
            "rate up|down [cents] adjust rate".to_string(),
            "preview <command>    inspect first".to_string(),
        ],
        "Capital",
        &[
            "issue [amount]       issue stock".to_string(),
            "buyback [amount]     repurchase shares".to_string(),
            "borrow [amount|max]  raise debt".to_string(),
            "repay [amount|max]   pay debt down".to_string(),
            "diligence <number>   reveal deal terms".to_string(),
            "buy <number>         acquire rival".to_string(),
            "stock issue/buyback  aliases".to_string(),
            "debt / debt repay    aliases".to_string(),
        ],
    );
    print_box_pair(
        "Navigation",
        &[
            "status / s     show dashboard".to_string(),
            "rivals         competitor detail".to_string(),
            "board          objectives".to_string(),
            "region         regional mandate".to_string(),
            "report         last quarter detail".to_string(),
            "preview/quote  inspect command".to_string(),
            "Return         cycle screens / return".to_string(),
            "next / n / end finish quarter".to_string(),
            "continue       post-review play".to_string(),
            "save [name]    write save file".to_string(),
            "load [name]    restore save file".to_string(),
            "help / ?       command reference".to_string(),
            "quit / exit    leave game".to_string(),
        ],
        "Input Notes",
        &[
            "Money accepts 20000 or 20k.".to_string(),
            "Invalid amounts are rejected.".to_string(),
            "Build sizes are clamped to sane bounds.".to_string(),
            "Large rate increases above public tolerance are rejected.".to_string(),
            "Adjacent expansion creates integration work.".to_string(),
            "Saves live in ~/.electrification.".to_string(),
            "Use the dashboard guide for live costs.".to_string(),
        ],
    );
}

pub(super) fn print_status(game: &Game) {
    print_box(
        &format!("{} | {}", game.date_label(), game.player.name),
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
        &stable_panel_lines(project_lines(game), 5, "board for planning context"),
        "Commands",
        &stable_panel_lines(compact_command_lines(game), 5, "help for all commands"),
    );
}

pub(super) fn print_competitors(game: &Game) {
    print_box("Rival Overview", &rival_overview_lines(game));
    print_box("Rival Detail", &rival_detail_lines(game));
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

pub(super) fn print_last_report(game: &Game) {
    if let Some(report) = &game.last_report {
        print_report(report);
    } else {
        print_box(
            "Quarter Results",
            &["No quarter has been completed yet.".to_string()],
        );
    }
}

pub(super) fn print_notice(message: &str) {
    println!();
    print_box("Notice", &wrap_plain_text(message, content_width()));
}

pub(super) fn print_report(report: &QuarterReport) {
    let mut lines = vec![
        format!(
            "{} {} | {} {} | {} {} | {} {}",
            muted("Revenue"),
            styled(BOLD_GREEN, money(report.revenue)),
            muted("Costs"),
            styled(YELLOW, money(report.operating_cost)),
            muted("Interest"),
            styled(YELLOW, money(report.interest)),
            muted("Profit"),
            styled(profit_tone(report.profit), money(report.profit))
        ),
        format!(
            "{} {} gross, {} churn ({}) | {} {}",
            muted("Customers:"),
            styled(BOLD_GREEN, format!("+{:.0}", report.new_customers)),
            styled(RED, format!("-{:.0}", report.lost_customers)),
            styled(
                churn_tone(report.lost_customer_rate),
                format_churn_rate(report.lost_customer_rate)
            ),
            muted("Market share"),
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
    if !report.attributions.is_empty() {
        lines.push(styled(BOLD_CYAN, "Drivers"));
        for attribution in &report.attributions {
            for (index, wrapped) in wrap_plain_text(attribution, content_width().saturating_sub(4))
                .iter()
                .enumerate()
            {
                if index == 0 {
                    lines.push(format!("  {wrapped}"));
                } else {
                    lines.push(format!("    {wrapped}"));
                }
            }
        }
    }
    if !report.events.is_empty() {
        lines.push(styled(BOLD_CYAN, "Events"));
        for event in &report.events {
            for (index, wrapped) in wrap_plain_text(event, content_width().saturating_sub(7))
                .iter()
                .enumerate()
            {
                if index == 0 {
                    lines.push(format!("{} {wrapped}", styled(BOLD_YELLOW, "Event:")));
                } else {
                    lines.push(format!("       {wrapped}"));
                }
            }
        }
    }
    println!();
    print_box(&format!("{} Results", report.label), &lines);
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
            format!("{} use 'next' to complete the first quarter", muted("Next")),
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

    if let Some(event) = report.events.first() {
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

fn report_margin(report: &QuarterReport) -> f64 {
    if report.revenue > 0.0 {
        report.profit / report.revenue * 100.0
    } else {
        0.0
    }
}

pub(super) fn milestone_snapshot_lines(game: &Game) -> Vec<String> {
    let next = next_board_target(game);
    let coverage = game
        .player_last_interest_coverage()
        .unwrap_or(f64::INFINITY);
    vec![
        format!(
            "{} {}   {} {}",
            muted("Y5 review"),
            styled(BOLD_CYAN, quarters_remaining_label(game)),
            muted("Next"),
            styled(BOLD, next.label)
        ),
        format!(
            "{} {:.0}% | {} {:.0}% | {} <= {:.0}%",
            muted("share"),
            SHARE_TARGET * 100.0,
            muted("reliability"),
            RELIABILITY_TARGET * 100.0,
            muted("debt/assets"),
            LEVERAGE_LIMIT * 100.0
        ),
        format!(
            "{} {}   {} {}",
            muted("Rate support"),
            styled(
                rate_support_tone(game.player_rate_support_ratio()),
                format!("{:.0}%", game.player_rate_support_ratio() * 100.0)
            ),
            muted("Coverage"),
            styled(coverage_tone(coverage), format_coverage(coverage))
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
    let mut lines = vec![
        compact_action_line("next/n", "finish quarter | preview <command>"),
        compact_action_line(
            "build",
            format!(
                "gen 400 {} | lines 600 {}",
                project_quote_short("gen", 400.0),
                project_quote_short("lines", 600.0)
            ),
        ),
        compact_action_line(
            "operate",
            format!(
                "rate {:.1} | marketing/maint | hire",
                game.market.standard_rate_cents
            ),
        ),
        compact_action_line(
            "capital",
            format!(
                "borrow max {} | issue/buyback",
                styled(
                    borrowing_room_tone(game.borrowing_room()),
                    money(game.borrowing_room())
                )
            ),
        ),
    ];

    let opportunity = if let Some((index, price)) = cheapest_diligenced_competitor(game) {
        let rank = competitor_rank(game, index).unwrap_or(index + 1);
        if game.competitors.len() == 1 {
            "buy blocked: final rival protected".to_string()
        } else if game.acquisition_cooldown > 0 {
            format!("buy {rank} waits {}q", game.acquisition_cooldown)
        } else {
            format!("buy {rank} needs {}", money(price))
        }
    } else if let Some((index, estimate)) = cheapest_public_acquisition_target(game) {
        let rank = competitor_rank(game, index).unwrap_or(index + 1);
        format!("buy/diligence {rank} est {}", money(estimate))
    } else if should_show_adjacent_expansion(game) {
        format!("expand {}", adjacent_expansion_footer(game))
    } else {
        "rivals | board | report | save".to_string()
    };

    lines.push(compact_action_line("more", opportunity));
    lines
}

fn compact_action_line(command: impl std::fmt::Display, effect: impl std::fmt::Display) -> String {
    format!(
        "{}  {}",
        styled(BOLD_CYAN, format!("{command:<12}")),
        effect
    )
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

pub(super) fn print_outcome(outcome: &Outcome) {
    let (outcome_style, result) = match outcome.kind {
        OutcomeKind::Victory => (BOLD_GREEN, "victory"),
        OutcomeKind::Defeat => (BOLD_RED, "defeat"),
    };
    let mut lines = vec![styled(outcome_style, &outcome.headline)];
    lines.extend(wrap_plain_text(&outcome.details, content_width()));
    lines.push(format!(
        "{} {}",
        muted("Result:"),
        styled(outcome_style, result)
    ));
    if outcome.can_continue {
        lines.push("Type 'continue' to keep operating, or 'quit' to leave the game.".to_string());
    }

    println!();
    print_box("Outcome", &lines);
}

pub(super) fn scorecard_lines(game: &Game) -> Vec<String> {
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

struct ScorecardReviewTargets {
    share: f64,
    reliability: f64,
    leverage: f64,
    regional: bool,
}

fn scorecard_review_targets(game: &Game) -> ScorecardReviewTargets {
    if game.review_completed {
        ScorecardReviewTargets {
            share: REGIONAL_MANDATE_SHARE_TARGET,
            reliability: REGIONAL_MANDATE_RELIABILITY_TARGET,
            leverage: REGIONAL_MANDATE_LEVERAGE_LIMIT,
            regional: true,
        }
    } else {
        ScorecardReviewTargets {
            share: SHARE_TARGET,
            reliability: RELIABILITY_TARGET,
            leverage: LEVERAGE_LIMIT,
            regional: false,
        }
    }
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

fn ownership_tone(ownership: f64) -> &'static str {
    if ownership < 0.18 {
        RED
    } else if ownership < 0.26 {
        YELLOW
    } else {
        GREEN
    }
}

fn manager_tone(active: bool) -> &'static str {
    if active { GREEN } else { DIM }
}

fn format_coverage(coverage: f64) -> String {
    if coverage.is_infinite() {
        "no debt".to_string()
    } else {
        format!("{coverage:.1}x")
    }
}

pub(super) fn quarters_remaining_label(game: &Game) -> String {
    if game.review_completed {
        return "review complete".to_string();
    }

    let remaining = game.campaign_quarters.saturating_sub(game.quarter);
    match remaining {
        0 => "review due now".to_string(),
        1 => "1 quarter left".to_string(),
        _ => format!("{remaining} quarters left"),
    }
}

pub(super) fn board_overview_lines(game: &Game) -> Vec<String> {
    let next = next_board_target(game);
    vec![
        format!(
            "{} {}   {} {}",
            muted("Current"),
            styled(BOLD_CYAN, game.date_label()),
            muted("Next"),
            styled(BOLD, next.label)
        ),
        format!(
            "{} {} | {} {} | {} {}",
            muted("Final"),
            styled(BOLD_GREEN, format!("{:.0}% share", SHARE_TARGET * 100.0)),
            muted("reliability"),
            styled(BOLD_GREEN, format!("{:.0}%+", RELIABILITY_TARGET * 100.0)),
            muted("debt/assets"),
            styled(BOLD_GREEN, format!("<={:.0}%", LEVERAGE_LIMIT * 100.0))
        ),
        format!(
            "{} {} | {} {} | {} {}",
            muted("Now"),
            styled(
                share_tone(game.market_share()),
                format!("{:.0}% share", game.market_share() * 100.0)
            ),
            muted("reliability"),
            styled(
                reliability_tone(game.player.reliability),
                format!("{:.0}%", game.player.reliability * 100.0)
            ),
            muted("debt/assets"),
            styled(
                leverage_tone(game.player.debt_to_assets()),
                format!("{:.0}%", game.player.debt_to_assets() * 100.0)
            )
        ),
    ]
}

pub(super) fn board_milestone_lines(game: &Game) -> Vec<String> {
    let target = next_board_target(game);
    vec![
        format!(
            "{} {}",
            muted("Checkpoint"),
            styled(BOLD_CYAN, target.label)
        ),
        board_metric_line("Share", game.market_share(), target.share, true),
        board_metric_line(
            "Reliability",
            game.player.reliability,
            target.reliability,
            true,
        ),
        board_metric_line(
            "Debt/assets",
            game.player.debt_to_assets(),
            target.leverage,
            false,
        ),
    ]
}

pub(super) fn board_risk_lines(game: &Game) -> Vec<String> {
    let target = next_board_target(game);
    let mut lines = Vec::new();
    let share_gap = target.share - game.market_share();
    if share_gap > 0.06 {
        lines.push(signal_line(
            "Growth",
            YELLOW,
            format!("share needs {}", points(share_gap)),
        ));
    } else if share_gap <= 0.0 {
        lines.push(signal_line("Growth", GREEN, "share milestone already met"));
    } else {
        lines.push(signal_line("Growth", CYAN, "share gap is manageable"));
    }

    let headroom = game.player.capacity_headroom(&game.market);
    if headroom < 70.0 {
        lines.push(signal_line("Capacity", RED, "network may block growth"));
    } else if headroom < 140.0 {
        lines.push(signal_line("Capacity", YELLOW, "plan expansion soon"));
    } else {
        lines.push(signal_line("Capacity", GREEN, "room for customer growth"));
    }

    if game.player.reliability < target.reliability {
        lines.push(signal_line("Service", RED, "maintenance before review"));
    } else if game.player.reliability < target.reliability + 0.04 {
        lines.push(signal_line("Service", YELLOW, "thin reliability cushion"));
    } else {
        lines.push(signal_line("Service", GREEN, "reliability cushion intact"));
    }

    let leverage = game.player.debt_to_assets();
    if leverage > target.leverage {
        lines.push(signal_line("Capital", RED, "delever before checkpoint"));
    } else if game.borrowing_room() > 25_000.0 {
        lines.push(signal_line("Capital", GREEN, "borrowing room available"));
    } else {
        lines.push(signal_line("Capital", YELLOW, "funding room is limited"));
    }

    let rate_support = game.player_rate_support_ratio();
    if rate_support < REVIEW_MIN_RATE_SUPPORT_RATIO {
        lines.push(signal_line(
            "Earnings",
            RED,
            format!(
                "{:.0}% of break-even; rate path is not durable",
                rate_support * 100.0
            ),
        ));
    } else if let Some(coverage) = game.player_last_interest_coverage() {
        if coverage < REVIEW_MIN_INTEREST_COVERAGE {
            lines.push(signal_line(
                "Earnings",
                RED,
                format!("interest coverage only {}", format_coverage(coverage)),
            ));
        } else if coverage < 1.30 {
            lines.push(signal_line(
                "Earnings",
                YELLOW,
                format!("thin interest coverage at {}", format_coverage(coverage)),
            ));
        } else {
            lines.push(signal_line(
                "Earnings",
                GREEN,
                "rate and debt service look durable",
            ));
        }
    } else {
        lines.push(signal_line(
            "Earnings",
            CYAN,
            "first operating report pending",
        ));
    }

    lines
}

pub(super) fn board_path_lines(game: &Game) -> Vec<String> {
    board_targets()
        .iter()
        .map(|target| {
            let status = if game.quarter >= target.quarter {
                let met = game.market_share() >= target.share
                    && game.player.reliability >= target.reliability
                    && game.player.debt_to_assets() <= target.leverage;
                if met {
                    styled(BOLD_GREEN, "met")
                } else {
                    styled(BOLD_RED, "missed")
                }
            } else if target.quarter == next_board_target(game).quarter {
                styled(BOLD_CYAN, "next")
            } else {
                styled(DIM, "later")
            };
            format!(
                "{} {:<10} | share {:.0}% | rel {:.0}% | debt/assets <= {:.0}%",
                status,
                target.label,
                target.share * 100.0,
                target.reliability * 100.0,
                target.leverage * 100.0
            )
        })
        .collect()
}

pub(super) fn should_show_regional_status(game: &Game) -> bool {
    !game.regional_mandate_completed && game.quarter < REGIONAL_MANDATE_QUARTER
}

#[cfg(test)]
pub(super) fn regional_status_lines(game: &Game) -> Vec<String> {
    let pending = adjacent_pending_quarters(game);
    let mut lines = vec![
        format!(
            "{} {} / {}   {} {}",
            muted("Territories"),
            styled(
                expansion_tone(game.adjacent_expansions),
                game.adjacent_expansions
            ),
            muted(REGIONAL_MANDATE_EXPANSION_TARGET),
            muted("mandate"),
            styled(
                if game.regional_mandate_met() {
                    BOLD_GREEN
                } else {
                    BOLD_CYAN
                },
                regional_due_label(game)
            )
        ),
        format!(
            "{} {}   {} {}",
            muted("Regional share"),
            styled(
                share_tone(game.market_share()),
                format!(
                    "{:.0}% / {:.0}%",
                    game.market_share() * 100.0,
                    REGIONAL_MANDATE_SHARE_TARGET * 100.0
                )
            ),
            muted("Integration"),
            styled(
                regional_integration_tone(game.regional_integration),
                format!("{:.0} pts", game.regional_integration * 100.0)
            )
        ),
    ];

    if let Some(quarters) = pending {
        lines.push(format!(
            "{} {}",
            muted("Pending"),
            styled(YELLOW, format!("adjacent territory opens in {quarters}q"))
        ));
    } else if game.regional_integration > 0.12 {
        lines.push(format!(
            "{} {}",
            muted("Follow-up"),
            styled(
                YELLOW,
                "marketing and maintenance reduce regional integration burden"
            )
        ));
    } else if game.review_completed && !game.regional_mandate_completed {
        lines.push(format!(
            "{} {}",
            muted("Next step"),
            styled(CYAN, "expand and defend service quality before Year 10")
        ));
    } else if !game.review_completed {
        lines.push(format!(
            "{} {}",
            muted("Plan ahead"),
            styled(
                DIM,
                format!(
                    "Y5 review unlocks adjacent expansion (need {:.0}% share, {:.0}% reliability, debt/assets <= {:.0}%)",
                    REGIONAL_MANDATE_SHARE_TARGET * 100.0,
                    REGIONAL_MANDATE_RELIABILITY_TARGET * 100.0,
                    REGIONAL_MANDATE_LEVERAGE_LIMIT * 100.0
                )
            )
        ));
    }

    lines
}

pub(super) fn regional_mandate_lines(game: &Game) -> Vec<String> {
    vec![
        format!(
            "{} {}   {} {}",
            muted("Due"),
            styled(BOLD_CYAN, regional_due_label(game)),
            muted("Status"),
            styled(
                if game.regional_mandate_met() {
                    BOLD_GREEN
                } else {
                    YELLOW
                },
                if game.regional_mandate_met() {
                    "on mandate"
                } else {
                    "not yet secure"
                }
            )
        ),
        board_metric_line(
            "Share",
            game.market_share(),
            REGIONAL_MANDATE_SHARE_TARGET,
            true,
        ),
        board_metric_line(
            "Reliability",
            game.player.reliability,
            REGIONAL_MANDATE_RELIABILITY_TARGET,
            true,
        ),
        board_metric_line(
            "Debt/assets",
            game.player.debt_to_assets(),
            REGIONAL_MANDATE_LEVERAGE_LIMIT,
            false,
        ),
        format!(
            "{} {} / {}   {} {}",
            styled(BOLD, "Territories"),
            styled(
                expansion_tone(game.adjacent_expansions),
                game.adjacent_expansions
            ),
            muted(REGIONAL_MANDATE_EXPANSION_TARGET),
            styled(BOLD, "Integration"),
            styled(
                regional_integration_tone(game.regional_integration),
                format!("{:.0} pts", game.regional_integration * 100.0)
            )
        ),
    ]
}

fn regional_due_label(game: &Game) -> String {
    if game.regional_mandate_completed {
        return "mandate reviewed".to_string();
    }
    if game.quarter >= REGIONAL_MANDATE_QUARTER {
        return "review due now".to_string();
    }
    let remaining = game.regional_mandate_due_in();
    if remaining == 1 {
        "Year 10, 1 quarter left".to_string()
    } else {
        format!("Year 10, {remaining} quarters left")
    }
}

#[cfg(test)]
fn adjacent_pending_quarters(game: &Game) -> Option<u32> {
    game.pending_projects.iter().find_map(|project| {
        matches!(project.kind, crate::ProjectKind::AdjacentTerritory { .. })
            .then_some(project.quarters_remaining)
    })
}

fn expansion_tone(expansions: u32) -> &'static str {
    if expansions >= REGIONAL_MANDATE_EXPANSION_TARGET {
        BOLD_GREEN
    } else if expansions > 0 {
        YELLOW
    } else {
        RED
    }
}

fn regional_integration_tone(value: f64) -> &'static str {
    if value <= 0.12 {
        BOLD_GREEN
    } else if value <= 0.28 {
        YELLOW
    } else {
        RED
    }
}

fn integration_strain_label(value: f64) -> &'static str {
    if value >= 0.50 {
        "severe"
    } else if value >= 0.30 {
        "high"
    } else if value >= 0.12 {
        "moderate"
    } else if value > 0.01 {
        "light"
    } else {
        "clear"
    }
}

fn integration_strain_tone(value: f64) -> &'static str {
    if value >= 0.50 {
        RED
    } else if value >= 0.30 {
        BOLD_YELLOW
    } else if value >= 0.08 {
        YELLOW
    } else {
        GREEN
    }
}

pub(super) fn board_metric_line(
    label: &str,
    current: f64,
    target: f64,
    higher_is_better: bool,
) -> String {
    let met = if higher_is_better {
        current >= target
    } else {
        current <= target
    };
    let tone = if met { BOLD_GREEN } else { BOLD_YELLOW };
    let note = if met {
        if higher_is_better {
            format!("{} cushion", points(current - target))
        } else {
            format!("{} room", points(target - current))
        }
    } else if higher_is_better {
        format!("{} short", points(target - current))
    } else {
        format!("{} over", points(current - target))
    };
    format!(
        "{} {} / {}   {}",
        styled(BOLD, label),
        styled(tone, format!("{:.0}%", current * 100.0)),
        muted(format!("{:.0}%", target * 100.0)),
        muted(note)
    )
}

pub(super) fn next_board_target(game: &Game) -> BoardTarget {
    if game.review_completed {
        return BoardTarget {
            label: "Continuation",
            quarter: game.quarter,
            share: SHARE_TARGET,
            reliability: RELIABILITY_TARGET,
            leverage: LEVERAGE_LIMIT,
        };
    }

    board_targets()
        .into_iter()
        .find(|target| game.quarter < target.quarter)
        .unwrap_or(BoardTarget {
            label: "Final",
            quarter: game.campaign_quarters,
            share: SHARE_TARGET,
            reliability: RELIABILITY_TARGET,
            leverage: LEVERAGE_LIMIT,
        })
}

pub(super) fn board_targets() -> [BoardTarget; 4] {
    [
        BoardTarget {
            label: "Year 2",
            quarter: 8,
            share: 0.28,
            reliability: 0.74,
            leverage: 1.05,
        },
        BoardTarget {
            label: "Year 3",
            quarter: 12,
            share: 0.34,
            reliability: 0.74,
            leverage: 1.00,
        },
        BoardTarget {
            label: "Year 4",
            quarter: 16,
            share: 0.40,
            reliability: 0.73,
            leverage: 0.98,
        },
        BoardTarget {
            label: "Final",
            quarter: 20,
            share: SHARE_TARGET,
            reliability: RELIABILITY_TARGET,
            leverage: LEVERAGE_LIMIT,
        },
    ]
}

pub(super) fn points(value: f64) -> String {
    format!("{:.0} pts", value.abs() * 100.0)
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
                format!("{:.1}%", game.player_ownership() * 100.0)
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
            muted("Firm"),
            styled(
                reserve_tone(firm_generation_reserve),
                format!("{:+.0}", firm_generation_reserve)
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
    if firm_generation_reserve < 15.0 {
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

    if game.player.reliability < 0.74 {
        push_line(
            78,
            signal_line("Reliability", RED, "below target; fund maintenance"),
        );
    } else if game.player.reliability > 0.88 {
        push_line(
            25,
            signal_line("Reliability", GREEN, "strong; expansion can lead"),
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
            "marketing 4000, maint 6000, or hire maint/marketing"
        },
    ));
    if should_show_adjacent_expansion(game) {
        lines.push(action_line("expand", adjacent_expansion_footer(game)));
    }
    lines.push(action_line(
        "rate | capital",
        if dashboard_width() >= 112 {
            format!(
                "rate {:.1} near market; borrow max {} at {}; issue/buyback",
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
                "rate {:.1}; borrow max {} @ {}; equity",
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
