# Playtest Seed Notes

Seed notes use two values:

- `playtest seed` is the 1-based seed window used by `stress_scan` and `playtest`.
- `game seed` is the direct simulation seed currently produced by the harness (`playtest seed * 1003`).

Use `game seed` if adding a curated release seed picker that calls `Game::with_seed`.

## 2026-04-27 Stress Pass

Coverage:

- `100,000` default-variance seeds across all strategies (`900,000` playthroughs)
- `20,000` seeds across six starting-variance profiles and all strategies (`1,080,000` playthroughs)
- `50,000` max-variance seeds across all strategies (`450,000` playthroughs)

## 2026-04-27 Final Matrix Addendum

Additional coverage:

- `50,000` seeds across six starting-variance profiles and all strategies (`2,700,000` playthroughs)
- `100,000` default-variance stress seeds across all strategies (`900,000` playthroughs)
- `100,000` max-variance stress seeds across all strategies (`900,000` playthroughs)

### High-Share Failure Seeds

These are useful for testing whether the end screen explains that market share alone is not enough.

| playtest seed | game seed | strategy | notes |
|---:|---:|---|---|
| 83386 | 83636158 | raider | 97.4% share, 69% reliability, board defeat |
| 57883 | 58056649 | raider | 97.1% share, 53% reliability, board defeat |
| 66264 | 66462792 | glonzo | 97.1% share, 90% reliability, negative profit, board defeat |
| 38777 | 38893331 | glonzo | 97.3% share at max variance, negative profit, board defeat |
| 15704 | 15751112 | glonzo | 94.8% share at max variance, negative cash, receivership |

### Early Collapse Seeds

These are good regression seeds for reckless-play failure messaging and market-access loss.

| playtest seed | game seed | strategy | notes |
|---:|---:|---|---|
| 49310 | 49457930 | mna | Max variance Q2 market-access loss after reliability falls to 41% |
| 99196 | 99493588 | glonzo | Max variance Q2 market-access loss |
| 35291 | 35396873 | glonzo | Default variance Q3 market-access loss |
| 41919 | 42044757 | glonzo | Default variance Q3 market-access loss with 52.7% share |

### Close Max-Variance Wins

These are useful for testing tense but fair endings under the harshest start variance.

| playtest seed | game seed | strategy | notes |
|---:|---:|---|---|
| 73428 | 73648284 | naive | 45.5% share, 84% reliability, clean low-leverage win |
| 44173 | 44305519 | organic | 45.5% share, $3.0k profit |
| 96484 | 96773452 | balanced | 45.5% share, 83% reliability |
| 54929 | 55093787 | mna | 45.5% share after 62.6% peak |
| 6039 | 6057117 | regional | 58.5% Y10 share, 88% reliability |

### Balanced Challenge Seeds

Organic loses, while balanced and landshark win. These look useful for demonstrating why the balanced strategy exists.

| playtest seed | game seed | notes |
|---:|---:|---|
| 16 | 16048 | Organic 47.0%, balanced 56.8%, landshark 56.6% |
| 33 | 33099 | Organic 47.5%, balanced 57.0%, M&A loses |
| 67 | 67201 | Organic 44.9%, balanced 61.3%, M&A wins narrowly |
| 77 | 77231 | Organic 47.1%, balanced 58.9%, costanza struggles |
| 85 | 85255 | Very close balanced win at 45.8% |
| 96 | 96288 | Organic 43.4%, balanced 59.0%, landshark 59.5% |
| 101 | 101303 | All strong strategies can win; organic cannot |
| 154 | 154462 | Balanced and landshark win, M&A loses |

### High-Risk Payoff Seeds

Organic loses, but M&A wins. These are useful for testing whether risky dealmaking has a legitimate role.

| playtest seed | game seed | notes |
|---:|---:|---|
| 8 | 8024 | Organic and balanced lose; M&A wins at 52.6% |
| 29 | 29087 | M&A wins at 54.2%; landshark misses |
| 36 | 36108 | Organic/balanced/landshark all lose; M&A wins |
| 58 | 58174 | Balanced wins narrowly; M&A wins bigger |
| 88 | 88264 | Organic and balanced lose; M&A wins at 53.7% |
| 113 | 113339 | Balanced barely wins, M&A wins better |
| 146 | 146438 | Balanced and M&A win; landshark misses |
| 148 | 148444 | M&A and landshark find different winning paths |

### Very Hard Seeds

All deliberate Year 5 strategies lose in the default-variance scan. Use sparingly; these may feel unfair until manually reviewed.

| playtest seed | game seed | notes |
|---:|---:|---|
| 7 | 7021 | All deliberate strategies below 45% |
| 11 | 11033 | Several strategies reach 50%+ share but miss sustainability |
| 25 | 25075 | Broadly poor outcome across strategies |
| 37 | 37111 | Balanced gets closer than others but still fails |
| 50 | 50150 | All deliberate strategies cluster around low 40s |
| 64 | 64192 | All deliberate strategies fail, no terminal collapse |
| 72 | 72216 | Similar poor outcome across all deliberate paths |
| 87 | 87261 | Consistently difficult without extreme volatility |

### Favorable Seeds

Naive wins. Useful as forgiving tutorial/demo starts.

| playtest seed | game seed | notes |
|---:|---:|---|
| 2 | 2006 | Almost every strategy wins |
| 48 | 48144 | Naive wins; M&A loses |
| 199 | 199597 | Naive wins; M&A loses |
| 292 | 292876 | Broadly forgiving |
| 319 | 319957 | Broadly forgiving, costanza high finish |
| 360 | 361080 | Naive wins narrowly |
| 456 | 457368 | Broadly favorable |
| 474 | 475422 | Broadly favorable |
| 498 | 499494 | Naive wins; M&A loses |

### Close Year 5 Wins

These finish just over the review threshold and should feel tense.

| playtest seed | game seed | strategy | notes |
|---:|---:|---|---|
| 39878 | 39997634 | organic | 45.5% share, $2.8k profit |
| 51767 | 51922301 | organic | 45.5% share, $1.9k profit |
| 17518 | 17570554 | balanced | 45.5% share, $2.6k profit |
| 3997 | 4008991 | landshark | 45.5% share, $1.4k profit |
| 52936 | 53094808 | mna | 45.5% share after 62.3% peak |
| 78765 | 79001295 | raider | 45.5% share after 70.7% peak |

### Close Year 10 Regional Wins

These hit the regional mandate very close to the share threshold.

| playtest seed | game seed | notes |
|---:|---:|---|
| 70098 | 70308294 | 58.5% share, $7.5k profit |
| 95993 | 96280979 | 58.5% share, strong cash cushion |
| 57813 | 57986439 | 58.5% share, modest cash |
| 42883 | 43011649 | 58.5% share, low peak share |
| 35386 | 35492158 | 58.5% share, high leverage but passes |
| 7592 | 7614776 | 58.5% share, $368 profit |
| 48776 | 48922328 | 58.5% share, reached $0 minimum cash |
