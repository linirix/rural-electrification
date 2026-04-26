use electrification::{Decision, Game, sim::money};

fn snapshot(game: &Game, label: &str) {
    println!(
        "{:<20} cash={:>9}  shares={:>7.0}  price=${:>5.2}  market_cap={:>9}  fatigue={:.3}",
        label,
        money(game.player.cash),
        game.player.shares,
        game.player.stock_price,
        money(game.player.market_cap()),
        game.equity_market_fatigue
    );
}

fn main() {
    println!("=== ROUND TRIP CYCLE: $20k issue → $20k buyback (no quarter advance) ===");
    let mut game = Game::with_seed(1);
    snapshot(&game, "start");
    let _ = game.apply_decision(Decision::IssueStock { amount: 20_000.0 });
    snapshot(&game, "after issue $20k");
    let _ = game.apply_decision(Decision::BuyBackStock { amount: 20_000.0 });
    snapshot(&game, "after buyback $20k");

    println!("\n=== TEN BACK-TO-BACK ROUND TRIPS (no quarter advance) ===");
    let mut game = Game::with_seed(1);
    snapshot(&game, "start");
    for i in 1..=10 {
        let r1 = game.apply_decision(Decision::IssueStock { amount: 10_000.0 });
        let r2 = game.apply_decision(Decision::BuyBackStock { amount: 10_000.0 });
        if r1.is_err() || r2.is_err() {
            println!("cycle {} aborted: issue={:?} buyback={:?}", i, r1, r2);
            break;
        }
        snapshot(&game, &format!("after rt {}", i));
    }

    println!("\n=== ONE PER QUARTER (advance between each) ===");
    let mut game = Game::with_seed(1);
    snapshot(&game, "start");
    for i in 1..=8 {
        let _ = game.apply_decision(Decision::IssueStock { amount: 10_000.0 });
        let _ = game.apply_decision(Decision::BuyBackStock { amount: 10_000.0 });
        let _ = game.advance_quarter();
        snapshot(&game, &format!("end of Q{}", i));
    }

    println!("\n=== BUYBACK ONLY (no issue first) - is the price pump usable? ===");
    let mut game = Game::with_seed(1);
    game.player.cash = 200_000.0;
    snapshot(&game, "start");
    let _ = game.apply_decision(Decision::BuyBackStock { amount: 10_000.0 });
    snapshot(&game, "after $10k buyback");
    let _ = game.apply_decision(Decision::BuyBackStock { amount: 10_000.0 });
    snapshot(&game, "after 2nd $10k buyback");
    let _ = game.apply_decision(Decision::BuyBackStock { amount: 10_000.0 });
    snapshot(&game, "after 3rd $10k buyback");
}
