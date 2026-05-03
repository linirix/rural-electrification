# Playtest Memo: 1,000,008-Game Strategy Stress Scan

Run date: 2026-05-02

Command:

```sh
cargo run --release --bin stress_scan -- --seeds 111112
```

This ran 111,112 seeds across all 9 strategy bots, for 1,000,008 total simulated games at default variance.

## Top-Line Results

| Strategy | Win % | Avg Finish Share | Finish Range | Main Defeats | Avg Stress | Max Stress |
|---|---:|---:|---:|---|---:|---:|
| naive | 1.4% | 30.2% | 6.3-50.5% | board 105,527; access 4,076 | 0 | 0 |
| organic | 61.5% | 48.0% | 34.4-69.3% | board 42,812 | 0 | 0 |
| balanced | 67.6% | 51.7% | 36.4-73.6% | board 35,971 | 8 | 40 |
| regional | 54.8% | 58.7% | 37.3-97.5% | board 50,210 | 17 | 43 |
| ma | 58.3% | 46.0% | 28.3-74.5% | board 46,385 | 36 | 103 |
| raider | 34.6% | 72.0% | 28.8-97.4% | board 68,891; access 3,723 | 42 | 156 |
| landshark | 65.2% | 50.9% | 35.2-69.7% | board 38,690 | 15 | 103 |
| costanza | 10.1% | 41.4% | 12.1-65.5% | board 99,889 | 12 | 51 |
| glonzo | 16.4% | 41.0% | 3.0-97.2% | board 87,097; receivership 723; access 5,069 | 31 | 163 |

Defeat-mode coverage across 1,000,008 games:

| Defeat Mode | Count |
|---|---:|
| Board review / mandate | 524,748 |
| Receivership | 723 |
| Market access loss | 12,868 |
| Hostile takeover | 519 |
| Acquisition-stress receivership | 2 |

## What Looks Healthy

The strategy curve is doing what we wanted. Balanced is the benchmark winner, landshark is close but not dominant, organic is viable and relatively safe, ma is meaningfully risky, raider has the biggest upside but loses often, and glonzo/costanza continue to exercise ugly edge cases.

The game also survived 1,000,008 simulations with no panics or state explosions. That is a strong result for the current simulation architecture.

Organic growth is not automatic. It wins 61.5%, averages 48.0% finish share, and can fall as low as 34.4%. That is a good shape: low operational risk, but still close-run.

Balanced strategy is still doing the intended job. It wins 67.6%, beats organic by about 6 points, and keeps acquisition stress low enough that it reads as opportunistic M&A rather than roll-up gambling.

Raider is now a true high-variance strategy. It averages 72.0% finish share and can reach 97.4%, but wins only 34.6%. That means raw share accumulation is not enough if reliability, service quality, and integration collapse.

Regional play is meaningfully contested. It wins 54.8% at the Y10 mandate and averages 58.7% share against a 58% target. That is a knife-edge mandate, but it is reachable.

## Critical Findings

### 1. Board defeats are doing almost all the work

More than half of all games end through board review or mandate failure. That is mostly right for deliberate strategies, but it creates a playability risk: a player can feel like they were winning operationally, then lose because a review gate was not salient enough.

The clearest example is raider. Many raider losses finish with 95-97% market share, high cash, and strong profits, but only 65-72% reliability. Those are valid losses, but the interface needs to make the reason impossible to miss: the board is not judging monopoly share alone; it is judging whether Metro can operate the network it bought.

Recommended improvement:

- Make the dashboard and board screen call out current failing review gates more sharply.
- Add a "share is not enough" warning when market share is very high but reliability or sustainability is below the active review target.
- On outcome screens, lead with the failed gate before the headline share number.

### 2. Acquisition-stress receivership is effectively dead

Only 2 acquisition-stress receiverships occurred in 1,000,008 games, despite stress reaching 103 in ma, 156 in raider, and 163 in glonzo.

This does not mean acquisition stress is useless. It is clearly affecting outcomes through board failure, market access, and operating drag. But the specific terminal path named "acquisition stress receivership" is not doing much.

Recommended improvement:

- Either accept acquisition stress as a nonterminal pressure system and stop presenting stress receivership as a live threat, or
- Convert high acquisition stress into more visible nonterminal consequences: lender restrictions, forced concessions, delayed projects, dividend blocks, or emergency integration costs.

I would not make this a broad cash-insolvency punishment. The current user-facing philosophy that a player can skate close to the edge if they stay solvent still feels right.

### 3. Hostile takeover is present but extremely rare

Hostile takeover fired 519 times in 1,000,008 games. That is not zero, but it is rare enough that many players will never see it.

Recommended improvement:

- Keep hostile takeover as a tail event, but surface warning language when the player is in the danger zone.
- If we want this to become a meaningful strategic pressure, tune it through warning events first rather than immediately raising the failure rate.

### 4. Market access loss is working as the early-collapse path

Market access loss occurred 12,868 times. The earliest collapses were glonzo failures at Q3 with reliability in the 35-43% range. That is healthy: catastrophic operating neglect can kill the company early, but normal deliberate play is not randomly wiped out.

Recommended improvement:

- Add a prominent dashboard signal when reliability falls below 50%, with language that market access is at risk.
- Consider a report-screen warning one quarter before access loss if reliability is in the danger band.

### 5. Regional mandate is balanced but may feel arbitrary without stronger explanation

Close Y10 regional victories often landed around exactly 58.5% share. That is good mechanically, but it can feel like the game is grading a hidden spreadsheet if the target is not persistent and clear.

Recommended improvement:

- Keep the Y10 mandate target where it is for now.
- Make the active mandate line more explicit in post-review play: `Y10 mandate: share 58%, territories 2, reliability 82%, leverage <=90%`.
- On regional defeat, list each failed gate with current vs target.

### 6. Release-seed candidates are now identifiable

Potentially useful seeds:

| Seed | Game Seed | Why It Is Interesting |
|---:|---:|---|
| 7 | 7,021 | Hard-but-fair: organic loses, balanced/ma/landshark win close. |
| 9 | 9,027 | High-risk payoff: organic and balanced lose, ma wins. |
| 8 | 8,024 | Very hard: all deliberate Y5 strategies lose. |
| 106 | 106,318 | Favorable opener: naive can win; good tutorial/easy seed candidate. |
| 131 | 131,393 | Favorable but not automatic: naive wins, landshark loses. |
| 59109 | 59,286,327 | Close naive victory at exactly 45.5% share; useful for testing review messaging. |
| 9150 | 9,177,450 | Close regional Y10 victory at 58.5% share. |
| 51615 | 51,769,845 | Regional victory with thin profit and near-target share. |

## Recommended Next Work

1. Improve review-gate communication. This is the highest playability payoff. The sim is correctly rejecting high-share, low-service empires, but players need earlier and clearer warnings.

2. Add danger-zone warnings for terminal operating failures. Reliability below 50% should feel materially dangerous before market access is lost.

3. Reframe acquisition stress. Treat it as a visible pressure system that creates costs and restrictions rather than a nominal receivership path that almost never fires.

4. Preserve the current strategy balance. The million-game curve is strong enough that I would avoid broad balance changes until the UI explains the current mechanics better.

5. Keep a curated seed list. The release build could eventually expose a few named starts: easy opener, hard-but-fair, hostile market, regional stretch, and raider cautionary tale.

## Bottom Line

The simulation is stable and the strategy curve is healthy. The largest remaining issue is not mathematical balance; it is player comprehension. The board and mandate gates are doing important design work, but the interface should make those gates feel like strategic objectives rather than surprise grading criteria.
