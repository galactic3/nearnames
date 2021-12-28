use crate::*;

pub fn calculate_lot_bid_rewards(
    prev_amount: Option<Balance>,
    amount: Balance,
    seller_rewards_commission: Fraction,
    prev_bidder_commission_share: Fraction,
) -> (Option<Balance>, Balance) {
    match prev_amount {
        Some(prev_amount) => {
            let to_prev_bidder_bid = prev_amount;
            let to_seller = amount - to_prev_bidder_bid;
            let commission = seller_rewards_commission * to_seller;
            let to_seller = to_seller - commission;
            let to_prev_bidder_reward = prev_bidder_commission_share * commission;
            let to_prev_bidder = to_prev_bidder_bid + to_prev_bidder_reward;

            (Some(to_prev_bidder), to_seller)
        }
        None => {
            let to_seller = amount;
            let commission = seller_rewards_commission * to_seller;
            let to_seller = to_seller - commission;
            (None, to_seller)
        }
    }
}
