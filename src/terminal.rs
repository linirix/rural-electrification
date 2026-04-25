use std::io::{self, Write};

use crate::sim::{
    DISTRIBUTION_PROJECT_CAPACITY, Decision, GENERATION_PROJECT_CAPACITY_MWH, Game, Outcome,
    OutcomeKind, QuarterReport, ShockKind, distribution_project_cost,
    distribution_project_duration, generation_project_cost, generation_project_duration, money,
};

const SCREEN_WIDTH: usize = 88;
const CONTENT_WIDTH: usize = SCREEN_WIDTH - 4;
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
    Quit,
}

fn handle_command(game: &mut Game, command: &str) -> CommandResult {
    let parts = command.split_whitespace().collect::<Vec<_>>();
    let Some(first) = parts.first().copied() else {
        return CommandResult::Continue(String::new());
    };

    match first {
        "help" | "?" => CommandResult::ShowHelp,
        "status" | "s" => CommandResult::ShowStatus,
        "advance" | "end" | "next" => CommandResult::Advanced(game.advance_quarter()),
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
        "maintenance" | "maintain" | "reliability" => {
            let spend = parts
                .get(1)
                .and_then(|value| parse_money(value))
                .unwrap_or(4_500.0);
            apply(game, Decision::Maintenance { spend })
        }
        "competitors" | "rivals" => CommandResult::ShowRivals,
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
    print_box(
        "Commands",
        &[
            "status                 show the dashboard".to_string(),
            "build gen [MWh]        add generation capacity; larger projects cost less per MWh"
                .to_string(),
            "build lines [cust]     add customer capacity; larger projects cost less per customer"
                .to_string(),
            "marketing [amount]     invest in customer acquisition".to_string(),
            "stock issue [amount]   raise equity at an issue discount".to_string(),
            "buyback [amount]       repurchase shares with cash".to_string(),
            "debt [amount]          borrow against the asset base".to_string(),
            "repay [amount]         pay down debt with cash".to_string(),
            "buy <number>           acquire a listed competitor".to_string(),
            "rate up|down [cents]   change rate by cents, or 'rate 10.0' to set target".to_string(),
            "maintenance [amount]   improve reliability and reputation".to_string(),
            "advance                finish the quarter".to_string(),
            "quit                   leave the game".to_string(),
        ],
    );
}

fn print_status(game: &Game) {
    let share = game.market_share();
    let reliability = game.player.reliability;
    let leverage = game.player.debt_to_assets();
    print_box(
        &format!("{} | {}", game.date_label(), game.player.name),
        &[
            format!(
                "{} {}",
                muted("Market review:"),
                styled(BOLD, &game.market.territory)
            ),
            "Targets by end of Year 5: 45%+ share | 72%+ reliability | debt/assets <= 95%"
                .to_string(),
            format!(
                "{} {} share | {} reliability | {} debt/assets",
                muted("Current position:"),
                styled(share_tone(share), format!("{:.0}%", share * 100.0)),
                styled(
                    reliability_tone(reliability),
                    format!("{:.0}%", reliability * 100.0)
                ),
                styled(leverage_tone(leverage), format!("{:.0}%", leverage * 100.0))
            ),
        ],
    );
    print_box("Financials", &financial_lines(game));
    print_box("Operations", &operation_lines(game));
    print_box("Market And Rivals", &market_lines(game));
    print_box("Projects", &project_lines(game));
    print_box("Signals", &signal_lines(game));
    print_box("Action Effects", &action_effect_lines(game));
}

fn print_competitors(game: &Game) {
    print_box("Rivals", &competitor_lines(game));
}

fn print_notice(message: &str) {
    println!();
    print_box("Notice", &[message.to_string()]);
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
                format!("{:.0}%", report.market_share * 100.0)
            )
        ),
    ];
    if !report.events.is_empty() {
        for event in &report.events {
            lines.push(format!("{} {event}", styled(BOLD_YELLOW, "Event:")));
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
    println!();
    print_box(
        "Outcome",
        &[
            styled(outcome_style, &outcome.headline),
            outcome.details.clone(),
            format!("{} {}", muted("Result:"), styled(outcome_style, result)),
        ],
    );
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

fn financial_lines(game: &Game) -> Vec<String> {
    let borrowing_room = game.borrowing_room();
    let leverage = game.player.debt_to_assets();
    let debt_rate = game.player_annual_interest_rate();
    let mut lines = vec![
        format!(
            "{} {} | {} {} | {} {}",
            muted("Cash"),
            styled(
                cash_tone(game.player.cash),
                format!("{:>8}", money(game.player.cash))
            ),
            muted("Debt"),
            styled(
                leverage_tone(leverage),
                format!("{:>8}", money(game.player.debt))
            ),
            muted("Borrowing room"),
            styled(
                borrowing_room_tone(borrowing_room),
                format!("{:>8}", money(borrowing_room))
            )
        ),
        format!(
            "{} {} | {} {} | {} {}",
            muted("Stock"),
            styled(BOLD_CYAN, format!("${:>5.2}", game.player.stock_price)),
            muted("Shares"),
            styled(BOLD, format!("{:>6.0}", game.player.shares)),
            muted("Market cap"),
            styled(BOLD, format!("{:>8}", money(game.player.market_cap())))
        ),
        format!(
            "{} {} | {} {} | {} {} | {} {} | {} {}",
            muted("Macro"),
            styled(macro_credit_tone(game), game.macro_state.credit_label()),
            muted("Base"),
            styled(
                interest_rate_tone(game.macro_state.annual_base_rate),
                format!("{:.1}%", game.macro_state.annual_base_rate * 100.0)
            ),
            muted("Spread"),
            styled(
                spread_tone(game.macro_state.credit_spread),
                format!("{:.1}%", game.macro_state.credit_spread * 100.0)
            ),
            muted("Debt rate"),
            styled(
                interest_rate_tone(debt_rate),
                format!("{:.1}%", debt_rate * 100.0)
            ),
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
            "{} {} | {} {} | {} {}",
            muted("Last quarter: profit"),
            styled(
                profit_tone(report.profit),
                format!("{:>8}", money(report.profit))
            ),
            muted("margin"),
            styled(profit_tone(margin), format!("{:>5.1}%", margin)),
            muted("interest"),
            styled(YELLOW, format!("{:>8}", money(report.interest)))
        ));
    } else {
        lines.push(format!(
            "{} no operating report yet",
            muted("Last quarter:")
        ));
    }

    lines
}

fn operation_lines(game: &Game) -> Vec<String> {
    let capacity = game.player.customer_capacity(&game.market);
    let headroom = (capacity - game.player.customers).max(0.0);
    let generation_customer_capacity =
        game.player.generation_capacity_mwh / game.market.avg_mwh_per_customer.max(0.05);
    let demanded_mwh = game.player.demanded_mwh(&game.market);
    let generation_reserve = game.player.generation_reserve_mwh(&game.market);
    let firm_generation_reserve = game.player.firm_generation_reserve_mwh(&game.market);
    let limiting = if game.player.distribution_capacity <= generation_customer_capacity {
        "distribution"
    } else {
        "generation"
    };

    vec![
        format!(
            "{} {} | {} {} | {} {} | {} {}",
            muted("Customers"),
            styled(BOLD, format!("{:>6.0}", game.player.customers)),
            muted("Capacity"),
            styled(BOLD, format!("{:>6.0}", capacity)),
            muted("Headroom"),
            styled(headroom_tone(headroom), format!("{:>6.0}", headroom)),
            muted("Limit:"),
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
            "{} {} MWh/q | {} {} | {} {} | {} {}",
            muted("Generation"),
            styled(
                BOLD,
                format!("{:>6.0}", game.player.generation_capacity_mwh)
            ),
            muted("Demand"),
            styled(BOLD, format!("{:>5.0}", demanded_mwh)),
            muted("Reserve"),
            styled(
                reserve_tone(generation_reserve),
                format!("{:>+6.0}", generation_reserve)
            ),
            muted("Firm"),
            styled(
                reserve_tone(firm_generation_reserve),
                format!("{:>+6.0}", firm_generation_reserve)
            )
        ),
        format!(
            "{} {} | {} {}",
            muted("Utilization"),
            styled(
                utilization_tone(game.player.utilization(&game.market)),
                format!("{:>5.0}%", game.player.utilization(&game.market) * 100.0)
            ),
            muted("Avg use"),
            styled(
                BOLD,
                format!("{:.2} MWh/customer/q", game.market.avg_mwh_per_customer)
            ),
        ),
        format!(
            "{} {} | {} {} | {} {}",
            muted("Reliability"),
            styled(
                reliability_tone(game.player.reliability),
                format!("{:>5.0}%", game.player.reliability * 100.0)
            ),
            muted("Reputation"),
            styled(
                reputation_tone(game.player.reputation),
                format!("{:>5.0}", game.player.reputation)
            ),
            muted("Rate"),
            styled(BOLD_CYAN, format!("{:.1}c/kWh", game.player.rate_cents))
        ),
    ]
}

fn market_lines(game: &Game) -> Vec<String> {
    let mut lines = vec![
        format!(
            "{} {} / {} | {} {}",
            muted("Serviceable customers"),
            styled(BOLD, format!("{:>6.0}", game.serviceable_customers())),
            styled(
                DIM,
                format!("{:>6.0} addressable", game.market.addressable_customers)
            ),
            muted("Electrification"),
            styled(
                BOLD_CYAN,
                format!("{:.1}%", game.market.electrification * 100.0)
            )
        ),
        format!(
            "{} {} | {} {} | {} {}",
            muted("Connected accounts"),
            styled(BOLD, format!("{:>6.0}", game.total_connected_customers())),
            muted("Your share"),
            styled(
                share_tone(game.market_share()),
                format!("{:.0}%", game.market_share() * 100.0)
            ),
            muted("Addressable share"),
            styled(BOLD, format!("{:.1}%", game.addressable_share() * 100.0))
        ),
    ];
    lines.extend(competitor_lines(game));
    lines
}

fn competitor_lines(game: &Game) -> Vec<String> {
    if game.competitors.is_empty() {
        return vec!["No active rivals.".to_string()];
    }

    game.competitors
        .iter()
        .enumerate()
        .map(|(index, competitor)| {
            let (acquisition_note, acquisition_tone) = if game.competitors.len() == 1 {
                ("blocked".to_string(), RED)
            } else if game.acquisition_cooldown > 0 {
                (format!("{}q wait", game.acquisition_cooldown), YELLOW)
            } else {
                (money(game.acquisition_price(index)), CYAN)
            };
            format!(
                "{}. {} | {} {} | {} {} | {} {} | {} {}",
                styled(DIM, index + 1),
                styled(BOLD, format!("{:<20}", competitor.name)),
                muted("customers"),
                styled(BOLD, format!("{:>5.0}", competitor.customers)),
                muted("rel"),
                styled(
                    reliability_tone(competitor.reliability),
                    format!("{:>3.0}%", competitor.reliability * 100.0)
                ),
                muted("rate"),
                styled(CYAN, format!("{:>4.1}c", competitor.rate_cents)),
                muted("buy"),
                styled(acquisition_tone, acquisition_note)
            )
        })
        .collect()
}

fn project_lines(game: &Game) -> Vec<String> {
    let mut lines = if game.pending_projects.is_empty() {
        vec![styled(DIM, "No active projects.")]
    } else {
        game.pending_projects
            .iter()
            .map(|project| {
                format!(
                    "{} | {}",
                    styled(BOLD, &project.name),
                    styled(
                        YELLOW,
                        format!("{} quarter(s) remaining", project.quarters_remaining)
                    )
                )
            })
            .collect()
    };

    if game.acquisition_cooldown > 0 {
        lines.push(format!(
            "{} {}",
            muted("Acquisition integration:"),
            styled(
                YELLOW,
                format!("{} quarter(s) remaining", game.acquisition_cooldown)
            )
        ));
    }

    lines
}

fn signal_lines(game: &Game) -> Vec<String> {
    let mut lines = Vec::new();

    if let Some(report) = &game.last_report {
        let net_customers = report.new_customers - report.lost_customers;
        lines.push(signal_line(
            "Trend",
            CYAN,
            format!(
                "last quarter net customers {}, churn {}, profit {}, share {}",
                styled(
                    customer_tone(net_customers),
                    format!("{:+.0}", net_customers)
                ),
                styled(
                    churn_tone(report.lost_customer_rate),
                    format_churn_rate(report.lost_customer_rate)
                ),
                styled(profit_tone(report.profit), money(report.profit)),
                styled(
                    share_tone(report.market_share),
                    format!("{:.0}%", report.market_share * 100.0)
                )
            ),
        ));
    } else {
        lines.push(signal_line(
            "Trend",
            CYAN,
            "no prior quarter yet; first decision sets growth posture.",
        ));
    }

    lines.push(signal_line(
        "Macro",
        macro_credit_tone(game),
        format!(
            "credit {} | demand {} | costs {} | your debt rate {}",
            styled(macro_credit_tone(game), game.macro_state.credit_label()),
            styled(
                demand_tone(game.macro_state.demand_index),
                game.macro_state.demand_label()
            ),
            styled(
                cost_pressure_tone(game.macro_state.cost_pressure),
                game.macro_state.cost_label()
            ),
            styled(
                interest_rate_tone(game.player_annual_interest_rate()),
                format!("{:.1}%", game.player_annual_interest_rate() * 100.0)
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
                "{} above market average; margin improves, churn pressure rises.",
                styled(BOLD_YELLOW, format!("{:.1}c", rate_gap))
            ),
        ));
    } else if rate_gap <= -0.4 {
        lines.push(signal_line(
            "Rate",
            CYAN,
            format!(
                "{} below market average; demand improves, margin tightens.",
                styled(BOLD_CYAN, format!("{:.1}c", rate_gap.abs()))
            ),
        ));
    } else {
        lines.push(signal_line(
            "Rate",
            CYAN,
            "close to market average; pricing is not the main swing factor.",
        ));
    }

    let capacity = game.player.customer_capacity(&game.market);
    let headroom = capacity - game.player.customers;
    let firm_generation_reserve = game.player.firm_generation_reserve_mwh(&game.market);
    if headroom < 60.0 {
        lines.push(signal_line(
            "Capacity",
            RED,
            "tight; marketing may create demand you cannot serve.",
        ));
    } else if headroom < 140.0 {
        lines.push(signal_line(
            "Capacity",
            YELLOW,
            "adequate but narrowing; another growth push needs expansion.",
        ));
    } else {
        lines.push(signal_line(
            "Capacity",
            GREEN,
            "enough headroom to absorb near-term customer growth.",
        ));
    }
    if firm_generation_reserve < 15.0 {
        lines.push(signal_line(
            "Generation",
            reserve_tone(firm_generation_reserve),
            "firm reserve is thin; outages or growth could expose a shortfall.",
        ));
    }

    let leverage = game.player.debt_to_assets();
    if leverage > 0.85 {
        lines.push(signal_line(
            "Debt",
            RED,
            "high leverage; more borrowing risks lender pressure.",
        ));
    } else if leverage < 0.55 {
        lines.push(signal_line(
            "Debt",
            GREEN,
            "borrowing capacity remains available for projects or acquisitions.",
        ));
    } else {
        lines.push(signal_line(
            "Debt",
            YELLOW,
            "usable but no longer cheap; balance growth with solvency.",
        ));
    }

    if game.player.reliability < 0.74 {
        lines.push(signal_line(
            "Reliability",
            RED,
            "below target; maintenance has strategic value now.",
        ));
    } else if game.player.reliability > 0.88 {
        lines.push(signal_line(
            "Reliability",
            GREEN,
            "strong; expansion can be prioritized if capacity is available.",
        ));
    }

    if !game.active_shocks.is_empty() {
        let descriptors: Vec<String> = game
            .active_shocks
            .iter()
            .map(|shock| {
                format!(
                    "{} ({}q)",
                    shock_label(&shock.kind),
                    shock.quarters_remaining
                )
            })
            .collect();
        lines.push(signal_line("Shocks", BOLD_RED, descriptors.join(" | ")));
    }

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
            "build gen [MWh]",
            "larger projects cost more, but less per MWh",
        ),
        project_quote("gen", GENERATION_PROJECT_CAPACITY_MWH),
        project_quote("gen", 400.0),
        action_line(
            "build lines [cust]",
            "larger projects cost more, but less per customer",
        ),
        project_quote("lines", DISTRIBUTION_PROJECT_CAPACITY),
        project_quote("lines", 600.0),
        action_line(
            "marketing 4000",
            "-$4.0k cash now | more customer capture this quarter; decays fast",
        ),
        action_line(
            "rate up 2",
            "+2.0c/kWh | rate down 1 -1.0c | rate 10.0 sets target",
        ),
        action_line(
            "stock issue [amt]",
            "no fixed cap; larger sales face discounts and fees",
        ),
        action_line(
            "buyback [amt]",
            "limited by cash and public float; pays premium",
        ),
        action_line(
            "debt [amt]",
            "floating-rate debt; room depends on credit and assets",
        ),
        action_line("repay [amt]", "limited by cash and outstanding debt"),
        action_line(
            "maintenance",
            "immediate reliability/reputation gain; best before outages compound",
        ),
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
                    "-{} cash plus assumed debt | immediate customers and capacity",
                    styled(BOLD_YELLOW, money(price))
                ),
            ));
        }
    }

    lines
}

fn project_quote(kind: &str, size: f64) -> String {
    match kind {
        "gen" => {
            let cost = generation_project_cost(size);
            format!(
                "  {} {} | {} | {}",
                styled(CYAN, format!("build gen {:>4.0}", size)),
                styled(BOLD_YELLOW, format!("{:>7}", money(cost))),
                styled(YELLOW, format!("{}q", generation_project_duration(size))),
                styled(DIM, format!("{:>5}/MWh/q", money(cost / size)))
            )
        }
        "lines" => {
            let cost = distribution_project_cost(size);
            format!(
                "  {} {} | {} | {}",
                styled(CYAN, format!("build lines {:>4.0}", size)),
                styled(BOLD_YELLOW, format!("{:>7}", money(cost))),
                styled(YELLOW, format!("{}q", distribution_project_duration(size))),
                styled(DIM, format!("{:>5}/customer", money(cost / size)))
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

fn spread_tone(spread: f64) -> &'static str {
    if spread >= 0.045 {
        BOLD_RED
    } else if spread >= 0.028 {
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
    print_top_border(title);
    if lines.is_empty() {
        print_box_line("");
    } else {
        for line in lines {
            print_box_line(line);
        }
    }
    print_bottom_border();
}

fn clear_screen() {
    print!("\x1B[2J\x1B[H");
}

fn print_top_border(title: &str) {
    if title.is_empty() {
        println!("{DIM}┌{}┐{RESET}", "─".repeat(SCREEN_WIDTH - 2));
    } else {
        let fill = (SCREEN_WIDTH - 2).saturating_sub(title.chars().count() + 3);
        println!(
            "{DIM}┌─ {BOLD_CYAN}{title}{RESET}{DIM} {}┐{RESET}",
            "─".repeat(fill)
        );
    }
}

fn print_bottom_border() {
    println!("{DIM}└{}┘{RESET}", "─".repeat(SCREEN_WIDTH - 2));
}

fn print_box_line(line: &str) {
    let fitted = fit_line(line, CONTENT_WIDTH);
    let padding = CONTENT_WIDTH.saturating_sub(visible_width(&fitted));
    println!(
        "{DIM}│{RESET} {}{} {DIM}│{RESET}",
        fitted,
        " ".repeat(padding)
    );
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
}
