use crate::{AcquisitionTerms, Decision, Game};

pub(super) fn finance_to_cash(game: &mut Game, target_cash: f64, max_debt_to_assets: f64) {
    if game.player.cash >= target_cash {
        return;
    }

    let need = target_cash - game.player.cash;
    borrow_up_to(game, need, max_debt_to_assets);

    if game.player.cash < target_cash {
        let amount = (target_cash - game.player.cash) * 1.15;
        if amount >= 1_000.0 {
            let _ = game.apply_decision(Decision::IssueStock { amount });
        }
    }
}

pub(super) fn borrow_up_to(game: &mut Game, target_amount: f64, max_debt_to_assets: f64) {
    let leverage_room = (game.player.asset_base * max_debt_to_assets - game.player.debt).max(0.0);
    let amount = target_amount.min(game.borrowing_room()).min(leverage_room);
    if amount >= 1_000.0 {
        let _ = game.apply_decision(Decision::Borrow { amount });
    }
}

pub(super) fn cheapest_competitor(game: &Game) -> Option<(usize, f64)> {
    (0..game.competitors.len())
        .filter_map(|index| {
            estimated_acquisition_terms(game, index).map(|terms| (index, terms.price))
        })
        .min_by(|left, right| left.1.total_cmp(&right.1))
}

pub(super) fn estimated_acquisition_terms(game: &Game, index: usize) -> Option<AcquisitionTerms> {
    if game.has_diligence(index) {
        game.acquisition_terms(index)
    } else {
        game.public_acquisition_estimate(index)
    }
}
