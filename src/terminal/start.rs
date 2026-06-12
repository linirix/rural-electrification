use super::render::*;
use super::*;

const START_SCREEN_COMMAND_WIDTH: usize = 18;

pub(super) fn print_start_screen(game: &Game) {
    println!("{}", start_screen_banner_line());
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

pub(super) fn start_screen_banner_line() -> String {
    centered_line(styled(BOLD_CYAN, "more corgi's rural electrification!"))
}

pub(super) fn start_screen_overview_lines(game: &Game) -> Vec<String> {
    vec![
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
        start_screen_command_line("next / n", "finish the quarter"),
        start_screen_command_line("preview <cmd>", "inspect before acting"),
        start_screen_command_line(
            "rate / build",
            format!(
                "set price; add gen/lines near {:.1}c",
                game.market.standard_rate_cents
            ),
        ),
        start_screen_command_line("marketing / maint", "growth; reliability"),
        start_screen_command_line("hire / fire", "automate marketing or upkeep"),
        start_screen_command_line("borrow / issue", "debt, equity, buybacks"),
        start_screen_command_line("rivals / board", "competitors; objectives"),
        start_screen_command_line("save / load", "keep long campaigns"),
        start_screen_command_line("sandbox", "disable board reviews"),
    ]
}

pub(super) fn start_screen_begin_lines() -> Vec<String> {
    vec![
        "Press Return to open the main dashboard.".to_string(),
        "From the dashboard, Return cycles: Report -> Rivals -> Board -> Help -> Dashboard."
            .to_string(),
        "From a screen opened by command, Return goes back to the dashboard.".to_string(),
        "Type sandbox to play without board review constraints.".to_string(),
        "Type help at any time for the full command reference.".to_string(),
    ]
}

fn centered_line(line: String) -> String {
    let padding = content_width().saturating_sub(visible_width(&line)) / 2;
    format!("{}{}", " ".repeat(padding), line)
}

fn start_screen_command_line(
    command: impl std::fmt::Display,
    effect: impl std::fmt::Display,
) -> String {
    single_command_line_with_width(command, effect, START_SCREEN_COMMAND_WIDTH)
}

fn single_command_line_with_width(
    command: impl std::fmt::Display,
    effect: impl std::fmt::Display,
    command_width: usize,
) -> String {
    let command = command.to_string();
    let padding = command_width.saturating_sub(visible_width(&command));
    format!(
        "{}{}  {}",
        styled(BOLD_CYAN, command),
        " ".repeat(padding),
        effect
    )
}
