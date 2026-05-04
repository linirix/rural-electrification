use super::render::*;
use super::screens::{FramedScreen, print_subscreen_frame};
use super::*;

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

pub(super) fn print_report_screen(game: &Game) {
    print_subscreen_frame(game, FramedScreen::Report);
    print_last_report(game);
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
