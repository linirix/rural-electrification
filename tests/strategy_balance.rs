use electrification::{InitialVariance, Strategy, StrategySummary, run_strategy_batch};

const REGRESSION_SEEDS: u32 = 250;

#[test]
fn strategy_win_rates_stay_in_design_bands() {
    let variance = InitialVariance::default();
    let cases = [
        (Strategy::Naive, 0.00, 0.07),
        (Strategy::Organic, 0.66, 0.82),
        (Strategy::Balanced, 0.72, 0.88),
        (Strategy::Mna, 0.45, 0.68),
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
        balanced >= organic + 0.03,
        "balanced play should modestly outperform pure organic growth: balanced {:.1}%, organic {:.1}%",
        balanced * 100.0,
        organic * 100.0
    );
    assert!(
        organic >= mna + 0.08,
        "organic should remain the more reliable low-risk lane: organic {:.1}%, mna {:.1}%",
        organic * 100.0,
        mna * 100.0
    );
    assert!(
        balanced >= mna + 0.08,
        "balanced play should outperform reckless M&A while still carrying deal risk: balanced {:.1}%, mna {:.1}%",
        balanced * 100.0,
        mna * 100.0
    );
    let organic_summary = summary_for(&summaries, Strategy::Organic);
    assert!(
        (0.455..=0.505).contains(&organic_summary.average_finish_share()),
        "organic wins should stay close-run rather than automatic: avg finish share {:.1}%",
        organic_summary.average_finish_share() * 100.0
    );
    let mna_summary = summary_for(&summaries, Strategy::Mna);
    assert!(
        mna_summary.finish_share_range.min < organic_summary.finish_share_range.min,
        "M&A should retain a wider downside tail: mna min {:.1}%, organic min {:.1}%",
        mna_summary.finish_share_range.min * 100.0,
        organic_summary.finish_share_range.min * 100.0
    );
}

#[test]
fn glonzo_random_strategy_runs_to_completion_with_varied_outcomes() {
    let summary = run_strategy_batch(120, InitialVariance::default(), Strategy::Glonzo);
    assert_eq!(summary.runs, 120);
    assert_eq!(summary.victories + summary.defeats(), 120);
    assert!(summary.average_finish_share().is_finite());
    assert!(
        summary.finish_share_range.max - summary.finish_share_range.min > 0.08,
        "glonzo should create a broad random testing spread, got {:.1}-{:.1}%",
        summary.finish_share_range.min * 100.0,
        summary.finish_share_range.max * 100.0
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
