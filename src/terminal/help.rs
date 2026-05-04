use super::render::*;
use super::screens::{FramedScreen, print_subscreen_frame};
use super::*;

pub(super) fn print_help() {
    print_box(
        "Command Reference",
        &[
            "Type the command shown at left; use preview <command> before major actions."
                .to_string(),
            "Return follows the screen navigation strip unless you are entering a command."
                .to_string(),
        ],
    );
    print_box_pair(
        "Build + Operate",
        &[
            help_command_line("build gen [MWh]", "start generation project"),
            help_command_line("build lines [accounts]", "expand distribution"),
            help_command_line("marketing [amount]", "buy growth and reputation"),
            help_command_line("maint [amount]", "fund reliability work"),
            help_command_line("hire maint 85%", "automate service work"),
            help_command_line("hire marketing 90", "automate reputation work"),
            help_command_line("fire maint/marketing", "dismiss manager"),
            help_command_line("expand", "enter adjacent territory"),
            help_command_line("rate 10.0", "set target rate"),
            help_command_line("rate up|down [cents]", "adjust rate"),
            help_command_line("preview <command>", "inspect first"),
        ],
        "Capital",
        &[
            help_command_line("issue [amount]", "issue stock"),
            help_command_line("buyback [amount]", "repurchase shares"),
            help_command_line("dividend [amount]", "pay owners pro rata"),
            help_command_line("borrow [amount|max]", "raise debt"),
            help_command_line("repay [amount|max]", "pay debt down"),
            help_command_line("diligence <number>", "reveal deal terms"),
            help_command_line("buy <number>", "acquire rival"),
            help_command_line("stock issue/buyback", "aliases"),
            help_command_line("debt / debt repay", "aliases"),
        ],
    );
    print_box_pair(
        "Navigation",
        &[
            help_command_line("status / s", "show dashboard"),
            help_command_line("rivals", "competitor detail"),
            help_command_line("board", "objectives"),
            help_command_line("region", "regional mandate"),
            help_command_line("report", "last quarter detail"),
            help_command_line("preview / quote", "inspect command"),
            help_command_line("Return", "cycle screens / return"),
            help_command_line("next / n / end", "finish quarter"),
            help_command_line("continue", "post-review play"),
            help_command_line("sandbox", "disable board reviews"),
            help_command_line("save [name]", "write save file"),
            help_command_line("load [name]", "restore save file"),
            help_command_line("help / ?", "command reference"),
            help_command_line("quit / exit", "leave game"),
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
            "For marketing/maintenance, 1-9 means $1k-$9k.".to_string(),
        ],
    );
}

pub(super) fn print_help_screen(game: &Game) {
    print_subscreen_frame(game, FramedScreen::Help);
    print_help();
}

pub(super) fn help_command_line(
    command: impl std::fmt::Display,
    effect: impl std::fmt::Display,
) -> String {
    single_command_line_with_width(command, effect, 23)
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
