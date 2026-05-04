use super::render::*;
use super::*;

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

pub(super) fn print_quit_summary(game: &Game) {
    println!();
    println!("{}", quit_summary_line(game));
}

pub(super) fn quit_summary_line(game: &Game) -> String {
    let outcome = game
        .outcome
        .as_ref()
        .map(|outcome| outcome.headline.as_str())
        .unwrap_or("in progress");
    format!(
        "{} {} | {} {:.0}% | {} {:.0} | {} {:.0}% | {} {:.0}% | {} {} | {} {:.1}% | {} {}",
        styled(BOLD_CYAN, "Summary:"),
        styled(BOLD, outcome),
        muted("share"),
        game.market_share() * 100.0,
        muted("customers"),
        game.player.customers,
        muted("reliability"),
        game.player.reliability * 100.0,
        muted("debt/assets"),
        game.player.debt_to_assets() * 100.0,
        muted("cash"),
        styled(cash_tone(game.player.cash), money(game.player.cash)),
        muted("founder"),
        game.player_ownership() * 100.0,
        muted("personal wealth"),
        styled(BOLD_GREEN, money(game.player_wealth()))
    )
}
