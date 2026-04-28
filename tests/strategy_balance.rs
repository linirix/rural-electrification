use electrification::{InitialVariance, Strategy, StrategySummary, run_strategy_batch};

const REGRESSION_SEEDS: u32 = 250;

#[test]
fn strategy_profiles_stay_in_design_bands() {
    let variance = InitialVariance::default();
    let cases = [
        (Strategy::Naive, 0.00, 0.05),
        (Strategy::Organic, 0.54, 0.70),
        (Strategy::Balanced, 0.60, 0.78),
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

    let organic_summary = summary_for(&summaries, Strategy::Organic);
    let balanced_summary = summary_for(&summaries, Strategy::Balanced);
    let mna_summary = summary_for(&summaries, Strategy::Mna);
    let organic = organic_summary.win_rate();
    let balanced = balanced_summary.win_rate();
    let mna = mna_summary.win_rate();

    assert!(
        balanced >= organic + 0.015,
        "balanced play should modestly outperform pure organic growth: balanced {:.1}%, organic {:.1}%",
        balanced * 100.0,
        organic * 100.0
    );
    assert!(
        balanced >= mna + 0.04,
        "balanced play should outperform reckless M&A while still carrying deal risk: balanced {:.1}%, mna {:.1}%",
        balanced * 100.0,
        mna * 100.0
    );
    assert!(
        (0.455..=0.505).contains(&organic_summary.average_finish_share()),
        "organic wins should stay close-run rather than automatic: avg finish share {:.1}%",
        organic_summary.average_finish_share() * 100.0
    );
    let organic_spread =
        organic_summary.finish_share_range.max - organic_summary.finish_share_range.min;
    let mna_spread = mna_summary.finish_share_range.max - mna_summary.finish_share_range.min;
    assert!(
        organic_summary.finish_share_range.min >= mna_summary.finish_share_range.min + 0.020,
        "organic should have the safer downside tail: organic min {:.1}%, mna min {:.1}%",
        organic_summary.finish_share_range.min * 100.0,
        mna_summary.finish_share_range.min * 100.0
    );
    assert!(
        mna_summary.finish_share_range.max >= organic_summary.finish_share_range.max + 0.010,
        "M&A should retain the higher upside tail: mna max {:.1}%, organic max {:.1}%",
        mna_summary.finish_share_range.max * 100.0,
        organic_summary.finish_share_range.max * 100.0
    );
    assert!(
        mna_spread >= organic_spread + 0.035,
        "M&A should be materially more variable: mna spread {:.1} pts, organic spread {:.1} pts",
        mna_spread * 100.0,
        organic_spread * 100.0
    );
    assert!(
        mna_summary.average_peak_acquisition_stress()
            >= organic_summary.average_peak_acquisition_stress() + 0.20,
        "M&A should carry visibly higher deal stress: mna avg peak {:.0} pts, organic avg peak {:.0} pts",
        mna_summary.average_peak_acquisition_stress() * 100.0,
        organic_summary.average_peak_acquisition_stress() * 100.0
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

#[test]
fn test_player_personalities_exercise_distinct_edges() {
    let landshark = run_strategy_batch(250, InitialVariance::default(), Strategy::Landshark);
    let costanza = run_strategy_batch(250, InitialVariance::default(), Strategy::Costanza);

    assert_eq!(landshark.victories + landshark.defeats(), 250);
    assert_eq!(costanza.victories + costanza.defeats(), 250);
    assert!(
        (0.58..=0.78).contains(&landshark.win_rate()),
        "landshark should be a strong visible-mechanics optimizer, got {:.1}%",
        landshark.win_rate() * 100.0
    );
    assert!(
        landshark.finish_share_range.max >= 0.60,
        "landshark should find high-upside visible lines, max finish share {:.1}%",
        landshark.finish_share_range.max * 100.0
    );
    assert!(
        (0.04..=0.16).contains(&costanza.win_rate()),
        "costanza should beat naive instincts without becoming a tuned optimizer, got {:.1}%",
        costanza.win_rate() * 100.0
    );
    assert!(
        costanza.average_finish_share() > 0.38,
        "costanza should materially improve on naive finish share, got {:.1}%",
        costanza.average_finish_share() * 100.0
    );
}

#[test]
fn glonzo_uses_policy_seed_even_from_fixed_starts() {
    let summary = run_strategy_batch(120, InitialVariance::fixed(), Strategy::Glonzo);
    assert!(
        summary.finish_share_range.max - summary.finish_share_range.min > 0.08,
        "glonzo should vary policy choices even when all starts are identical, got {:.1}-{:.1}%",
        summary.finish_share_range.min * 100.0,
        summary.finish_share_range.max * 100.0
    );
}

#[test]
fn raider_strategy_exercises_acquisition_stress() {
    let summary = run_strategy_batch(120, InitialVariance::default(), Strategy::Raider);
    assert_eq!(summary.runs, 120);
    assert!(
        summary.average_peak_acquisition_stress() >= 0.35,
        "raider should exercise stressed roll-up mechanics: avg peak {:.0} pts",
        summary.average_peak_acquisition_stress() * 100.0
    );
    assert!(
        summary.peak_acquisition_stress_range.max >= 0.80,
        "raider should reach distressed underwriting in at least some seeds: max {:.0} pts",
        summary.peak_acquisition_stress_range.max * 100.0
    );
    assert!(
        summary.acquisition_stress_defeats >= 3,
        "raider should produce defeats involving acquisition stress, got {}",
        summary.acquisition_stress_defeats
    );
}

#[test]
fn regional_strategy_can_meet_year_ten_mandate() {
    let summary = run_strategy_batch(
        REGRESSION_SEEDS,
        InitialVariance::default(),
        Strategy::Regional,
    );
    assert_eq!(summary.runs, REGRESSION_SEEDS);
    assert_eq!(summary.victories + summary.defeats(), REGRESSION_SEEDS);
    assert!(
        summary.win_rate() >= 0.30,
        "regional strategy should prove the Y10 mandate is reachable in at least 30% of seeds: {:.1}%",
        summary.win_rate() * 100.0
    );
    assert!(
        summary.win_rate() <= 0.65,
        "regional mandate should remain contested, not automatic: {:.1}%",
        summary.win_rate() * 100.0
    );
    assert!(
        summary.average_finish_share() >= 0.55,
        "regional strategy should usually finish near the regional share target: avg {:.1}%",
        summary.average_finish_share() * 100.0
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
