use std::io::{self, Write};

use crate::sim::{
    DISTRIBUTION_PROJECT_CAPACITY, Decision, GENERATION_PROJECT_CAPACITY_MWH, Game, OutcomeKind,
    QuarterReport, distribution_project_cost, distribution_project_duration,
    generation_project_cost, generation_project_duration, money,
};

const SCREEN_WIDTH: usize = 88;
const CONTENT_WIDTH: usize = SCREEN_WIDTH - 4;

pub fn run() -> io::Result<()> {
    let mut game = Game::new();

    clear_screen();
    println!("Electrification");
    println!(
        "You are general manager of {} in {}.",
        game.player.name, game.market.territory
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
                if !message.is_empty() {
                    print_box("Notice", &[message]);
                }
                print_status(&game);
            }
            CommandResult::Advanced(report) => {
                clear_screen();
                print_report(&report);
                if let Some(outcome) = &game.outcome {
                    println!("\n{}", outcome.headline);
                    println!("{}", outcome.details);
                    match outcome.kind {
                        OutcomeKind::Victory => println!("Result: victory"),
                        OutcomeKind::Defeat => println!("Result: defeat"),
                    }
                    break;
                } else {
                    print_status(&game);
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
    print_box(
        &format!("{} | {}", game.date_label(), game.player.name),
        &[
            format!("Market review: {}", game.market.territory),
            "Targets by end of Year 5: 45%+ share | 72%+ reliability | debt/assets <= 95%"
                .to_string(),
            format!(
                "Current position: {:.0}% share | {:.0}% reliability | {:.0}% debt/assets",
                game.market_share() * 100.0,
                game.player.reliability * 100.0,
                game.player.debt_to_assets() * 100.0
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

fn print_report(report: &QuarterReport) {
    let mut lines = vec![
        format!(
            "Revenue {} | Costs {} | Interest {} | Profit {}",
            money(report.revenue),
            money(report.operating_cost),
            money(report.interest),
            money(report.profit)
        ),
        format!(
            "Customers: +{:.0} gross, -{:.0} churn | Market share {:.0}%",
            report.new_customers,
            report.lost_customers,
            report.market_share * 100.0
        ),
    ];
    if !report.events.is_empty() {
        for event in &report.events {
            lines.push(format!("Event: {event}"));
        }
    }
    println!();
    print_box(&format!("{} Results", report.label), &lines);
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
    let borrowing_room = (game.player.asset_base * 0.95 - game.player.debt).max(0.0);
    let mut lines = vec![
        format!(
            "Cash {:>8} | Debt {:>8} | Borrowing room {:>8}",
            money(game.player.cash),
            money(game.player.debt),
            money(borrowing_room)
        ),
        format!(
            "Stock ${:>5.2} | Shares {:>6.0} | Market cap {:>8}",
            game.player.stock_price,
            game.player.shares,
            money(game.player.market_cap())
        ),
    ];

    if let Some(report) = &game.last_report {
        let margin = if report.revenue > 0.0 {
            report.profit / report.revenue * 100.0
        } else {
            0.0
        };
        lines.push(format!(
            "Last quarter: profit {:>8} | margin {:>5.1}% | interest {:>8}",
            money(report.profit),
            margin,
            money(report.interest)
        ));
    } else {
        lines.push("Last quarter: no operating report yet".to_string());
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
            "Customers {:>6.0} | Capacity {:>6.0} | Headroom {:>6.0} | Limit: {}",
            game.player.customers, capacity, headroom, limiting
        ),
        format!(
            "Generation {:>6.0} MWh/q | Demand {:>5.0} | Reserve {:>+6.0} | Firm {:>+6.0}",
            game.player.generation_capacity_mwh,
            demanded_mwh,
            generation_reserve,
            firm_generation_reserve
        ),
        format!(
            "Utilization {:>5.0}% | Avg use {:.2} MWh/customer/q",
            game.player.utilization(&game.market) * 100.0,
            game.market.avg_mwh_per_customer,
        ),
        format!(
            "Reliability {:>5.0}% | Reputation {:>5.0} | Rate {:.1}c/kWh",
            game.player.reliability * 100.0,
            game.player.reputation,
            game.player.rate_cents
        ),
    ]
}

fn market_lines(game: &Game) -> Vec<String> {
    let mut lines = vec![
        format!(
            "Serviceable customers {:>6.0} / addressable {:>6.0} | Electrification {:.1}%",
            game.serviceable_customers(),
            game.market.addressable_customers,
            game.market.electrification * 100.0
        ),
        format!(
            "Connected accounts {:>6.0} | Your share {:.0}% | Addressable share {:.1}%",
            game.total_connected_customers(),
            game.market_share() * 100.0,
            game.addressable_share() * 100.0
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
            let acquisition_note = if game.competitors.len() == 1 {
                "blocked".to_string()
            } else if game.acquisition_cooldown > 0 {
                format!("{}q wait", game.acquisition_cooldown)
            } else {
                money(game.acquisition_price(index))
            };
            format!(
                "{}. {:<20} | customers {:>5.0} | rel {:>3.0}% | rate {:>4.1}c | buy {}",
                index + 1,
                competitor.name,
                competitor.customers,
                competitor.reliability * 100.0,
                competitor.rate_cents,
                acquisition_note
            )
        })
        .collect()
}

fn project_lines(game: &Game) -> Vec<String> {
    let mut lines = if game.pending_projects.is_empty() {
        vec!["No active projects.".to_string()]
    } else {
        game.pending_projects
            .iter()
            .map(|project| {
                format!(
                    "{} | {} quarter(s) remaining",
                    project.name, project.quarters_remaining
                )
            })
            .collect()
    };

    if game.acquisition_cooldown > 0 {
        lines.push(format!(
            "Acquisition integration: {} quarter(s) remaining",
            game.acquisition_cooldown
        ));
    }

    lines
}

fn signal_lines(game: &Game) -> Vec<String> {
    let mut lines = Vec::new();

    if let Some(report) = &game.last_report {
        let net_customers = report.new_customers - report.lost_customers;
        lines.push(format!(
            "Trend: last quarter net customers {:+.0}, profit {}, share {:.0}%",
            net_customers,
            money(report.profit),
            report.market_share * 100.0
        ));
    } else {
        lines.push("Trend: no prior quarter yet; first decision sets growth posture.".to_string());
    }

    let average_rate = market_average_rate(game);
    let rate_gap = game.player.rate_cents - average_rate;
    if rate_gap >= 0.4 {
        lines.push(format!(
            "Rate: {:.1}c above market average; margin improves, churn pressure rises.",
            rate_gap
        ));
    } else if rate_gap <= -0.4 {
        lines.push(format!(
            "Rate: {:.1}c below market average; demand improves, margin tightens.",
            rate_gap.abs()
        ));
    } else {
        lines.push(
            "Rate: close to market average; pricing is not the main swing factor.".to_string(),
        );
    }

    let capacity = game.player.customer_capacity(&game.market);
    let headroom = capacity - game.player.customers;
    let firm_generation_reserve = game.player.firm_generation_reserve_mwh(&game.market);
    if headroom < 60.0 {
        lines.push("Capacity: tight; marketing may create demand you cannot serve.".to_string());
    } else if headroom < 140.0 {
        lines.push(
            "Capacity: adequate but narrowing; another growth push needs expansion.".to_string(),
        );
    } else {
        lines.push("Capacity: enough headroom to absorb near-term customer growth.".to_string());
    }
    if firm_generation_reserve < 15.0 {
        lines.push(
            "Generation: firm reserve is thin; outages or growth could expose a shortfall."
                .to_string(),
        );
    }

    let leverage = game.player.debt_to_assets();
    if leverage > 0.85 {
        lines.push("Debt: high leverage; more borrowing risks lender pressure.".to_string());
    } else if leverage < 0.55 {
        lines.push(
            "Debt: borrowing capacity remains available for projects or acquisitions.".to_string(),
        );
    } else {
        lines.push("Debt: usable but no longer cheap; balance growth with solvency.".to_string());
    }

    if game.player.reliability < 0.74 {
        lines.push("Reliability: below target; maintenance has strategic value now.".to_string());
    } else if game.player.reliability > 0.88 {
        lines.push(
            "Reliability: strong; expansion can be prioritized if capacity is available."
                .to_string(),
        );
    }

    lines
}

fn action_effect_lines(game: &Game) -> Vec<String> {
    let mut lines = vec![
        "build gen [MWh]    larger projects cost more, but less per MWh".to_string(),
        project_quote("gen", GENERATION_PROJECT_CAPACITY_MWH),
        project_quote("gen", 400.0),
        "build lines [cust] larger projects cost more, but less per customer".to_string(),
        project_quote("lines", DISTRIBUTION_PROJECT_CAPACITY),
        project_quote("lines", 600.0),
        "marketing 4000  -$4.0k cash now | more customer capture this quarter; decays fast"
            .to_string(),
        "rate up 2      +2.0c/kWh | rate down 1 -1.0c | rate 10.0 sets target".to_string(),
        "stock issue [amt]   no fixed cap; larger sales face discounts and fees".to_string(),
        "buyback [amt]       limited by cash and public float; pays premium".to_string(),
        "debt [amt]          limited by borrowing room shown in Financials".to_string(),
        "repay [amt]         limited by cash and outstanding debt".to_string(),
        "maintenance     immediate reliability/reputation gain; best before outages compound"
            .to_string(),
    ];

    if let Some((index, price)) = cheapest_competitor(game) {
        if game.competitors.len() == 1 {
            lines.push("buy            final rival acquisition blocked by regulator".to_string());
        } else if game.acquisition_cooldown > 0 {
            lines.push(format!(
                "buy {}          unavailable for {} more quarter(s) during integration",
                index + 1,
                game.acquisition_cooldown
            ));
        } else {
            lines.push(format!(
                "buy {}          -{} cash plus assumed debt | immediate customers and capacity",
                index + 1,
                money(price)
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
                "  build gen {:>4.0}    {:>7} | {}q | {:>5}/MWh/q",
                size,
                money(cost),
                generation_project_duration(size),
                money(cost / size)
            )
        }
        "lines" => {
            let cost = distribution_project_cost(size);
            format!(
                "  build lines {:>4.0}  {:>7} | {}q | {:>5}/customer",
                size,
                money(cost),
                distribution_project_duration(size),
                money(cost / size)
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
    let decorated = if title.is_empty() {
        "─".to_string()
    } else {
        format!("─ {title} ")
    };
    let fill = (SCREEN_WIDTH - 2).saturating_sub(decorated.chars().count());
    println!("┌{}{}┐", decorated, "─".repeat(fill));
}

fn print_bottom_border() {
    println!("└{}┘", "─".repeat(SCREEN_WIDTH - 2));
}

fn print_box_line(line: &str) {
    let fitted = fit_line(line, CONTENT_WIDTH);
    let padding = CONTENT_WIDTH.saturating_sub(fitted.chars().count());
    println!("│ {}{} │", fitted, " ".repeat(padding));
}

fn fit_line(line: &str, width: usize) -> String {
    if line.chars().count() <= width {
        return line.to_string();
    }

    let keep = width.saturating_sub(3);
    let mut truncated = line.chars().take(keep).collect::<String>();
    truncated.push_str("...");
    truncated
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
}
