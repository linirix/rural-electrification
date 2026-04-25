use std::io::{self, Write};

use crate::sim::{
    DISTRIBUTION_PROJECT_CAPACITY, Decision, GENERATION_PROJECT_CAPACITY_MWH, Game, Outcome,
    OutcomeKind, QuarterReport, ShockKind, distribution_project_cost,
    distribution_project_duration, generation_project_cost, generation_project_duration, money,
};

const SCREEN_WIDTH: usize = 88;
const CONTENT_WIDTH: usize = SCREEN_WIDTH - 4;
const COLUMN_GAP: usize = 2;
const COLUMN_WIDTH: usize = (SCREEN_WIDTH - COLUMN_GAP) / 2;
const SHARE_TARGET: f64 = 0.45;
const RELIABILITY_TARGET: f64 = 0.72;
const LEVERAGE_LIMIT: f64 = 0.95;
const RESET: &str = "\x1B[0m";
const BOLD: &str = "\x1B[1m";
const DIM: &str = "\x1B[2m";
const RED: &str = "\x1B[31m";
const GREEN: &str = "\x1B[32m";
const YELLOW: &str = "\x1B[33m";
const CYAN: &str = "\x1B[36m";
const BOLD_RED: &str = "\x1B[1;31m";
const BOLD_GREEN: &str = "\x1B[1;32m";
const BOLD_YELLOW: &str = "\x1B[1;33m";
const BOLD_CYAN: &str = "\x1B[1;36m";

pub fn run() -> io::Result<()> {
    let mut game = Game::new();

    clear_screen();
    println!("{}", styled(BOLD_CYAN, "Electrification"));
    println!(
        "You are general manager of {} in {}.",
        styled(BOLD, &game.player.name),
        game.market.territory
    );
    println!("Build a dominant, solvent, reliable utility before the market review ends.");
    println!("Type 'help' for commands.\n");
    print_status(&game);

    loop {
        print!("\n{}> ", game.date_label());
        io::stdout().flush()?;

        let mut line = String::new();
        if io::stdin().read_line(&mut line)? == 0 {
            break;
        }

        let command = line.trim();
        if command.is_empty() {
            continue;
        }

        match handle_command(&mut game, command) {
            CommandResult::Continue(message) => {
                clear_screen();
                print_status(&game);
                if !message.is_empty() {
                    print_notice(&message);
                }
            }
            CommandResult::Advanced(report) => {
                clear_screen();
                print_status(&game);
                print_report(&report);
                if let Some(outcome) = &game.outcome {
                    print_outcome(outcome);
                    break;
                }
            }
            CommandResult::ShowStatus => {
                clear_screen();
                print_status(&game);
            }
            CommandResult::ShowHelp => {
                clear_screen();
                print_help();
            }
            CommandResult::ShowRivals => {
                clear_screen();
                print_competitors(&game);
            }
            CommandResult::ShowBoard => {
                clear_screen();
                print_board(&game);
            }
            CommandResult::Quit => break,
        }
    }

    Ok(())
}

enum CommandResult {
    Continue(String),
    Advanced(QuarterReport),
    ShowStatus,
    ShowHelp,
    ShowRivals,
    ShowBoard,
    Quit,
}

#[derive(Clone, Copy)]
struct BoardTarget {
    label: &'static str,
    quarter: u32,
    share: f64,
    reliability: f64,
    leverage: f64,
}

fn handle_command(game: &mut Game, command: &str) -> CommandResult {
    let parts = command.split_whitespace().collect::<Vec<_>>();
    let Some(first) = parts.first().copied() else {
        return CommandResult::Continue(String::new());
    };

    match first {
        "help" | "?" => CommandResult::ShowHelp,
        "status" | "s" => CommandResult::ShowStatus,
        "next" | "n" | "end" => CommandResult::Advanced(game.advance_quarter()),
        "quit" | "exit" => CommandResult::Quit,
        "build" => {
            let decision = match parts.get(1).copied() {
                Some("gen") | Some("generator") | Some("generation") => {
                    let capacity_mwh = match optional_number(parts.get(2).copied()) {
                        Ok(Some(size)) => size,
                        Ok(None) => GENERATION_PROJECT_CAPACITY_MWH,
                        Err(_) => {
                            return CommandResult::Continue(
                                "Use 'build gen' or 'build gen 300' where the number is MWh/q."
                                    .to_string(),
                            );
                        }
                    };
                    Decision::BuildGeneration { capacity_mwh }
                }
                Some("lines") | Some("line") | Some("distribution") | Some("wires") => {
                    let customer_capacity = match optional_number(parts.get(2).copied()) {
                        Ok(Some(size)) => size,
                        Ok(None) => DISTRIBUTION_PROJECT_CAPACITY,
                        Err(_) => {
                            return CommandResult::Continue(
                                "Use 'build lines' or 'build lines 500' where the number is customer capacity."
                                    .to_string(),
                            );
                        }
                    };
                    Decision::BuildDistribution { customer_capacity }
                }
                _ => {
                    return CommandResult::Continue(
                        "Build what? Try 'build gen [MWh]' or 'build lines [customers]'."
                            .to_string(),
                    );
                }
            };
            apply(game, decision)
        }
        "marketing" | "market" | "advertise" => {
            let spend = parts
                .get(1)
                .and_then(|value| parse_money(value))
                .unwrap_or(4_000.0);
            apply(game, Decision::Marketing { spend })
        }
        "stock" | "equity" => match parts.get(1).copied() {
            Some("issue" | "sell") => {
                let amount = parts
                    .get(2)
                    .and_then(|value| parse_money(value))
                    .unwrap_or(20_000.0);
                apply(game, Decision::IssueStock { amount })
            }
            Some("buyback" | "repurchase" | "buy") => {
                let amount = parts
                    .get(2)
                    .and_then(|value| parse_money(value))
                    .unwrap_or(10_000.0);
                apply(game, Decision::BuyBackStock { amount })
            }
            Some(value) if parse_money(value).is_some() => {
                let amount = parse_money(value).unwrap_or(20_000.0);
                apply(game, Decision::IssueStock { amount })
            }
            Some(_) => CommandResult::Continue(
                "Use 'stock issue 20000', 'stock buyback 10000', or 'stock 20000'.".to_string(),
            ),
            _ => {
                let amount = parts
                    .get(1)
                    .and_then(|value| parse_money(value))
                    .unwrap_or(20_000.0);
                apply(game, Decision::IssueStock { amount })
            }
        },
        "buyback" | "repurchase" => {
            let amount = parts
                .get(1)
                .and_then(|value| parse_money(value))
                .unwrap_or(10_000.0);
            apply(game, Decision::BuyBackStock { amount })
        }
        "debt" | "borrow" | "loan" => {
            if first == "debt" && matches!(parts.get(1).copied(), Some("repay" | "pay" | "down")) {
                let amount = parts
                    .get(2)
                    .and_then(|value| parse_money(value))
                    .unwrap_or(10_000.0);
                return apply(game, Decision::RepayDebt { amount });
            }
            let amount = parts
                .get(1)
                .and_then(|value| parse_money(value))
                .unwrap_or(20_000.0);
            apply(game, Decision::Borrow { amount })
        }
        "repay" | "paydown" => {
            let amount = parts
                .get(1)
                .and_then(|value| parse_money(value))
                .unwrap_or(10_000.0);
            apply(game, Decision::RepayDebt { amount })
        }
        "buy" | "acquire" => {
            let Some(index) = parts.get(1).and_then(|value| value.parse::<usize>().ok()) else {
                return CommandResult::Continue("Use 'buy 1', 'buy 2', etc.".to_string());
            };
            if index == 0 {
                return CommandResult::Continue("Competitor numbers start at 1.".to_string());
            }
            apply(
                game,
                Decision::Acquire {
                    competitor_index: index - 1,
                },
            )
        }
        "rate" => {
            let delta = match parse_rate_delta(&parts, game.player.rate_cents) {
                Ok(Some(delta)) => delta,
                Ok(None) => {
                    return CommandResult::Continue(format!(
                        "Current rate is {:.1}c/kWh. Use 'rate up 2', 'rate down 1', or 'rate 10.0'.",
                        game.player.rate_cents
                    ));
                }
                Err(message) => return CommandResult::Continue(message),
            };
            apply(game, Decision::AdjustRate { delta_cents: delta })
        }
        "maintenance" | "maint" | "maintain" | "reliability" => {
            let spend = parts
                .get(1)
                .and_then(|value| parse_money(value))
                .unwrap_or(4_500.0);
            apply(game, Decision::Maintenance { spend })
        }
        "competitors" | "rivals" => CommandResult::ShowRivals,
        "board" | "goals" | "objectives" | "milestones" => CommandResult::ShowBoard,
        _ => CommandResult::Continue(format!("Unknown command '{first}'. Type 'help'.")),
    }
}

fn apply(game: &mut Game, decision: Decision) -> CommandResult {
    match game.apply_decision(decision) {
        Ok(message) => CommandResult::Continue(message),
        Err(error) => CommandResult::Continue(format!("Cannot do that: {error}")),
    }
}

fn print_help() {
    println!("{}", styled(BOLD_CYAN, "Electrification Command Reference"));
    print_box_pair(
        "Build + Operate",
        &[
            "build gen [MWh]".to_string(),
            "build lines [customers]".to_string(),
            "marketing [amount]".to_string(),
            "maintenance [amount]  alias: maint".to_string(),
            "rate up|down [cents]".to_string(),
            "rate 10.0             set target".to_string(),
        ],
        "Capital",
        &[
            "stock issue [amount]".to_string(),
            "stock [amount]        issue stock".to_string(),
            "buyback [amount]".to_string(),
            "debt [amount]         borrow".to_string(),
            "repay [amount]        pay debt down".to_string(),
            "buy <number>          acquire rival".to_string(),
        ],
    );
    print_box_pair(
        "Navigation",
        &[
            "status     show dashboard".to_string(),
            "rivals     competitor detail".to_string(),
            "board      objectives and milestones".to_string(),
            "help       command reference".to_string(),
            "next       finish quarter".to_string(),
            "n          finish quarter".to_string(),
            "quit       leave game".to_string(),
        ],
        "Input Notes",
        &[
            "Money accepts 20000 or 20k.".to_string(),
            "Build sizes are clamped to sane bounds.".to_string(),
            "Rate changes are in cents per kWh.".to_string(),
            "Use the dashboard guide for live costs.".to_string(),
        ],
    );
}

fn print_status(game: &Game) {
    print_box(
        &format!("{} | {}", game.date_label(), game.player.name),
        &scorecard_lines(game),
    );
    print_box_pair(
        "Capital",
        &financial_lines(game),
        "Network",
        &operation_lines(game),
    );
    print_box_pair(
        "Market",
        &market_lines(game),
        "Rivals",
        &dashboard_competitor_lines(game),
    );
    print_box_pair(
        "Projects",
        &project_lines(game),
        "Signals",
        &signal_lines(game),
    );
    print_box("Decision Guide", &action_effect_lines(game));
}

fn print_competitors(game: &Game) {
    print_box("Rival Overview", &rival_overview_lines(game));
    print_box("Rival Detail", &rival_detail_lines(game));
}

fn print_board(game: &Game) {
    print_box("Board Objectives", &board_overview_lines(game));
    print_box_pair(
        "Next Milestone",
        &board_milestone_lines(game),
        "Risk Notes",
        &board_risk_lines(game),
    );
    print_box("Milestone Path", &board_path_lines(game));
}

fn print_notice(message: &str) {
    println!();
    print_box("Notice", &wrap_plain_text(message, CONTENT_WIDTH));
}

fn print_report(report: &QuarterReport) {
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
            for (index, wrapped) in wrap_plain_text(attribution, CONTENT_WIDTH.saturating_sub(4))
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
            for (index, wrapped) in wrap_plain_text(event, CONTENT_WIDTH.saturating_sub(7))
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

fn print_outcome(outcome: &Outcome) {
    let (outcome_style, result) = match outcome.kind {
        OutcomeKind::Victory => (BOLD_GREEN, "victory"),
        OutcomeKind::Defeat => (BOLD_RED, "defeat"),
    };
    let mut lines = vec![styled(outcome_style, &outcome.headline)];
    lines.extend(wrap_plain_text(&outcome.details, CONTENT_WIDTH));
    lines.push(format!(
        "{} {}",
        muted("Result:"),
        styled(outcome_style, result)
    ));

    println!();
    print_box("Outcome", &lines);
}

fn parse_money(value: &str) -> Option<f64> {
    let trimmed = value.trim().trim_start_matches('$');
    if let Some(base) = trimmed
        .strip_suffix('k')
        .or_else(|| trimmed.strip_suffix('K'))
    {
        base.parse::<f64>().ok().map(|number| number * 1_000.0)
    } else {
        trimmed.parse::<f64>().ok()
    }
}

fn parse_number(value: &str) -> Option<f64> {
    let trimmed = value.trim().trim_start_matches('+');
    if let Some(base) = trimmed
        .strip_suffix('k')
        .or_else(|| trimmed.strip_suffix('K'))
    {
        base.parse::<f64>().ok().map(|number| number * 1_000.0)
    } else {
        trimmed.parse::<f64>().ok()
    }
}

fn optional_number(value: Option<&str>) -> Result<Option<f64>, ()> {
    let Some(value) = value else {
        return Ok(None);
    };
    let Some(number) = parse_number(value) else {
        return Err(());
    };
    if number.is_finite() && number > 0.0 {
        Ok(Some(number))
    } else {
        Err(())
    }
}

fn parse_rate_delta(parts: &[&str], current_rate: f64) -> Result<Option<f64>, String> {
    let Some(argument) = parts.get(1).copied() else {
        return Ok(None);
    };

    match argument {
        "up" | "+" => {
            let amount = rate_amount(parts.get(2).copied())?;
            Ok(Some(amount))
        }
        "down" | "-" => {
            let amount = rate_amount(parts.get(2).copied())?;
            Ok(Some(-amount))
        }
        value if value.starts_with('+') && value.len() > 1 => {
            let amount = parse_rate_number(&value[1..])?;
            if amount <= 0.0 {
                Err("Use a positive rate change, like 'rate +2'.".to_string())
            } else {
                Ok(Some(amount))
            }
        }
        value if value.starts_with('-') && value.len() > 1 => {
            let amount = parse_rate_number(&value[1..])?;
            if amount <= 0.0 {
                Err("Use a positive rate change, like 'rate -1'.".to_string())
            } else {
                Ok(Some(-amount))
            }
        }
        value => {
            let target = parse_rate_number(value)?;
            Ok(Some(target - current_rate))
        }
    }
}

fn rate_amount(value: Option<&str>) -> Result<f64, String> {
    match value {
        Some(value) => {
            let amount = parse_rate_number(value)?;
            if amount > 0.0 {
                Ok(amount)
            } else {
                Err("Use a positive rate change, like 'rate up 2'.".to_string())
            }
        }
        None => Ok(0.5),
    }
}

fn parse_rate_number(value: &str) -> Result<f64, String> {
    match value.parse::<f64>() {
        Ok(number) if number.is_finite() => Ok(number),
        _ => Err("Use 'rate up 2', 'rate down 1', or 'rate 10.0'.".to_string()),
    }
}

fn scorecard_lines(game: &Game) -> Vec<String> {
    let share = game.market_share();
    let reliability = game.player.reliability;
    let leverage = game.player.debt_to_assets();

    vec![
        format!(
            "{} {} | {} {} | {} {}",
            muted("Market"),
            styled(BOLD, &game.market.territory),
            muted("Review"),
            styled(BOLD_CYAN, quarters_remaining_label(game)),
            muted("Rate"),
            styled(BOLD_CYAN, format!("{:.1}c/kWh", game.player.rate_cents))
        ),
        score_line(
            "Share",
            share,
            SHARE_TARGET,
            share_tone(share),
            format!("{:.0}% target", SHARE_TARGET * 100.0),
            if share >= SHARE_TARGET {
                "target met".to_string()
            } else {
                format!("{:.0} pts to go", (SHARE_TARGET - share) * 100.0)
            },
        ),
        score_line(
            "Reliability",
            reliability,
            RELIABILITY_TARGET,
            reliability_tone(reliability),
            format!("{:.0}% target", RELIABILITY_TARGET * 100.0),
            if reliability >= RELIABILITY_TARGET {
                format!(
                    "{:.0} pts above",
                    (reliability - RELIABILITY_TARGET) * 100.0
                )
            } else {
                format!(
                    "{:.0} pts short",
                    (RELIABILITY_TARGET - reliability) * 100.0
                )
            },
        ),
        score_line(
            "Debt/assets",
            leverage,
            LEVERAGE_LIMIT,
            leverage_tone(leverage),
            format!("{:.0}% limit", LEVERAGE_LIMIT * 100.0),
            if leverage <= LEVERAGE_LIMIT {
                format!("{:.0} pts room", (LEVERAGE_LIMIT - leverage) * 100.0)
            } else {
                format!("{:.0} pts over", (leverage - LEVERAGE_LIMIT) * 100.0)
            },
        ),
    ]
}

fn score_line(
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

fn progress_meter(current: f64, reference: f64, tone: &str) -> String {
    let width = 16;
    let filled = ((current / reference.max(0.01)).clamp(0.0, 1.0) * width as f64).round() as usize;
    styled(
        tone,
        format!("[{}{}]", "=".repeat(filled), ".".repeat(width - filled)),
    )
}

fn quarters_remaining_label(game: &Game) -> String {
    let remaining = game.campaign_quarters.saturating_sub(game.quarter);
    match remaining {
        0 => "review due now".to_string(),
        1 => "1 quarter left".to_string(),
        _ => format!("{remaining} quarters left"),
    }
}

fn board_overview_lines(game: &Game) -> Vec<String> {
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

fn board_milestone_lines(game: &Game) -> Vec<String> {
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

fn board_risk_lines(game: &Game) -> Vec<String> {
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

    lines
}

fn board_path_lines(game: &Game) -> Vec<String> {
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

fn board_metric_line(label: &str, current: f64, target: f64, higher_is_better: bool) -> String {
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

fn next_board_target(game: &Game) -> BoardTarget {
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

fn board_targets() -> [BoardTarget; 4] {
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

fn points(value: f64) -> String {
    format!("{:.0} pts", value.abs() * 100.0)
}

fn financial_lines(game: &Game) -> Vec<String> {
    let borrowing_room = game.borrowing_room();
    let leverage = game.player.debt_to_assets();
    let debt_rate = game.player_annual_interest_rate();
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
            "{} {}   {} {}",
            muted("Stock"),
            styled(BOLD_CYAN, format!("${:.2}", game.player.stock_price)),
            muted("Market cap"),
            styled(BOLD, money(game.player.market_cap()))
        ),
        format!(
            "{} {}   {} {}",
            muted("Credit"),
            styled(macro_credit_tone(game), game.macro_state.credit_label()),
            muted("Debt cap"),
            styled(
                debt_cap_tone(game.macro_state.borrowing_limit_ratio()),
                format!("{:.0}%", game.macro_state.borrowing_limit_ratio() * 100.0)
            )
        ),
    ];

    if let Some(report) = &game.last_report {
        let margin = if report.revenue > 0.0 {
            report.profit / report.revenue * 100.0
        } else {
            0.0
        };
        lines.push(format!(
            "{} {}   {} {}",
            muted("Last profit"),
            styled(profit_tone(report.profit), money(report.profit)),
            muted("margin"),
            styled(profit_tone(margin), format!("{:.1}%", margin))
        ));
    } else {
        lines.push(format!("{} no operating report yet", muted("Last q")));
    }

    lines
}

fn operation_lines(game: &Game) -> Vec<String> {
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
            "{} {}   {} {}",
            muted("Utilization"),
            styled(
                utilization_tone(game.player.utilization(&game.market)),
                format!("{:.0}%", game.player.utilization(&game.market) * 100.0)
            ),
            muted("Avg use"),
            styled(
                BOLD,
                format!("{:.2} MWh/cust", game.market.avg_mwh_per_customer)
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
    ]
}

fn market_lines(game: &Game) -> Vec<String> {
    vec![
        format!(
            "{} {} / {}",
            muted("Serviceable"),
            styled(BOLD, format!("{:.0}", game.serviceable_customers())),
            styled(
                DIM,
                format!("{:.0} addressable", game.market.addressable_customers)
            )
        ),
        format!(
            "{} {}   {} {}",
            muted("Connected"),
            styled(BOLD, format!("{:.0}", game.total_connected_customers())),
            muted("Your share"),
            styled(
                share_tone(game.market_share()),
                format!("{:.0}%", game.market_share() * 100.0)
            )
        ),
        format!(
            "{} {}   {} {}",
            muted("Electrified"),
            styled(
                BOLD_CYAN,
                format!("{:.1}%", game.market.electrification * 100.0)
            ),
            muted("Address share"),
            styled(BOLD, format!("{:.1}%", game.addressable_share() * 100.0))
        ),
        format!(
            "{} {}   {} {}",
            muted("Demand"),
            styled(
                demand_tone(game.macro_state.demand_index),
                game.macro_state.demand_label()
            ),
            muted("Costs"),
            styled(
                cost_pressure_tone(game.macro_state.cost_pressure),
                game.macro_state.cost_label()
            )
        ),
    ]
}

fn competitor_lines(game: &Game) -> Vec<String> {
    if game.competitors.is_empty() {
        return vec!["No active rivals.".to_string()];
    }

    let mut lines = Vec::new();
    for (index, competitor) in game.competitors.iter().enumerate() {
        let (acquisition_note, acquisition_tone) = acquisition_status(game, index);

        let rate_tone = if competitor.rate_cents + 0.4 < game.player.rate_cents {
            RED
        } else if competitor.rate_cents > game.player.rate_cents + 0.4 {
            GREEN
        } else {
            CYAN
        };
        let rate_gap = competitor.rate_cents - game.player.rate_cents;
        lines.push(format!(
            "{} {}  {} acct  {}",
            styled(DIM, index + 1),
            styled(BOLD, shorten_plain(&competitor.name, 15)),
            styled(BOLD, format!("{:.0}", competitor.customers)),
            styled(rate_tone, format!("{:.1}c", competitor.rate_cents)),
        ));
        lines.push(format!(
            "  {} {}  {} {}  {} {}",
            muted("rel"),
            styled(
                reliability_tone(competitor.reliability),
                format!("{:.0}%", competitor.reliability * 100.0)
            ),
            muted("vs you"),
            styled(rate_tone, format!("{:+.1}c", rate_gap)),
            muted("buy"),
            styled(acquisition_tone, acquisition_note)
        ));
    }
    lines
}

fn rival_overview_lines(game: &Game) -> Vec<String> {
    vec![
        format!(
            "{} {}   {} {}   {} {}",
            muted("Your rate"),
            styled(BOLD_CYAN, format!("{:.1}c", game.player.rate_cents)),
            muted("Market avg"),
            styled(CYAN, format!("{:.1}c", market_average_rate(game))),
            muted("Rivals"),
            styled(BOLD, game.competitors.len())
        ),
        format!(
            "{} {}   {} {}",
            muted("Your share"),
            styled(
                share_tone(game.market_share()),
                format!("{:.0}%", game.market_share() * 100.0)
            ),
            muted("Connected market"),
            styled(
                BOLD,
                format!("{:.0} accounts", game.total_connected_customers())
            )
        ),
        "Use buy <number> to acquire; final independent rival is protected.".to_string(),
    ]
}

fn rival_detail_lines(game: &Game) -> Vec<String> {
    if game.competitors.is_empty() {
        return vec!["No active rivals.".to_string()];
    }

    let total_connected = game.total_connected_customers().max(1.0);
    let mut lines = Vec::new();
    for (index, competitor) in game.competitors.iter().enumerate() {
        let share = competitor.customers / total_connected;
        let customer_delta = competitor.customers - competitor.last_quarter_customers;
        let rate_gap = competitor.rate_cents - game.player.rate_cents;
        let rate_tone = if rate_gap <= -0.4 {
            RED
        } else if rate_gap >= 0.4 {
            GREEN
        } else {
            CYAN
        };
        let headroom = competitor.capacity_headroom(&game.market);
        let firm_reserve = competitor.firm_generation_reserve_mwh(&game.market);
        let (acquisition_note, acquisition_tone) = acquisition_status(game, index);

        lines.push(format!(
            "{}. {}",
            styled(DIM, index + 1),
            styled(BOLD, &competitor.name)
        ));
        lines.push(format!(
            "   {} {} ({})   {} {}   {} {} ({})",
            muted("customers"),
            styled(BOLD, format!("{:.0}", competitor.customers)),
            styled(
                rival_customer_delta_tone(customer_delta),
                signed_count(customer_delta)
            ),
            muted("share"),
            styled(BOLD, format!("{:.0}%", share * 100.0)),
            muted("rate"),
            styled(rate_tone, format!("{:.1}c", competitor.rate_cents)),
            styled(rate_tone, format!("{:+.1}c vs you", rate_gap))
        ));
        lines.push(format!(
            "   {} {}   {} {}   {} {}",
            muted("reliability"),
            styled(
                reliability_tone(competitor.reliability),
                format!("{:.0}%", competitor.reliability * 100.0)
            ),
            muted("headroom"),
            styled(headroom_tone(headroom), format!("{:.0}", headroom.max(0.0))),
            muted("firm reserve"),
            styled(
                reserve_tone(firm_reserve),
                format!("{:+.0} MWh", firm_reserve)
            )
        ));
        lines.push(format!(
            "   {} {}   {} {}   {} {}",
            muted("cash"),
            styled(cash_tone(competitor.cash), money(competitor.cash)),
            muted("debt/assets"),
            styled(
                leverage_tone(competitor.debt_to_assets()),
                format!("{:.0}%", competitor.debt_to_assets() * 100.0)
            ),
            muted("buy"),
            styled(acquisition_tone, acquisition_note)
        ));
        if let Some(event) = recent_rival_event(game, &competitor.name) {
            for (line_index, wrapped) in wrap_plain_text(&event, CONTENT_WIDTH.saturating_sub(12))
                .iter()
                .enumerate()
            {
                if line_index == 0 {
                    lines.push(format!("   {} {wrapped}", muted("recent")));
                } else {
                    lines.push(format!("          {wrapped}"));
                }
            }
        }
        if index + 1 < game.competitors.len() {
            lines.push(String::new());
        }
    }

    lines
}

fn dashboard_competitor_lines(game: &Game) -> Vec<String> {
    let mut lines = competitor_lines(game);
    if game.competitors.len() > 3 {
        lines.truncate(6);
        lines.push(format!(
            "{} use 'rivals' for {} more",
            muted("More:"),
            game.competitors.len() - 3
        ));
    }
    lines
}

fn acquisition_status(game: &Game, competitor_index: usize) -> (String, &'static str) {
    if game.competitors.len() == 1 {
        ("blocked".to_string(), RED)
    } else if game.acquisition_cooldown > 0 {
        (format!("{}q wait", game.acquisition_cooldown), YELLOW)
    } else {
        (money(game.acquisition_price(competitor_index)), CYAN)
    }
}

fn recent_rival_event(game: &Game, competitor_name: &str) -> Option<String> {
    game.last_report.as_ref().and_then(|report| {
        report
            .events
            .iter()
            .rev()
            .find(|event| event.contains(competitor_name))
            .cloned()
    })
}

fn signed_count(value: f64) -> String {
    format!("{:+.0}", value)
}

fn rival_customer_delta_tone(value: f64) -> &'static str {
    if value > 20.0 {
        BOLD_RED
    } else if value > 0.0 {
        BOLD_YELLOW
    } else if value < 0.0 {
        BOLD_GREEN
    } else {
        DIM
    }
}

fn project_lines(game: &Game) -> Vec<String> {
    let mut lines = if game.pending_projects.is_empty() {
        vec![styled(DIM, "No active projects.")]
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
            styled(YELLOW, format!("{}q left", game.acquisition_cooldown))
        ));
    }

    lines
}

fn signal_lines(game: &Game) -> Vec<String> {
    let mut lines = Vec::new();

    if !game.active_shocks.is_empty() {
        let descriptors: Vec<String> = game
            .active_shocks
            .iter()
            .map(|shock| format!("{} {}q", shock_label(&shock.kind), shock.quarters_remaining))
            .collect();
        lines.push(signal_line("Shock", BOLD_RED, descriptors.join(" | ")));
    }

    if let Some(report) = &game.last_report {
        let net_customers = report.new_customers - report.lost_customers;
        lines.push(signal_line(
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
        ));
    } else {
        lines.push(signal_line("Trend", CYAN, "no prior quarter yet"));
    }

    lines.push(signal_line(
        "Macro",
        macro_credit_tone(game),
        format!(
            "credit {} | demand {}",
            styled(macro_credit_tone(game), game.macro_state.credit_label()),
            styled(
                demand_tone(game.macro_state.demand_index),
                game.macro_state.demand_label()
            )
        ),
    ));

    let average_rate = market_average_rate(game);
    let rate_gap = game.player.rate_cents - average_rate;
    if rate_gap >= 0.4 {
        lines.push(signal_line(
            "Rate",
            YELLOW,
            format!(
                "{} above market; churn pressure",
                styled(BOLD_YELLOW, format!("{:.1}c", rate_gap))
            ),
        ));
    } else if rate_gap <= -0.4 {
        lines.push(signal_line(
            "Rate",
            CYAN,
            format!(
                "{} below market; margin tightens",
                styled(BOLD_CYAN, format!("{:.1}c", rate_gap.abs()))
            ),
        ));
    } else {
        lines.push(signal_line("Rate", CYAN, "near market average"));
    }

    let capacity = game.player.customer_capacity(&game.market);
    let headroom = capacity - game.player.customers;
    let firm_generation_reserve = game.player.firm_generation_reserve_mwh(&game.market);
    if headroom < 60.0 {
        lines.push(signal_line("Capacity", RED, "tight; build capacity first"));
    } else if headroom < 140.0 {
        lines.push(signal_line("Capacity", YELLOW, "adequate, but narrowing"));
    } else {
        lines.push(signal_line("Capacity", GREEN, "growth headroom available"));
    }
    if firm_generation_reserve < 15.0 {
        lines.push(signal_line(
            "Generation",
            reserve_tone(firm_generation_reserve),
            "firm reserve thin",
        ));
    }

    let leverage = game.player.debt_to_assets();
    if leverage > 0.85 {
        lines.push(signal_line("Debt", RED, "high leverage; lender risk"));
    } else if leverage < 0.55 {
        lines.push(signal_line("Debt", GREEN, "borrowing capacity available"));
    } else {
        lines.push(signal_line("Debt", YELLOW, "usable, but no longer cheap"));
    }

    if game.player.reliability < 0.74 {
        lines.push(signal_line(
            "Reliability",
            RED,
            "below target; fund maintenance",
        ));
    } else if game.player.reliability > 0.88 {
        lines.push(signal_line(
            "Reliability",
            GREEN,
            "strong; expansion can lead",
        ));
    }

    lines.truncate(6);
    lines
}

fn shock_label(kind: &ShockKind) -> &'static str {
    match kind {
        ShockKind::RateFreeze => "rate freeze",
        ShockKind::DemandRecession => "demand recession",
        ShockKind::DemandBoom => "demand boom",
        ShockKind::InputCostShock => "input cost shock",
    }
}

fn action_effect_lines(game: &Game) -> Vec<String> {
    let mut lines = vec![
        action_line(
            "next / n",
            "finish the quarter; results print below this guide",
        ),
        action_line("build gen 400", project_quote_inline("gen", 400.0)),
        action_line("build lines 600", project_quote_inline("lines", 600.0)),
        action_line(
            "marketing 4000",
            "-$4.0k now; increases customer capture this quarter",
        ),
        action_line(
            "maint 6000",
            "reliability gain scales down as the asset base grows",
        ),
        action_line(
            "debt [amt]",
            format!(
                "{} available at {} floating",
                styled(
                    borrowing_room_tone(game.borrowing_room()),
                    money(game.borrowing_room())
                ),
                styled(
                    interest_rate_tone(game.player_annual_interest_rate()),
                    format!("{:.1}%", game.player_annual_interest_rate() * 100.0)
                )
            ),
        ),
        action_line(
            "stock / buyback",
            "equity raises cash with dilution; buybacks spend cash",
        ),
        action_line("rivals / board", "inspect competitors or milestone targets"),
    ];

    if let Some((index, price)) = cheapest_competitor(game) {
        if game.competitors.len() == 1 {
            lines.push(action_line(
                "buy",
                styled(RED, "final rival acquisition blocked by regulator"),
            ));
        } else if game.acquisition_cooldown > 0 {
            lines.push(action_line(
                format!("buy {}", index + 1),
                format!(
                    "unavailable for {} more quarter(s) during integration",
                    styled(YELLOW, game.acquisition_cooldown)
                ),
            ));
        } else {
            lines.push(action_line(
                format!("buy {}", index + 1),
                format!(
                    "{} cash plus assumed debt; immediate customers/capacity",
                    styled(BOLD_YELLOW, money(price))
                ),
            ));
        }
    }

    lines.push(action_line("help", "full command reference"));

    lines
}

fn project_quote_inline(kind: &str, size: f64) -> String {
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

fn cheapest_competitor(game: &Game) -> Option<(usize, f64)> {
    (0..game.competitors.len())
        .map(|index| (index, game.acquisition_price(index)))
        .min_by(|left, right| left.1.total_cmp(&right.1))
}

fn market_average_rate(game: &Game) -> f64 {
    let mut weighted = game.player.rate_cents * game.player.customers;
    let mut customers = game.player.customers;
    for competitor in &game.competitors {
        weighted += competitor.rate_cents * competitor.customers;
        customers += competitor.customers;
    }
    if customers <= 0.0 {
        game.market.standard_rate_cents
    } else {
        weighted / customers
    }
}

fn shorten_plain(value: &str, width: usize) -> String {
    if value.chars().count() <= width {
        return value.to_string();
    }

    let keep = width.saturating_sub(1);
    let mut shortened = value.chars().take(keep).collect::<String>();
    shortened.push('…');
    shortened
}

fn styled(style: &str, value: impl std::fmt::Display) -> String {
    format!("{style}{value}{RESET}")
}

fn muted(value: impl std::fmt::Display) -> String {
    styled(DIM, value)
}

fn action_line(command: impl std::fmt::Display, effect: impl std::fmt::Display) -> String {
    format!(
        "{}  {}",
        styled(BOLD_CYAN, format!("{command:<18}")),
        effect
    )
}

fn signal_line(label: &str, tone: &str, body: impl std::fmt::Display) -> String {
    format!("{} {}", styled(tone, format!("{label}:")), body)
}

fn macro_credit_tone(game: &Game) -> &'static str {
    let rate = game.macro_state.benchmark_credit_rate();
    if rate >= 0.105 {
        BOLD_RED
    } else if rate >= 0.080 {
        BOLD_YELLOW
    } else {
        BOLD_GREEN
    }
}

fn interest_rate_tone(rate: f64) -> &'static str {
    if rate >= 0.14 {
        BOLD_RED
    } else if rate >= 0.09 {
        BOLD_YELLOW
    } else {
        BOLD_GREEN
    }
}

fn debt_cap_tone(limit: f64) -> &'static str {
    if limit <= 0.75 {
        BOLD_RED
    } else if limit <= 0.88 {
        BOLD_YELLOW
    } else {
        BOLD_GREEN
    }
}

fn demand_tone(index: f64) -> &'static str {
    if index <= -0.25 {
        BOLD_RED
    } else if index < 0.20 {
        BOLD_YELLOW
    } else {
        BOLD_GREEN
    }
}

fn cost_pressure_tone(pressure: f64) -> &'static str {
    if pressure >= 0.030 {
        BOLD_RED
    } else if pressure > 0.004 {
        BOLD_YELLOW
    } else {
        BOLD_GREEN
    }
}

fn cash_tone(cash: f64) -> &'static str {
    if cash < 5_000.0 {
        BOLD_RED
    } else if cash < 20_000.0 {
        BOLD_YELLOW
    } else {
        BOLD_GREEN
    }
}

fn borrowing_room_tone(room: f64) -> &'static str {
    if room < 10_000.0 {
        BOLD_RED
    } else if room < 35_000.0 {
        BOLD_YELLOW
    } else {
        BOLD_GREEN
    }
}

fn profit_tone(profit: f64) -> &'static str {
    if profit < 0.0 {
        BOLD_RED
    } else if profit.abs() < 1_000.0 {
        BOLD_YELLOW
    } else {
        BOLD_GREEN
    }
}

fn customer_tone(customers: f64) -> &'static str {
    if customers < 0.0 {
        BOLD_RED
    } else if customers < 15.0 {
        BOLD_YELLOW
    } else {
        BOLD_GREEN
    }
}

fn format_churn_rate(churn_rate: f64) -> String {
    format!("{:.2}%", churn_rate * 100.0)
}

fn churn_tone(churn_rate: f64) -> &'static str {
    if churn_rate >= 0.025 {
        BOLD_RED
    } else if churn_rate >= 0.012 {
        BOLD_YELLOW
    } else {
        BOLD_GREEN
    }
}

fn share_tone(share: f64) -> &'static str {
    if share >= 0.45 {
        BOLD_GREEN
    } else if share >= 0.35 {
        BOLD_YELLOW
    } else {
        BOLD_RED
    }
}

fn reliability_tone(reliability: f64) -> &'static str {
    if reliability >= 0.80 {
        BOLD_GREEN
    } else if reliability >= 0.72 {
        BOLD_YELLOW
    } else {
        BOLD_RED
    }
}

fn leverage_tone(leverage: f64) -> &'static str {
    if leverage > 0.85 {
        BOLD_RED
    } else if leverage > 0.70 {
        BOLD_YELLOW
    } else {
        BOLD_GREEN
    }
}

fn headroom_tone(headroom: f64) -> &'static str {
    if headroom < 60.0 {
        BOLD_RED
    } else if headroom < 140.0 {
        BOLD_YELLOW
    } else {
        BOLD_GREEN
    }
}

fn reserve_tone(reserve: f64) -> &'static str {
    if reserve < 0.0 {
        BOLD_RED
    } else if reserve < 15.0 {
        BOLD_YELLOW
    } else {
        BOLD_GREEN
    }
}

fn utilization_tone(utilization: f64) -> &'static str {
    if utilization > 0.92 {
        BOLD_RED
    } else if utilization > 0.80 {
        BOLD_YELLOW
    } else {
        BOLD_GREEN
    }
}

fn reputation_tone(reputation: f64) -> &'static str {
    if reputation < 48.0 {
        BOLD_RED
    } else if reputation < 60.0 {
        BOLD_YELLOW
    } else {
        BOLD_GREEN
    }
}

fn print_box(title: &str, lines: &[String]) {
    for line in render_box(title, lines, SCREEN_WIDTH) {
        println!("{line}");
    }
}

fn print_box_pair(
    left_title: &str,
    left_lines: &[String],
    right_title: &str,
    right_lines: &[String],
) {
    let content_height = left_lines.len().max(1).max(right_lines.len().max(1));
    let left = render_box(
        left_title,
        &padded_box_lines(left_lines, content_height),
        COLUMN_WIDTH,
    );
    let right = render_box(
        right_title,
        &padded_box_lines(right_lines, content_height),
        COLUMN_WIDTH,
    );

    for index in 0..left.len() {
        let left_line = left
            .get(index)
            .cloned()
            .unwrap_or_else(|| " ".repeat(COLUMN_WIDTH));
        let right_line = right
            .get(index)
            .cloned()
            .unwrap_or_else(|| " ".repeat(COLUMN_WIDTH));
        println!("{left_line}{}{right_line}", " ".repeat(COLUMN_GAP));
    }
}

fn padded_box_lines(lines: &[String], height: usize) -> Vec<String> {
    let mut padded = if lines.is_empty() {
        vec![String::new()]
    } else {
        lines.to_vec()
    };
    while padded.len() < height {
        padded.push(String::new());
    }
    padded
}

fn clear_screen() {
    print!("\x1B[2J\x1B[H");
}

fn render_box(title: &str, lines: &[String], width: usize) -> Vec<String> {
    let mut rendered = Vec::new();
    rendered.push(render_top_border(title, width));
    if lines.is_empty() {
        rendered.push(render_box_line("", width));
    } else {
        for line in lines {
            rendered.push(render_box_line(line, width));
        }
    }
    rendered.push(render_bottom_border(width));
    rendered
}

fn render_top_border(title: &str, width: usize) -> String {
    if title.is_empty() {
        format!("{DIM}┌{}┐{RESET}", "─".repeat(width - 2))
    } else {
        let title = shorten_plain(title, width.saturating_sub(6));
        let fill = (width - 2).saturating_sub(visible_width(&title) + 3);
        format!(
            "{DIM}┌─ {BOLD_CYAN}{title}{RESET}{DIM} {}┐{RESET}",
            "─".repeat(fill)
        )
    }
}

fn render_bottom_border(width: usize) -> String {
    format!("{DIM}└{}┘{RESET}", "─".repeat(width - 2))
}

fn render_box_line(line: &str, width: usize) -> String {
    let content_width = width.saturating_sub(4);
    let fitted = fit_line(line, content_width);
    let padding = content_width.saturating_sub(visible_width(&fitted));
    format!(
        "{DIM}│{RESET} {}{} {DIM}│{RESET}",
        fitted,
        " ".repeat(padding)
    )
}

fn fit_line(line: &str, width: usize) -> String {
    if visible_width(line) <= width {
        return line.to_string();
    }

    let keep = width.saturating_sub(3);
    let mut truncated = String::new();
    let mut visible = 0;
    let mut chars = line.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\x1B' {
            truncated.push(ch);
            if matches!(chars.peek(), Some(&'[')) {
                truncated.push(chars.next().expect("peeked CSI introducer"));
                while let Some(sequence_char) = chars.next() {
                    truncated.push(sequence_char);
                    if ('@'..='~').contains(&sequence_char) {
                        break;
                    }
                }
            }
            continue;
        }

        if visible >= keep {
            break;
        }
        truncated.push(ch);
        visible += 1;
    }

    truncated.push_str(RESET);
    truncated.push_str("...");
    truncated
}

fn wrap_plain_text(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();

    for word in text.split_whitespace() {
        if current.is_empty() {
            current.push_str(word);
        } else if current.len() + 1 + word.len() <= width {
            current.push(' ');
            current.push_str(word);
        } else {
            lines.push(current);
            current = word.to_string();
        }
    }

    if !current.is_empty() {
        lines.push(current);
    }

    if lines.is_empty() {
        lines.push(String::new());
    }

    lines
}

fn visible_width(line: &str) -> usize {
    let mut width = 0;
    let mut chars = line.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\x1B' {
            if matches!(chars.peek(), Some(&'[')) {
                chars.next();
                while let Some(sequence_char) = chars.next() {
                    if ('@'..='~').contains(&sequence_char) {
                        break;
                    }
                }
            }
        } else {
            width += 1;
        }
    }

    width
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rate_up_with_amount_uses_requested_delta() {
        let parts = vec!["rate", "up", "2"];
        assert_eq!(parse_rate_delta(&parts, 10.5).unwrap(), Some(2.0));
    }

    #[test]
    fn rate_down_with_amount_uses_requested_delta() {
        let parts = vec!["rate", "down", "1.5"];
        assert_eq!(parse_rate_delta(&parts, 10.5).unwrap(), Some(-1.5));
    }

    #[test]
    fn bare_rate_number_sets_target_rate() {
        let parts = vec!["rate", "9.5"];
        assert_eq!(parse_rate_delta(&parts, 10.5).unwrap(), Some(-1.0));
    }

    #[test]
    fn next_advances_the_quarter() {
        let mut game = Game::with_seed(100);

        match handle_command(&mut game, "next") {
            CommandResult::Advanced(report) => assert_eq!(report.label, "Year 1 Q1"),
            _ => panic!("next should advance the quarter"),
        }
        assert_eq!(game.quarter, 1);
    }

    #[test]
    fn n_alias_advances_the_quarter() {
        let mut game = Game::with_seed(101);

        match handle_command(&mut game, "n") {
            CommandResult::Advanced(report) => assert_eq!(report.label, "Year 1 Q1"),
            _ => panic!("n should advance the quarter"),
        }
        assert_eq!(game.quarter, 1);
    }

    #[test]
    fn old_advance_command_is_not_the_turn_command() {
        let mut game = Game::with_seed(102);

        match handle_command(&mut game, "advance") {
            CommandResult::Continue(message) => assert!(message.contains("Unknown command")),
            _ => panic!("advance should no longer finish the quarter"),
        }
        assert_eq!(game.quarter, 0);
    }

    #[test]
    fn board_command_shows_objectives() {
        let mut game = Game::with_seed(104);

        match handle_command(&mut game, "board") {
            CommandResult::ShowBoard => {}
            _ => panic!("board should show objectives"),
        }
    }

    #[test]
    fn maint_alias_runs_maintenance() {
        let mut game = Game::with_seed(103);
        let starting_cash = game.player.cash;
        let starting_reliability = game.player.reliability;

        match handle_command(&mut game, "maint 4500") {
            CommandResult::Continue(message) => assert!(message.contains("reliability")),
            _ => panic!("maint should apply maintenance"),
        }
        assert!(game.player.cash < starting_cash);
        assert!(game.player.reliability > starting_reliability);
    }

    #[test]
    fn visible_width_ignores_ansi_escape_sequences() {
        let line = format!("{}Cash{} {}", DIM, RESET, styled(BOLD_GREEN, "$34.0k"));
        assert_eq!(visible_width(&line), "Cash $34.0k".len());
    }

    #[test]
    fn fit_line_preserves_ansi_sequences_without_counting_them() {
        let line = format!(
            "{} {}",
            styled(BOLD_CYAN, "stock issue [amt]"),
            "no fixed cap"
        );
        let fitted = fit_line(&line, 24);
        assert_eq!(visible_width(&fitted), 24);
        assert!(fitted.contains(BOLD_CYAN));
    }

    #[test]
    fn render_box_preserves_requested_visible_width() {
        let lines = vec![format!(
            "{} {}",
            styled(BOLD_GREEN, "Cash"),
            styled(BOLD_CYAN, "$34.0k")
        )];

        for line in render_box("Capital", &lines, COLUMN_WIDTH) {
            assert_eq!(visible_width(&line), COLUMN_WIDTH);
        }
    }

    #[test]
    fn outcome_details_wrap_within_box_width() {
        let details = "Repeated outages pushed regulators and lenders to move the company into managed restructuring.";

        let lines = wrap_plain_text(details, 42);

        assert!(lines.len() > 1);
        assert!(lines.iter().all(|line| visible_width(line) <= 42));
    }
}
