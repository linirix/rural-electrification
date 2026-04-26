use std::{
    io::{self, Write},
    sync::OnceLock,
};

#[cfg(test)]
use crate::sim::ActiveShock;
use crate::sim::{
    ADJACENT_EXPANSION_ADDRESSABLE_CUSTOMERS, ADJACENT_EXPANSION_INITIAL_CUSTOMERS,
    DISTRIBUTION_PROJECT_CAPACITY, Decision, GENERATION_PROJECT_CAPACITY_MWH, Game,
    MAX_DISTRIBUTION_PROJECT_CUSTOMERS, MAX_GENERATION_PROJECT_MWH, MAX_PUBLIC_RATE_PREMIUM_CENTS,
    MIN_DISTRIBUTION_PROJECT_CUSTOMERS, MIN_GENERATION_PROJECT_MWH, Outcome, OutcomeKind,
    QuarterReport, ShockKind, Utility, distribution_project_cost, distribution_project_duration,
    generation_project_cost, generation_project_duration,
    generation_reliability_after_new_capacity, money, public_rate_tolerance,
};

mod command;
mod preview;
mod render;
mod rivals;
mod screens;
#[cfg(test)]
mod tests;

use command::handle_command;
use render::{clear_screen, print_box, styled};
use screens::{
    print_board, print_competitors, print_help, print_notice, print_outcome, print_report,
    print_status,
};

const MIN_SCREEN_WIDTH: usize = 88;
const DEFAULT_SCREEN_WIDTH: usize = 118;
const MAX_SCREEN_WIDTH: usize = 124;
const COLUMN_GAP: usize = 2;
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
                    if !outcome.can_continue {
                        break;
                    }
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
            CommandResult::Preview(lines) => {
                clear_screen();
                print_status(&game);
                println!();
                print_box("Command Preview", &lines);
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
    Preview(Vec<String>),
    Quit,
}
