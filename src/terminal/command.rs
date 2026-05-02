use std::{
    fs,
    path::{Path, PathBuf},
};

use super::preview::preview_command;
use super::render::competitor_index_for_rank;
use super::*;

pub(super) fn handle_command(game: &mut Game, command: &str) -> CommandResult {
    let parts = command.split_whitespace().collect::<Vec<_>>();
    let Some(first) = parts.first().copied() else {
        return CommandResult::Continue(String::new());
    };

    match first {
        "help" | "?" => CommandResult::ShowHelp,
        "status" | "s" => CommandResult::ShowStatus,
        "next" | "n" | "end" => CommandResult::Advanced(game.advance_quarter()),
        "save" => {
            if parts.len() > 2 {
                CommandResult::Continue(
                    "Cannot save: use 'save [name]' with a single slot name.".to_string(),
                )
            } else {
                match save_game(game, parts.get(1).copied()) {
                    Ok(message) => CommandResult::Continue(message),
                    Err(message) => CommandResult::Continue(format!("Cannot save: {message}")),
                }
            }
        }
        "load" => {
            if parts.len() > 2 {
                CommandResult::Continue(
                    "Cannot load: use 'load [name]' with a single slot name.".to_string(),
                )
            } else {
                match load_game(parts.get(1).copied()) {
                    Ok((loaded, message)) => {
                        *game = loaded;
                        CommandResult::Continue(message)
                    }
                    Err(message) => CommandResult::Continue(format!("Cannot load: {message}")),
                }
            }
        }
        "continue" | "resume" => match game.continue_after_review() {
            Ok(message) => CommandResult::Continue(message),
            Err(message) => CommandResult::Continue(format!("Cannot continue: {message}")),
        },
        "sandbox" => match game.enable_sandbox_mode() {
            Ok(message) => CommandResult::Continue(message),
            Err(message) => CommandResult::Continue(format!("Cannot enable sandbox: {message}")),
        },
        "quit" | "exit" => CommandResult::Quit,
        "preview" | "quote" | "plan" => preview_command(game, &parts),
        "build" | "marketing" | "market" | "advertise" | "issue" | "stock" | "equity"
        | "buyback" | "repurchase" | "dividend" | "dividends" | "debt" | "borrow" | "loan"
        | "repay" | "paydown" | "buy" | "acquire" | "diligence" | "dilig" | "inspect"
        | "expand" | "adjacent" | "territory" | "rate" | "maintenance" | "maint" | "maintain"
        | "reliability" | "hire" | "fire" => match parse_decision(game, &parts) {
            Ok(decision) => apply(game, decision),
            Err(message) => CommandResult::Continue(message),
        },
        "competitors" | "rivals" => CommandResult::ShowRivals,
        "report" | "reports" | "result" | "results" | "events" => CommandResult::ShowReport,
        "board" | "goals" | "objectives" | "milestones" | "region" | "regional" => {
            CommandResult::ShowBoard
        }
        _ => CommandResult::Continue(format!("Unknown command '{first}'. Type 'help'.")),
    }
}

pub(super) fn apply(game: &mut Game, decision: Decision) -> CommandResult {
    let preview_hint = preview_hint_for_direct_decision(&decision);
    match game.apply_decision(decision) {
        Ok(mut message) => {
            if let Some(hint) = preview_hint {
                message.push(' ');
                message.push_str(hint);
            }
            CommandResult::Continue(message)
        }
        Err(error) => CommandResult::Continue(format!("Cannot do that: {error}")),
    }
}

fn preview_hint_for_direct_decision(decision: &Decision) -> Option<&'static str> {
    let high_impact = match decision {
        Decision::Acquire { .. } | Decision::EnterAdjacentMarket => true,
        Decision::IssueStock { amount }
        | Decision::BuyBackStock { amount }
        | Decision::DeclareDividend { amount }
        | Decision::Borrow { amount } => *amount >= 20_000.0,
        Decision::BuildGeneration { capacity_mwh } => *capacity_mwh >= 300.0,
        Decision::BuildDistribution { customer_capacity } => *customer_capacity >= 500.0,
        Decision::AdjustRate { delta_cents } => delta_cents.abs() >= 1.0,
        _ => false,
    };

    high_impact.then_some("Tip: use 'preview <command>' before similar high-impact moves.")
}

pub(super) fn save_game(game: &Game, name: Option<&str>) -> Result<String, String> {
    save_game_to_dir(game, name.unwrap_or("autosave"), &default_save_dir()?)
}

pub(super) fn load_game(name: Option<&str>) -> Result<(Game, String), String> {
    load_game_from_dir(name.unwrap_or("autosave"), &default_save_dir()?)
}

pub(super) fn default_save_dir() -> Result<PathBuf, String> {
    let home = std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .ok_or_else(|| "could not find a home directory for save files".to_string())?;
    Ok(PathBuf::from(home).join(".electrification"))
}

pub(super) fn save_game_to_dir(game: &Game, raw_name: &str, root: &Path) -> Result<String, String> {
    let path = save_file_path(root, raw_name)?;
    fs::create_dir_all(root).map_err(|error| {
        format!(
            "could not create save directory {}: {error}",
            root.display()
        )
    })?;
    let json = serde_json::to_string_pretty(game)
        .map_err(|error| format!("could not serialize game: {error}"))?;
    fs::write(&path, json)
        .map_err(|error| format!("could not write {}: {error}", path.display()))?;
    Ok(format!("Saved game to {}.", path.display()))
}

pub(super) fn load_game_from_dir(raw_name: &str, root: &Path) -> Result<(Game, String), String> {
    let path = save_file_path(root, raw_name)?;
    let json = fs::read_to_string(&path)
        .map_err(|error| format!("could not read {}: {error}", path.display()))?;
    let mut game = serde_json::from_str::<Game>(&json)
        .map_err(|error| format!("could not parse {}: {error}", path.display()))?;
    game.normalize_after_load();
    Ok((game, format!("Loaded game from {}.", path.display())))
}

pub(super) fn save_file_path(root: &Path, raw_name: &str) -> Result<PathBuf, String> {
    let name = normalized_save_name(raw_name)?;
    Ok(root.join(format!("{name}.json")))
}

pub(super) fn normalized_save_name(raw_name: &str) -> Result<String, String> {
    let trimmed = raw_name.trim();
    let trimmed = trimmed.strip_suffix(".json").unwrap_or(trimmed);

    if trimmed.is_empty() {
        return Err("save name cannot be empty".to_string());
    }

    if trimmed == "." || trimmed == ".." {
        return Err("save name cannot be a path".to_string());
    }

    if !trimmed
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
    {
        return Err("save names may use letters, numbers, '-' and '_' only".to_string());
    }

    Ok(trimmed.to_string())
}

pub(super) fn money_amount(value: Option<&str>, default: f64, usage: &str) -> Result<f64, String> {
    match value {
        Some(value) => parse_money(value)
            .filter(|amount| amount.is_finite() && *amount > 0.0)
            .ok_or_else(|| format!("Use '{usage}' with a positive amount.")),
        None => Ok(default),
    }
}

pub(super) fn operating_spend_amount(
    value: Option<&str>,
    default: f64,
    usage: &str,
) -> Result<f64, String> {
    match value {
        Some(value) if is_single_digit_money_shorthand(value) => parse_money(value)
            .map(|amount| amount * 1_000.0)
            .filter(|amount| amount.is_finite() && *amount > 0.0)
            .ok_or_else(|| format!("Use '{usage}' with a positive amount.")),
        Some(value) => parse_money(value)
            .filter(|amount| amount.is_finite() && *amount > 0.0)
            .ok_or_else(|| format!("Use '{usage}' with a positive amount.")),
        None => Ok(default),
    }
}

fn is_single_digit_money_shorthand(value: &str) -> bool {
    let trimmed = value.trim().trim_start_matches('$');
    trimmed.len() == 1 && matches!(trimmed.as_bytes()[0], b'1'..=b'9')
}

pub(super) fn borrow_amount(game: &Game, value: Option<&str>) -> Result<f64, String> {
    match value {
        Some("max") => Ok(game.borrowing_room()),
        Some(value) => parse_money(value).ok_or_else(|| "Use 'borrow [amount|max]'.".to_string()),
        None => Ok(20_000.0),
    }
}

pub(super) fn repay_amount(game: &Game, value: Option<&str>) -> Result<f64, String> {
    match value {
        Some("max") => Ok(game.player.debt.min(game.player.cash)),
        Some(value) => parse_money(value).ok_or_else(|| "Use 'repay [amount|max]'.".to_string()),
        None => Ok(10_000.0),
    }
}

pub(super) fn parse_decision(game: &Game, parts: &[&str]) -> Result<Decision, String> {
    let Some(first) = parts.first().copied() else {
        return Err("Use 'preview <command>' or enter a command.".to_string());
    };

    match first {
        "build" => match parts.get(1).copied() {
            Some("gen") | Some("generator") | Some("generation") => {
                let capacity_mwh = match optional_number(parts.get(2).copied()) {
                    Ok(Some(size)) => size,
                    Ok(None) => GENERATION_PROJECT_CAPACITY_MWH,
                    Err(_) => {
                        return Err(
                            "Use 'build gen' or 'build gen 300' where the number is MWh/q."
                                .to_string(),
                        );
                    }
                };
                Ok(Decision::BuildGeneration { capacity_mwh })
            }
            Some("lines") | Some("line") | Some("distribution") | Some("wires") => {
                let customer_capacity = match optional_number(parts.get(2).copied()) {
                    Ok(Some(size)) => size,
                    Ok(None) => DISTRIBUTION_PROJECT_CAPACITY,
                    Err(_) => {
                        return Err(
                            "Use 'build lines' or 'build lines 500' where the number is customer capacity."
                                .to_string(),
                        );
                    }
                };
                Ok(Decision::BuildDistribution { customer_capacity })
            }
            _ => Err("Build what? Try 'build gen [MWh]' or 'build lines [customers]'.".to_string()),
        },
        "expand" | "adjacent" | "territory" => Ok(Decision::EnterAdjacentMarket),
        "marketing" | "market" | "advertise" => {
            let spend =
                operating_spend_amount(parts.get(1).copied(), 4_000.0, "marketing [amount]")?;
            Ok(Decision::Marketing { spend })
        }
        "issue" => {
            let amount = money_amount(parts.get(1).copied(), 20_000.0, "issue [amount]")?;
            Ok(Decision::IssueStock { amount })
        }
        "stock" | "equity" => match parts.get(1).copied() {
            Some("issue" | "sell") => {
                let amount = money_amount(parts.get(2).copied(), 20_000.0, "stock issue [amount]")?;
                Ok(Decision::IssueStock { amount })
            }
            Some("buyback" | "repurchase" | "buy") => {
                let amount =
                    money_amount(parts.get(2).copied(), 10_000.0, "stock buyback [amount]")?;
                Ok(Decision::BuyBackStock { amount })
            }
            Some(value) if parse_money(value).is_some() => {
                let amount = parse_money(value).unwrap_or(20_000.0);
                Ok(Decision::IssueStock { amount })
            }
            Some(_) => {
                Err("Use 'stock issue 20000', 'stock buyback 10000', or 'stock 20000'.".to_string())
            }
            _ => Ok(Decision::IssueStock { amount: 20_000.0 }),
        },
        "buyback" | "repurchase" => {
            let amount = money_amount(parts.get(1).copied(), 10_000.0, "buyback [amount]")?;
            Ok(Decision::BuyBackStock { amount })
        }
        "dividend" | "dividends" => {
            let amount = money_amount(parts.get(1).copied(), 5_000.0, "dividend [amount]")?;
            Ok(Decision::DeclareDividend { amount })
        }
        "debt" | "borrow" | "loan" => {
            if first == "debt" && matches!(parts.get(1).copied(), Some("repay" | "pay" | "down")) {
                let amount = repay_amount(game, parts.get(2).copied())?;
                return Ok(Decision::RepayDebt { amount });
            }
            let amount = borrow_amount(game, parts.get(1).copied())?;
            Ok(Decision::Borrow { amount })
        }
        "repay" | "paydown" => {
            let amount = repay_amount(game, parts.get(1).copied())?;
            Ok(Decision::RepayDebt { amount })
        }
        "buy" | "acquire" => {
            let Some(rank) = parts.get(1).and_then(|value| value.parse::<usize>().ok()) else {
                return Err("Use 'buy 1', 'buy 2', etc.".to_string());
            };
            if rank == 0 {
                return Err("Competitor numbers start at 1.".to_string());
            }
            let Some(competitor_index) = competitor_index_for_rank(game, rank) else {
                return Err(format!("No rival is ranked {rank} right now."));
            };
            Ok(Decision::Acquire { competitor_index })
        }
        "diligence" | "dilig" | "inspect" => {
            let Some(rank) = parts.get(1).and_then(|value| value.parse::<usize>().ok()) else {
                return Err("Use 'diligence 1', 'diligence 2', etc.".to_string());
            };
            if rank == 0 {
                return Err("Competitor numbers start at 1.".to_string());
            }
            let Some(competitor_index) = competitor_index_for_rank(game, rank) else {
                return Err(format!("No rival is ranked {rank} right now."));
            };
            Ok(Decision::Diligence { competitor_index })
        }
        "rate" => {
            let delta = match parse_rate_delta(parts, game.player.rate_cents) {
                Ok(Some(delta)) => delta,
                Ok(None) => {
                    return Err(format!(
                        "Current rate is {:.1}c/kWh. Use 'rate up 2', 'rate down 1', or 'rate 10.0'.",
                        game.player.rate_cents
                    ));
                }
                Err(message) => return Err(message),
            };
            Ok(Decision::AdjustRate { delta_cents: delta })
        }
        "maintenance" | "maint" | "maintain" | "reliability" => {
            let spend =
                operating_spend_amount(parts.get(1).copied(), 4_500.0, "maintenance [amount]")?;
            Ok(Decision::Maintenance { spend })
        }
        "hire" => match parts.get(1).copied() {
            Some("maintenance" | "maint" | "reliability") => {
                let target = manager_reliability_target(parts.get(2).copied())?;
                Ok(Decision::HireMaintenanceManager {
                    target_reliability: target,
                })
            }
            Some("marketing" | "market" | "advertise" | "reputation") => {
                let target = manager_reputation_target(parts.get(2).copied())?;
                Ok(Decision::HireMarketingManager {
                    target_reputation: target,
                })
            }
            _ => Err(
                "Hire which manager? Use 'hire maintenance [target%]' or 'hire marketing [target]'."
                    .to_string(),
            ),
        },
        "fire" => match parts.get(1).copied() {
            Some("maintenance" | "maint" | "reliability") => Ok(Decision::FireMaintenanceManager),
            Some("marketing" | "market" | "advertise" | "reputation") => {
                Ok(Decision::FireMarketingManager)
            }
            _ => Err("Fire which manager? Use 'fire maintenance' or 'fire marketing'.".to_string()),
        },
        _ => Err(format!("Unknown command '{first}'. Type 'help'.")),
    }
}

pub(super) fn parse_money(value: &str) -> Option<f64> {
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

pub(super) fn parse_number(value: &str) -> Option<f64> {
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

pub(super) fn optional_number(value: Option<&str>) -> Result<Option<f64>, ()> {
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

fn manager_reliability_target(value: Option<&str>) -> Result<f64, String> {
    let Some(value) = value else {
        return Ok(DEFAULT_MAINTENANCE_MANAGER_TARGET);
    };
    let trimmed = value.trim();
    let has_percent = trimmed.ends_with('%');
    let number = trimmed
        .trim_end_matches('%')
        .parse::<f64>()
        .map_err(|_| "Use 'hire maintenance 85%' or 'hire maintenance'.".to_string())?;
    let target = if has_percent || number > 1.0 {
        number / 100.0
    } else {
        number
    };
    if !(0.35..=0.98).contains(&target) {
        return Err("Maintenance manager target must be between 35% and 98%.".to_string());
    }
    Ok(target)
}

fn manager_reputation_target(value: Option<&str>) -> Result<f64, String> {
    let Some(value) = value else {
        return Ok(DEFAULT_MARKETING_MANAGER_TARGET);
    };
    let trimmed = value.trim();
    let has_percent = trimmed.ends_with('%');
    let number = trimmed
        .trim_end_matches('%')
        .parse::<f64>()
        .map_err(|_| "Use 'hire marketing 90' or 'hire marketing 90%'.".to_string())?;
    let target = if has_percent {
        number
    } else if number <= 1.0 {
        number * 100.0
    } else {
        number
    };
    if !(0.0..=100.0).contains(&target) {
        return Err("Marketing manager target must be between 0 and 100.".to_string());
    }
    Ok(target)
}

pub(super) fn parse_rate_delta(parts: &[&str], current_rate: f64) -> Result<Option<f64>, String> {
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

pub(super) fn rate_amount(value: Option<&str>) -> Result<f64, String> {
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

pub(super) fn parse_rate_number(value: &str) -> Result<f64, String> {
    match value.parse::<f64>() {
        Ok(number) if number.is_finite() => Ok(number),
        _ => Err("Use 'rate up 2', 'rate down 1', or 'rate 10.0'.".to_string()),
    }
}
