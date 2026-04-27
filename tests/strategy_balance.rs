use electrification::{InitialVariance, Strategy, StrategySummary, run_strategy_batch};

const REGRESSION_SEEDS: u32 = 120;

#[test]
fn strategy_win_rates_stay_in_design_bands() {
    let variance = InitialVariance::default();
    let cases = [
        (Strategy::Naive, 0.00, 0.06),
        (Strategy::Organic, 0.40, 0.65),
        (Strategy::Balanced, 0.62, 0.86),
        (Strategy::Mna, 0.70, 0.90),
    ];

    let summaries = cases
        .iter()
        .map(|&(strategy, min_win, max_win)| {
            let summary = run_strategy_batch(REGRESSION_SEEDS, variance, strategy);
            assert_win_rate(strategy, &summary, min_win, max_win);
            (strategy, summary)
        })
        .collect::<Vec<_>>();

    let organic = summary_for(&summaries, Strategy::Organic).win_rate();
    let balanced = summary_for(&summaries, Strategy::Balanced).win_rate();
    let mna = summary_for(&summaries, Strategy::Mna).win_rate();

    assert!(
        balanced >= organic,
        "balanced strategy should not underperform organic in the fixed regression window: balanced {:.1}%, organic {:.1}%",
        balanced * 100.0,
        organic * 100.0
    );
    assert!(
        balanced - organic >= 0.10,
        "balanced strategy should retain a meaningful edge over organic: balanced {:.1}%, organic {:.1}%",
        balanced * 100.0,
        organic * 100.0
    );
    assert!(
        mna - balanced <= 0.18,
        "M&A should not reopen the old dominant-strategy gap: mna {:.1}%, balanced {:.1}%",
        mna * 100.0,
        balanced * 100.0
    );
}

fn assert_win_rate(strategy: Strategy, summary: &StrategySummary, min_win: f64, max_win: f64) {
    let win_rate = summary.win_rate();
    assert!(
        (min_win..=max_win).contains(&win_rate),
        "{} win rate {:.1}% outside expected {:.0}-{:.0}% band over {} seeds; avg finish share {:.1}%",
        strategy.name(),
        win_rate * 100.0,
        min_win * 100.0,
        max_win * 100.0,
        summary.runs,
        summary.average_finish_share() * 100.0
    );
}

fn summary_for(summaries: &[(Strategy, StrategySummary)], target: Strategy) -> &StrategySummary {
    summaries
        .iter()
        .find_map(|(strategy, summary)| (*strategy == target).then_some(summary))
        .expect("missing strategy summary")
}
