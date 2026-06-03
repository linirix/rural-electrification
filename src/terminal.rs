use std::{io, sync::OnceLock};

#[cfg(test)]
use crate::sim::ActiveShock;
use crate::sim::{
    ACQUISITION_STRESS_NOTICE, ACQUISITION_STRESS_STRAINED,
    ADJACENT_EXPANSION_ADDRESSABLE_CUSTOMERS, ADJACENT_EXPANSION_INITIAL_CUSTOMERS,
    AcquisitionTerms, DEFAULT_MAINTENANCE_MANAGER_TARGET, DEFAULT_MARKETING_MANAGER_TARGET,
    DISTRIBUTION_PROJECT_CAPACITY, Decision, GENERATION_PROJECT_CAPACITY_MWH, Game,
    MAINTENANCE_MANAGER_PROFIT_SHARE, MARKETING_MANAGER_PROFIT_SHARE,
    MAX_DISTRIBUTION_PROJECT_CUSTOMERS, MAX_GENERATION_PROJECT_MWH, MAX_PUBLIC_RATE_PREMIUM_CENTS,
    MIN_DISTRIBUTION_PROJECT_CUSTOMERS, MIN_GENERATION_PROJECT_MWH, Outcome, OutcomeKind,
    QuarterReport, REGIONAL_MANDATE_EXPANSION_TARGET, REGIONAL_MANDATE_LEVERAGE_LIMIT,
    REGIONAL_MANDATE_QUARTER, REGIONAL_MANDATE_RELIABILITY_TARGET, REGIONAL_MANDATE_SHARE_TARGET,
    REVIEW_MIN_INTEREST_COVERAGE, REVIEW_MIN_RATE_SUPPORT_RATIO, REVIEW_MIN_RELIABILITY, ShockKind,
    Utility, distribution_project_cost, distribution_project_duration, generation_project_cost,
    generation_project_duration, generation_reliability_after_new_capacity, money,
    public_rate_tolerance,
};
use rustyline::{DefaultEditor, error::ReadlineError};

mod board;
mod command;
mod help;
mod outcomes;
mod preview;
mod render;
mod report;
mod rivals;
mod screens;
mod start;
#[cfg(test)]
mod tests;

use command::{apply, handle_command, is_known_command, parse_decision};
use help::print_help_screen;
use outcomes::{print_outcome, print_quit_summary};
use render::{clear_screen, print_box, styled};
use report::print_report_screen;
use screens::{print_board_screen, print_notice, print_rivals_screen, print_status};
use start::print_start_screen;

const MIN_SCREEN_WIDTH: usize = 88;
const DEFAULT_SCREEN_WIDTH: usize = 132;
const MAX_SCREEN_WIDTH: usize = 148;
const COLUMN_GAP: usize = 2;
const SHARE_TARGET: f64 = 0.45;
const RELIABILITY_TARGET: f64 = REVIEW_MIN_RELIABILITY;
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

fn acquisition_projected_reliability(
    game: &Game,
    competitor: &Utility,
    terms: &AcquisitionTerms,
) -> f64 {
    let player_weight = game.player.generation_capacity_mwh.max(0.0);
    let acquired_weight = (terms.acquired_generation_capacity_mwh * 0.70).max(0.0);
    let combined_weight = player_weight + acquired_weight;
    let weighted = if combined_weight <= 0.0 {
        game.player.reliability
    } else {
        (game.player.reliability * player_weight + competitor.reliability * acquired_weight)
            / combined_weight
    };
    (weighted - 0.035).clamp(0.35, 0.98)
}

fn acquisition_projected_integration_burden(game: &Game, terms: &AcquisitionTerms) -> f64 {
    let post_customers = (game.player.customers + terms.acquired_customers).max(1.0);
    let deal_share = (terms.acquired_customers / post_customers).clamp(0.0, 1.0);
    (game.integration_strain + deal_share * 1.35).clamp(0.0, 1.6)
}

fn acquisition_service_tone(projected_reliability: f64) -> &'static str {
    if projected_reliability < REVIEW_MIN_RELIABILITY {
        BOLD_RED
    } else if projected_reliability < REVIEW_MIN_RELIABILITY + 0.04 {
        BOLD_YELLOW
    } else {
        GREEN
    }
}

fn acquisition_service_cushion_text(projected_reliability: f64) -> String {
    let cushion = projected_reliability - REVIEW_MIN_RELIABILITY;
    if cushion >= 0.0 {
        format!("{:.1} pts cushion", cushion * 100.0)
    } else {
        format!("{:.1} pts short", cushion.abs() * 100.0)
    }
}

fn acquisition_integration_burden_label(value: f64) -> &'static str {
    if value >= 0.70 {
        "severe"
    } else if value >= 0.42 {
        "heavy"
    } else if value >= 0.20 {
        "moderate"
    } else {
        "light"
    }
}

fn acquisition_integration_burden_tone(value: f64) -> &'static str {
    if value >= 0.70 {
        BOLD_RED
    } else if value >= 0.42 {
        BOLD_YELLOW
    } else if value >= 0.20 {
        YELLOW
    } else {
        GREEN
    }
}

pub fn run() -> io::Result<()> {
    run_with_game(Game::new())
}

pub fn run_with_game(game: Game) -> io::Result<()> {
    let mut game = game;
    let mut current_screen = TerminalScreen::Start;
    let mut screen_entry = ScreenEntry::Cycle;
    let mut pending_confirmation: Option<PendingConfirmation> = None;
    let mut command_reader = DefaultEditor::new().map_err(io::Error::other)?;

    clear_screen();
    print_start_screen(&game);

    loop {
        let prompt = format!("\n{}> ", game.date_label());
        let Some(line) = read_command_line(&mut command_reader, &prompt)? else {
            break;
        };

        if is_blank_input(&line) {
            current_screen = current_screen.next_on_enter(screen_entry);
            screen_entry = ScreenEntry::Cycle;
            render_terminal_screen(&game, current_screen);
            continue;
        }

        let command = line.trim();
        if command.is_empty() {
            continue;
        }

        // Set when a recognized command typed at a y/n prompt cancels the
        // pending action and runs in its place, so the supersession can be
        // surfaced once the new command has rendered.
        let mut superseded_pending: Option<String> = None;

        if let Some(pending) = pending_confirmation.take() {
            match confirmation_response(command) {
                ConfirmationResponse::Confirm => {
                    let result = apply(&mut game, pending.decision);
                    if render_command_result(&game, result, &mut current_screen, &mut screen_entry)
                    {
                        break;
                    }
                    continue;
                }
                ConfirmationResponse::Cancel => {
                    clear_screen();
                    print_status(&game);
                    current_screen = TerminalScreen::Dashboard;
                    screen_entry = ScreenEntry::Cycle;
                    print_notice(&format!("Cancelled '{}'.", pending.command));
                    continue;
                }
                ConfirmationResponse::Other if is_known_command(command) => {
                    // A recognized command typed at the prompt supersedes the
                    // pending action: cancel it and fall through to run the new
                    // command (which raises its own confirmation if it is itself
                    // high-impact). This avoids trapping the player in a modal
                    // that silently swallows every non-y/n input.
                    superseded_pending = Some(pending.command.clone());
                }
                ConfirmationResponse::Other => {
                    clear_screen();
                    print_status(&game);
                    println!();
                    print_box("Confirm Action", &pending.lines);
                    print_notice("Type 'y' to confirm this action, or 'n' to cancel it.");
                    pending_confirmation = Some(pending);
                    continue;
                }
            }
        }

        if should_record_command_history(command) {
            command_reader
                .add_history_entry(command)
                .map_err(io::Error::other)?;
        }

        if let Some(pending) = confirmation_for_command(&game, command) {
            clear_screen();
            print_status(&game);
            println!();
            print_box("Confirm Action", &pending.lines);
            if let Some(name) = &superseded_pending {
                print_notice(&format!("Cancelled '{name}' for this action."));
            }
            pending_confirmation = Some(pending);
            continue;
        }

        let result = handle_command(&mut game, command);
        let should_quit =
            render_command_result(&game, result, &mut current_screen, &mut screen_entry);
        if let Some(name) = &superseded_pending {
            print_notice(&format!("Cancelled '{name}' to run '{command}'."));
        }
        if should_quit {
            break;
        }
    }

    Ok(())
}

fn render_command_result(
    game: &Game,
    result: CommandResult,
    current_screen: &mut TerminalScreen,
    screen_entry: &mut ScreenEntry,
) -> bool {
    match result {
        CommandResult::Continue(message) => {
            clear_screen();
            print_status(game);
            *current_screen = TerminalScreen::Dashboard;
            *screen_entry = ScreenEntry::Cycle;
            if !message.is_empty() {
                print_notice(&message);
            }
        }
        CommandResult::Advanced(report) => {
            clear_screen();
            debug_assert!(!report.label.is_empty());
            print_status(game);
            *current_screen = TerminalScreen::Dashboard;
            *screen_entry = ScreenEntry::Cycle;
            if let Some(outcome) = &game.outcome {
                print_outcome(outcome);
                if !outcome.can_continue {
                    return true;
                }
            }
        }
        CommandResult::ShowStatus => {
            clear_screen();
            print_status(game);
            *current_screen = TerminalScreen::Dashboard;
            *screen_entry = ScreenEntry::Cycle;
        }
        CommandResult::ShowHelp => {
            clear_screen();
            print_help_screen(game);
            *current_screen = TerminalScreen::Help;
            *screen_entry = ScreenEntry::DirectCommand;
        }
        CommandResult::ShowRivals => {
            clear_screen();
            print_rivals_screen(game);
            *current_screen = TerminalScreen::Rivals;
            *screen_entry = ScreenEntry::DirectCommand;
        }
        CommandResult::ShowBoard => {
            clear_screen();
            print_board_screen(game);
            *current_screen = TerminalScreen::Board;
            *screen_entry = ScreenEntry::DirectCommand;
        }
        CommandResult::ShowReport => {
            clear_screen();
            print_report_screen(game);
            *current_screen = TerminalScreen::Report;
            *screen_entry = ScreenEntry::DirectCommand;
        }
        CommandResult::Preview(lines) => {
            clear_screen();
            print_status(game);
            println!();
            print_box("Command Preview", &lines);
            *current_screen = TerminalScreen::Preview;
            *screen_entry = ScreenEntry::DirectCommand;
        }
        CommandResult::Quit => {
            print_quit_summary(game);
            return true;
        }
    }

    false
}

struct PendingConfirmation {
    command: String,
    decision: Decision,
    lines: Vec<String>,
}

enum ConfirmationResponse {
    Confirm,
    Cancel,
    Other,
}

fn confirmation_response(command: &str) -> ConfirmationResponse {
    match command.trim().to_ascii_lowercase().as_str() {
        "y" | "yes" | "confirm" => ConfirmationResponse::Confirm,
        "n" | "no" | "cancel" => ConfirmationResponse::Cancel,
        _ => ConfirmationResponse::Other,
    }
}

fn confirmation_for_command(game: &Game, command: &str) -> Option<PendingConfirmation> {
    let parts = command.split_whitespace().collect::<Vec<_>>();
    let decision = parse_decision(game, &parts).ok()?;
    if !decision_requires_confirmation(game, &decision) {
        return None;
    }

    let mut preview_parts = vec!["preview"];
    preview_parts.extend(parts.iter().copied());
    let mut lines = match preview::preview_command(game, &preview_parts) {
        CommandResult::Preview(lines) => lines,
        _ => Vec::new(),
    };
    lines.push(styled(
        BOLD_YELLOW,
        "Type 'y' to commit this action, or 'n' to cancel.",
    ));

    Some(PendingConfirmation {
        command: command.to_string(),
        decision,
        lines,
    })
}

fn decision_requires_confirmation(game: &Game, decision: &Decision) -> bool {
    match decision {
        Decision::Acquire { .. } | Decision::EnterAdjacentMarket => true,
        Decision::AdjustRate { delta_cents } => delta_cents.abs() >= 1.0,
        Decision::DeclareDividend { amount } => *amount >= game.player.cash.max(1.0) * 0.05,
        Decision::IssueStock { amount } | Decision::BuyBackStock { amount } => *amount >= 20_000.0,
        Decision::Borrow { amount } => *amount >= 40_000.0,
        _ => false,
    }
}

fn read_command_line(editor: &mut DefaultEditor, prompt: &str) -> io::Result<Option<String>> {
    match editor.readline(prompt) {
        Ok(line) => Ok(Some(line)),
        Err(ReadlineError::Interrupted | ReadlineError::Eof) => Ok(None),
        Err(error) => Err(io::Error::other(error)),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ScreenEntry {
    Cycle,
    DirectCommand,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TerminalScreen {
    Start,
    Dashboard,
    Report,
    Rivals,
    Board,
    Help,
    Preview,
}

impl TerminalScreen {
    fn next_on_enter(self, entry: ScreenEntry) -> Self {
        match (self, entry) {
            (
                Self::Report | Self::Rivals | Self::Board | Self::Help,
                ScreenEntry::DirectCommand,
            ) => Self::Dashboard,
            (Self::Preview, _) => Self::Dashboard,
            (Self::Start, _) => Self::Dashboard,
            (Self::Dashboard, _) => Self::Report,
            (Self::Report, ScreenEntry::Cycle) => Self::Rivals,
            (Self::Rivals, ScreenEntry::Cycle) => Self::Board,
            (Self::Board, ScreenEntry::Cycle) => Self::Help,
            (Self::Help, ScreenEntry::Cycle) => Self::Dashboard,
        }
    }
}

fn is_blank_input(raw_line: &str) -> bool {
    let input = raw_line.trim_end_matches(['\n', '\r']);
    input.chars().all(char::is_whitespace)
}

fn should_record_command_history(command: &str) -> bool {
    !command.trim().is_empty()
}

fn render_terminal_screen(game: &Game, screen: TerminalScreen) {
    clear_screen();
    match screen {
        TerminalScreen::Start => print_start_screen(game),
        TerminalScreen::Dashboard | TerminalScreen::Preview => print_status(game),
        TerminalScreen::Report => print_report_screen(game),
        TerminalScreen::Rivals => print_rivals_screen(game),
        TerminalScreen::Board => print_board_screen(game),
        TerminalScreen::Help => print_help_screen(game),
    }
}

enum CommandResult {
    Continue(String),
    Advanced(QuarterReport),
    ShowStatus,
    ShowHelp,
    ShowRivals,
    ShowBoard,
    ShowReport,
    Preview(Vec<String>),
    Quit,
}
