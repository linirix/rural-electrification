use super::*;

pub(super) fn cheapest_diligenced_competitor(game: &Game) -> Option<(usize, f64)> {
    (0..game.competitors.len())
        .filter(|index| game.has_diligence(*index))
        .map(|index| (index, game.acquisition_price(index)))
        .min_by(|left, right| left.1.total_cmp(&right.1))
}

pub(super) fn cheapest_diligence_target(game: &Game) -> Option<(usize, f64)> {
    (0..game.competitors.len())
        .filter(|index| !game.has_diligence(*index))
        .filter_map(|index| game.diligence_cost(index).map(|cost| (index, cost)))
        .min_by(|left, right| left.1.total_cmp(&right.1))
}

pub(super) fn cheapest_public_acquisition_target(game: &Game) -> Option<(usize, f64)> {
    (0..game.competitors.len())
        .filter(|index| !game.has_diligence(*index))
        .filter_map(|index| {
            game.public_acquisition_estimate(index)
                .map(|terms| (index, terms.price))
        })
        .min_by(|left, right| left.1.total_cmp(&right.1))
}

pub(super) fn market_average_rate(game: &Game) -> f64 {
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

pub(super) fn shorten_plain(value: &str, width: usize) -> String {
    if value.chars().count() <= width {
        return value.to_string();
    }

    let keep = width.saturating_sub(1);
    let mut shortened = value.chars().take(keep).collect::<String>();
    shortened.push('…');
    shortened
}

pub(super) fn styled(style: &str, value: impl std::fmt::Display) -> String {
    if ansi_enabled() {
        format!("{style}{value}{RESET}")
    } else {
        value.to_string()
    }
}

pub(super) fn muted(value: impl std::fmt::Display) -> String {
    styled(DIM, value)
}

pub(super) fn ansi(style: &'static str) -> &'static str {
    if ansi_enabled() { style } else { "" }
}

pub(super) fn ansi_enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| {
        let term = std::env::var("TERM").ok();
        should_use_ansi(std::env::var_os("NO_COLOR").is_some(), term.as_deref())
    })
}

pub(super) fn terminal_control_enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| {
        std::env::var("TERM")
            .map(|term| term != "dumb")
            .unwrap_or(true)
    })
}

pub(super) fn should_use_ansi(no_color_set: bool, term: Option<&str>) -> bool {
    !no_color_set && term != Some("dumb")
}

pub(super) fn action_line(
    command: impl std::fmt::Display,
    effect: impl std::fmt::Display,
) -> String {
    format!(
        "{}  {}",
        styled(BOLD_CYAN, format!("{command:<22}")),
        effect
    )
}

pub(super) fn signal_line(label: &str, tone: &str, body: impl std::fmt::Display) -> String {
    format!("{} {}", styled(tone, format!("{label}:")), body)
}

pub(super) fn macro_credit_tone(game: &Game) -> &'static str {
    let rate = game.macro_state.benchmark_credit_rate();
    if rate >= 0.105 {
        BOLD_RED
    } else if rate >= 0.080 {
        BOLD_YELLOW
    } else {
        BOLD_GREEN
    }
}

pub(super) fn interest_rate_tone(rate: f64) -> &'static str {
    if rate >= 0.14 {
        BOLD_RED
    } else if rate >= 0.09 {
        BOLD_YELLOW
    } else {
        BOLD_GREEN
    }
}

pub(super) fn debt_cap_tone(limit: f64) -> &'static str {
    if limit <= 0.75 {
        BOLD_RED
    } else if limit <= 0.88 {
        BOLD_YELLOW
    } else {
        BOLD_GREEN
    }
}

pub(super) fn demand_tone(index: f64) -> &'static str {
    if index <= -0.25 {
        BOLD_RED
    } else if index < 0.20 {
        BOLD_YELLOW
    } else {
        BOLD_GREEN
    }
}

pub(super) fn cost_pressure_tone(pressure: f64) -> &'static str {
    if pressure >= 0.030 {
        BOLD_RED
    } else if pressure > 0.004 {
        BOLD_YELLOW
    } else {
        BOLD_GREEN
    }
}

pub(super) fn cash_tone(cash: f64) -> &'static str {
    if cash < 5_000.0 {
        BOLD_RED
    } else if cash < 20_000.0 {
        BOLD_YELLOW
    } else {
        BOLD_GREEN
    }
}

pub(super) fn borrowing_room_tone(room: f64) -> &'static str {
    if room < 10_000.0 {
        BOLD_RED
    } else if room < 35_000.0 {
        BOLD_YELLOW
    } else {
        BOLD_GREEN
    }
}

pub(super) fn profit_tone(profit: f64) -> &'static str {
    if profit < 0.0 {
        BOLD_RED
    } else if profit.abs() < 1_000.0 {
        BOLD_YELLOW
    } else {
        BOLD_GREEN
    }
}

pub(super) fn customer_tone(customers: f64) -> &'static str {
    if customers < 0.0 {
        BOLD_RED
    } else if customers < 15.0 {
        BOLD_YELLOW
    } else {
        BOLD_GREEN
    }
}

pub(super) fn format_churn_rate(churn_rate: f64) -> String {
    format!("{:.2}%", churn_rate * 100.0)
}

pub(super) fn churn_tone(churn_rate: f64) -> &'static str {
    if churn_rate >= 0.025 {
        BOLD_RED
    } else if churn_rate >= 0.012 {
        BOLD_YELLOW
    } else {
        BOLD_GREEN
    }
}

pub(super) fn share_tone(share: f64) -> &'static str {
    if share >= 0.45 {
        BOLD_GREEN
    } else if share >= 0.35 {
        BOLD_YELLOW
    } else {
        BOLD_RED
    }
}

pub(super) fn reliability_tone(reliability: f64) -> &'static str {
    if reliability >= 0.80 {
        BOLD_GREEN
    } else if reliability >= 0.72 {
        BOLD_YELLOW
    } else {
        BOLD_RED
    }
}

pub(super) fn leverage_tone(leverage: f64) -> &'static str {
    if leverage > 0.85 {
        BOLD_RED
    } else if leverage > 0.70 {
        BOLD_YELLOW
    } else {
        BOLD_GREEN
    }
}

pub(super) fn headroom_tone(headroom: f64) -> &'static str {
    if headroom < 60.0 {
        BOLD_RED
    } else if headroom < 140.0 {
        BOLD_YELLOW
    } else {
        BOLD_GREEN
    }
}

pub(super) fn reserve_tone(reserve: f64) -> &'static str {
    if reserve < 0.0 {
        BOLD_RED
    } else if reserve < 15.0 {
        BOLD_YELLOW
    } else {
        BOLD_GREEN
    }
}

pub(super) fn rival_health_label(competitor: &Utility) -> (&'static str, &'static str) {
    let mut score = 0;
    if competitor.cash >= 20_000.0 {
        score += 1;
    } else if competitor.cash < 5_000.0 {
        score -= 1;
    }

    let leverage = competitor.debt_to_assets();
    if leverage <= 0.65 {
        score += 1;
    } else if leverage >= 0.85 {
        score -= 1;
    }

    if competitor.reliability >= 0.82 {
        score += 1;
    } else if competitor.reliability < 0.72 {
        score -= 1;
    }

    if competitor.reputation >= 60.0 {
        score += 1;
    } else if competitor.reputation < 48.0 {
        score -= 1;
    }

    if score >= 2 {
        ("healthy", BOLD_GREEN)
    } else if score <= -2 {
        ("vulnerable", BOLD_RED)
    } else {
        ("strained", BOLD_YELLOW)
    }
}

pub(super) fn utilization_tone(utilization: f64) -> &'static str {
    if utilization > 0.92 {
        BOLD_RED
    } else if utilization > 0.80 {
        BOLD_YELLOW
    } else {
        BOLD_GREEN
    }
}

pub(super) fn reputation_tone(reputation: f64) -> &'static str {
    if reputation < 48.0 {
        BOLD_RED
    } else if reputation < 60.0 {
        BOLD_YELLOW
    } else {
        BOLD_GREEN
    }
}

pub(super) fn print_box(title: &str, lines: &[String]) {
    for line in render_box(title, lines, dashboard_width()) {
        println!("{line}");
    }
}

pub(super) fn print_box_pair(
    left_title: &str,
    left_lines: &[String],
    right_title: &str,
    right_lines: &[String],
) {
    let content_height = left_lines.len().max(1).max(right_lines.len().max(1));
    let (left_width, right_width) = paired_column_widths(dashboard_width());
    let left = render_box(
        left_title,
        &padded_box_lines(left_lines, content_height),
        left_width,
    );
    let right = render_box(
        right_title,
        &padded_box_lines(right_lines, content_height),
        right_width,
    );

    for index in 0..left.len() {
        let left_line = left
            .get(index)
            .cloned()
            .unwrap_or_else(|| " ".repeat(left_width));
        let right_line = right
            .get(index)
            .cloned()
            .unwrap_or_else(|| " ".repeat(right_width));
        println!("{left_line}{}{right_line}", " ".repeat(COLUMN_GAP));
    }
}

pub(super) fn dashboard_width() -> usize {
    let columns = std::env::var("COLUMNS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok());
    dashboard_width_for_columns(columns)
}

pub(super) fn dashboard_width_for_columns(columns: Option<usize>) -> usize {
    columns
        .unwrap_or(DEFAULT_SCREEN_WIDTH)
        .clamp(MIN_SCREEN_WIDTH, MAX_SCREEN_WIDTH)
}

pub(super) fn content_width() -> usize {
    dashboard_width().saturating_sub(4)
}

pub(super) fn paired_column_widths(total_width: usize) -> (usize, usize) {
    let available = total_width
        .saturating_sub(COLUMN_GAP)
        .max(MIN_SCREEN_WIDTH - COLUMN_GAP);
    let left = available / 2;
    let right = available - left;
    (left, right)
}

pub(super) fn padded_box_lines(lines: &[String], height: usize) -> Vec<String> {
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

pub(super) fn clear_screen() {
    if terminal_control_enabled() {
        print!("\x1B[2J\x1B[H");
    }
}

pub(super) fn render_box(title: &str, lines: &[String], width: usize) -> Vec<String> {
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

pub(super) fn render_top_border(title: &str, width: usize) -> String {
    let dim = ansi(DIM);
    let bold_cyan = ansi(BOLD_CYAN);
    let reset = ansi(RESET);
    if title.is_empty() {
        format!("{dim}┌{}┐{reset}", "─".repeat(width - 2))
    } else {
        let title = shorten_plain(title, width.saturating_sub(6));
        let fill = (width - 2).saturating_sub(visible_width(&title) + 3);
        format!(
            "{dim}┌─ {bold_cyan}{title}{reset}{dim} {}┐{reset}",
            "─".repeat(fill)
        )
    }
}

pub(super) fn render_bottom_border(width: usize) -> String {
    format!("{}└{}┘{}", ansi(DIM), "─".repeat(width - 2), ansi(RESET))
}

pub(super) fn render_box_line(line: &str, width: usize) -> String {
    let content_width = width.saturating_sub(4);
    let fitted = fit_line(line, content_width);
    let padding = content_width.saturating_sub(visible_width(&fitted));
    let dim = ansi(DIM);
    let reset = ansi(RESET);
    format!(
        "{dim}│{reset} {}{} {dim}│{reset}",
        fitted,
        " ".repeat(padding)
    )
}

pub(super) fn fit_line(line: &str, width: usize) -> String {
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
                for sequence_char in chars.by_ref() {
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

    truncated.push_str(ansi(RESET));
    truncated.push_str("...");
    truncated
}

pub(super) fn wrap_plain_text(text: &str, width: usize) -> Vec<String> {
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

pub(super) fn visible_width(line: &str) -> usize {
    let mut width = 0;
    let mut chars = line.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\x1B' {
            if matches!(chars.peek(), Some(&'[')) {
                chars.next();
                for sequence_char in chars.by_ref() {
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
