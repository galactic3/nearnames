use crate::*;

pub const ERR_LOT_SELLS_SELF: &str = "expected lot_id != seller_id";
pub const ERR_LOT_PRICE_RESERVE_LE_BUY_NOW: &str = "expected reserve_price <= buy_now_price";
pub const ERR_LOT_BID_WRONG_STATUS: &str = "bid: expected status active";
pub const ERR_LOT_BID_BID_TOO_SMALL: &str = "bid: expected bigger bid";
pub const ERR_LOT_BID_WRONG_BIDDER: &str = "bid: seller and lot cannot bid";
pub const ERR_LOT_CLAIM_BY_BIDDER_WRONG_STATUS: &str =
    "claim by bidder: expected status sale success";
pub const ERR_LOT_CLAIM_BY_BIDDER_WRONG_CLAIMER: &str = "claim by bidder: wrong claimer";
pub const ERR_LOT_CLAIM_BY_SELLER_WRONG_STATUS: &str = "claim by seller: expected status withdrawn";
pub const ERR_LOT_CLAIM_BY_SELLER_WRONG_CLAIMER: &str = "claim by seller: wrong claimer";
pub const ERR_LOT_WITHDRAW_HAS_BID: &str = "withdraw: expected no bids";
pub const ERR_LOT_WITHDRAW_WRONG_STATUS: &str = "withdraw: already withdrawn";
pub const ERR_LOT_WITHDRAW_WRONG_WITHDRAWER: &str = "withdraw: wrong withdrawer";

#[derive(Debug, PartialEq, Eq)]
pub enum LotStatus {
    OnSale,
    Withdrawn,
    SaleSuccess,
    SaleFailure,
}

impl fmt::Display for LotStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Bid {
    pub bidder_id: ProfileId,
    pub amount: Balance,
    pub timestamp: Timestamp,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Lot {
    pub lot_id: LotId,
    pub seller_id: ProfileId,
    pub reserve_price: Balance,
    pub buy_now_price: Balance,
    pub start_timestamp: Timestamp,
    pub finish_timestamp: Timestamp,
    pub is_withdrawn: bool,

    bids: Vector<Bid>,
}

impl Lot {
    pub fn new(
        lot_id: LotId,
        seller_id: ProfileId,
        reserve_price: Balance,
        buy_now_price: Balance,
        time_now: Timestamp,
        duration: Duration,
    ) -> Lot {
        assert_ne!(lot_id, seller_id, "{}", ERR_LOT_SELLS_SELF);
        assert!(
            reserve_price <= buy_now_price,
            "{}",
            ERR_LOT_PRICE_RESERVE_LE_BUY_NOW,
        );

        // TODO: do we still need to hash the key
        let mut prefix: Vec<u8> = Vec::with_capacity(33);
        prefix.extend(PREFIX_LOTS_BIDS.as_bytes());
        prefix.extend(env::sha256(lot_id.as_bytes()));

        Lot {
            lot_id,
            seller_id,
            reserve_price,
            buy_now_price,
            start_timestamp: time_now,
            finish_timestamp: time_now + duration,
            is_withdrawn: false,
            bids: Vector::new(prefix),
        }
    }

    pub fn bids(&self) -> Vec<Bid> {
        self.bids.to_vec()
    }

    pub fn is_active(&self, time_now: Timestamp) -> bool {
        if time_now >= self.finish_timestamp {
            return false;
        }
        if let Some(last_bid_amount) = self.last_bid_amount() {
            if last_bid_amount >= self.buy_now_price {
                return false;
            }
        }
        if self.is_withdrawn {
            return false;
        }

        true
    }

    pub fn last_bid(&self) -> Option<Bid> {
        if self.bids.is_empty() {
            None
        } else {
            Some(self.bids.get(self.bids.len() - 1).unwrap())
        }
    }

    pub fn last_bid_amount(&self) -> Option<Balance> {
        self.last_bid().map(|x| x.amount)
    }

    pub fn next_bid_amount(&self, time_now: Timestamp, bid_step: Fraction) -> Option<Balance> {
        if !self.is_active(time_now) {
            return None;
        }
        if let Some(last_bid_amount) = self.last_bid_amount() {
            let mut next_bid_amount = last_bid_amount + bid_step * last_bid_amount;
            if next_bid_amount == last_bid_amount {
                next_bid_amount += 1;
            }
            Some(std::cmp::min(next_bid_amount, self.buy_now_price))
        } else {
            Some(self.reserve_price)
        }
    }

    pub fn potential_claimer_id(&self) -> Option<ProfileId> {
        self.last_bid().map(|x| x.bidder_id)
    }

    pub fn status(&self, time_now: Timestamp) -> LotStatus {
        if self.is_active(time_now) {
            LotStatus::OnSale
        } else if self.is_withdrawn {
            LotStatus::Withdrawn
        } else {
            match self.last_bid() {
                Some(_) => LotStatus::SaleSuccess,
                None => LotStatus::SaleFailure,
            }
        }
    }

    pub fn clean_up(&mut self) {
        self.bids.clear()
    }

    fn validate_claim_by_buyer(&self, claimer_id: &ProfileId, time_now: Timestamp) {
        assert!(
            !self.is_active(time_now),
            "{}",
            ERR_LOT_CLAIM_BY_BIDDER_WRONG_STATUS,
        );
        assert_eq!(
            self.potential_claimer_id().as_ref(),
            Some(claimer_id),
            "{}",
            ERR_LOT_CLAIM_BY_BIDDER_WRONG_CLAIMER,
        );
    }

    fn validate_claim_by_seller(&self, claimer_id: &ProfileId) {
        assert!(
            self.is_withdrawn,
            "{}",
            ERR_LOT_CLAIM_BY_SELLER_WRONG_STATUS,
        );
        assert_eq!(
            &self.seller_id, claimer_id,
            "{}",
            ERR_LOT_CLAIM_BY_SELLER_WRONG_CLAIMER,
        );
    }

    pub fn validate_claim(&self, claimer_id: &ProfileId, time_now: Timestamp) {
        if claimer_id == &self.seller_id {
            self.validate_claim_by_seller(claimer_id)
        } else {
            self.validate_claim_by_buyer(claimer_id, time_now)
        }
    }

    // add status
    fn validate_withdraw(&self, withdrawer_id: &ProfileId) {
        assert!(!self.is_withdrawn, "{}", ERR_LOT_WITHDRAW_WRONG_STATUS);
        assert!(self.last_bid().is_none(), "{}", ERR_LOT_WITHDRAW_HAS_BID);
        assert_eq!(
            &self.seller_id, withdrawer_id,
            "{}",
            ERR_LOT_WITHDRAW_WRONG_WITHDRAWER,
        );
    }

    pub fn withdraw(&mut self, withdrawer_id: &ProfileId) {
        self.validate_withdraw(withdrawer_id);
        self.is_withdrawn = true;
    }

    fn validate_place_bid(&mut self, bid: &Bid, bid_step: Fraction) {
        assert!(
            self.is_active(bid.timestamp),
            "{}",
            ERR_LOT_BID_WRONG_STATUS
        );
        let min_next_bid_amount = self.next_bid_amount(bid.timestamp, bid_step).unwrap();
        assert!(
            bid.amount >= min_next_bid_amount,
            "{}",
            ERR_LOT_BID_BID_TOO_SMALL
        );
        assert_ne!(
            self.seller_id, bid.bidder_id,
            "{}",
            ERR_LOT_BID_WRONG_BIDDER
        );
        assert_ne!(self.lot_id, bid.bidder_id, "{}", ERR_LOT_BID_WRONG_BIDDER);
    }

    pub fn place_bid(&mut self, bid: &Bid, bid_step: Fraction) {
        self.validate_place_bid(bid, bid_step);
        self.bids.push(bid);
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    use near_sdk_sim::{to_nanos, to_ts, to_yocto};

    pub fn create_lot_x_sells_y(seller_id: &ProfileId, lot_id: &LotId) -> Lot {
        let reserve_price = to_yocto("2");
        let buy_now_price = to_yocto("10");

        let time_now = to_ts(10);
        let duration = to_nanos(7);

        Lot::new(
            lot_id.clone(),
            seller_id.clone(),
            reserve_price,
            buy_now_price,
            time_now,
            duration,
        )
    }

    pub fn create_lot_alice() -> (Lot, Timestamp) {
        let lot = create_lot_x_sells_y(&"bob".parse().unwrap(), &"alice".parse().unwrap());
        let time_now = to_ts(16);

        (lot, time_now)
    }

    pub fn create_lot_alice_withdrawn() -> (Lot, Timestamp) {
        let (mut lot, time_now) = create_lot_alice();
        lot.is_withdrawn = true;

        (lot, time_now)
    }

    pub fn create_lot_alice_sale_failure() -> (Lot, Timestamp) {
        let (lot, _) = create_lot_alice();
        let time_now = to_ts(18);

        (lot, time_now)
    }

    pub fn create_lot_alice_with_bids() -> (Lot, Timestamp) {
        let (mut lot, time_now) = create_lot_alice();
        lot.bids.push(&Bid {
            bidder_id: "carol".parse().unwrap(),
            amount: to_yocto("3"),
            timestamp: to_ts(11),
        });
        lot.bids.push(&Bid {
            bidder_id: "dan".parse().unwrap(),
            amount: to_yocto("6"),
            timestamp: to_ts(12),
        });

        (lot, time_now)
    }

    pub fn create_lot_alice_with_bids_sale_success() -> (Lot, Timestamp) {
        let (lot, _) = create_lot_alice_with_bids();
        let time_now = to_ts(18);

        (lot, time_now)
    }

    pub fn create_lot_alice_buy_now_bid() -> (Lot, Timestamp) {
        let (mut lot, time_now) = create_lot_alice_with_bids();
        lot.bids.push(&Bid {
            bidder_id: "carol".parse().unwrap(),
            amount: to_yocto("10"),
            timestamp: to_ts(13),
        });

        (lot, time_now)
    }

    #[test]
    fn test_lot_new() {
        let (lot, _) = create_lot_alice();
        assert_eq!(lot.lot_id, "alice".parse().unwrap(), "wrong lot_id");
        assert_eq!(lot.seller_id, "bob".parse().unwrap(), "wrong seller_id");
        assert_eq!(lot.reserve_price, to_yocto("2"), "wrong reserve_price");
        assert_eq!(lot.buy_now_price, to_yocto("10"), "wrong buy_now_price");
        assert_eq!(lot.start_timestamp, to_ts(10), "wrong start_timestamp");
        assert_eq!(lot.finish_timestamp, to_ts(17), "wrong finish_timestamp");
        assert_eq!(lot.is_withdrawn, false, "expected withdrawn false");
        assert!(lot.bids.is_empty(), "expected bids list is empty");
    }

    #[test]
    #[should_panic(expected = "expected lot_id != seller_id")]
    fn test_lot_new_fail_lot_seller_same() {
        Lot::new(
            "alice".parse().unwrap(),
            "alice".parse().unwrap(),
            to_yocto("0"),
            to_yocto("0"),
            to_ts(0),
            to_nanos(0),
        );
    }

    #[test]
    #[should_panic(expected = "expected reserve_price <= buy_now_price")]
    fn test_lot_new_fail_reserve_grater_than_buy_now() {
        Lot::new(
            "alice".parse().unwrap(),
            "bob".parse().unwrap(),
            to_yocto("1"),
            to_yocto("0"),
            to_ts(0),
            to_nanos(0),
        );
    }

    #[test]
    fn test_lot_is_active_by_time_now() {
        let (lot, _) = create_lot_alice();
        assert_eq!(lot.is_active(to_ts(10) - 1), true);
        assert_eq!(lot.is_active(to_ts(10)), true);
        assert_eq!(lot.is_active(to_ts(17) - 1), true);
        assert_eq!(lot.is_active(to_ts(17)), false);
        assert_eq!(lot.is_active(to_ts(17) + 1), false);
    }

    #[test]
    fn test_lot_is_active_by_is_withdrawn() {
        let (lot, time_now) = create_lot_alice_withdrawn();
        assert_eq!(lot.is_active(time_now), false);
    }

    #[test]
    fn test_lot_is_active_by_buy_now_bid() {
        let (lot, time_now) = create_lot_alice_buy_now_bid();
        assert_eq!(
            lot.is_active(time_now),
            false,
            "expected lot inactive on buy now bid"
        );
    }

    #[test]
    fn test_lot_last_bid() {
        let (lot, _) = create_lot_alice();
        assert_eq!(lot.last_bid().map(|x| x.bidder_id), None);

        let (lot, _) = create_lot_alice_with_bids();
        assert_eq!(
            lot.last_bid().map(|x| x.bidder_id),
            Some("dan".parse().unwrap())
        );
    }

    #[test]
    fn test_lot_last_bid_amount() {
        let (lot, _) = create_lot_alice();
        assert_eq!(lot.last_bid_amount(), None, "expected none bid amount");

        let (lot, _) = create_lot_alice_with_bids();
        assert_eq!(
            lot.last_bid_amount(),
            Some(to_yocto("6")),
            "wrong last_bid_amount"
        );
    }

    #[test]
    fn test_lot_next_bid_amount() {
        let (lot, time_now) = create_lot_alice();
        assert_eq!(
            lot.next_bid_amount(time_now, Fraction::new(0, 1)).unwrap(),
            to_yocto("2"),
            "expected reserve_price for new lot"
        );

        let (lot, time_now) = create_lot_alice_with_bids();
        assert_eq!(
            lot.next_bid_amount(time_now, Fraction::new(0, 1)),
            Some(to_yocto("6") + 1),
            "expected increase by 1 yocto for zero step",
        );
        assert_eq!(
            lot.next_bid_amount(time_now, Fraction::new(1, 4)),
            Some(to_yocto("7.5")),
            "expected increase by 1 yocto for zero step",
        );
        assert_eq!(
            lot.next_bid_amount(time_now, Fraction::new(1, 1)),
            Some(to_yocto("10")),
            "expected buy now price cap",
        );

        let (lot, time_now) = create_lot_alice_with_bids_sale_success();
        assert_eq!(
            lot.next_bid_amount(time_now, Fraction::new(0, 1)),
            None,
            "expected none for inactive lot",
        );

        let (lot, time_now) = create_lot_alice_buy_now_bid();
        assert_eq!(
            lot.next_bid_amount(time_now, Fraction::new(0, 1)),
            None,
            "expected none for buy now sold lot",
        );

        let (lot, time_now) = create_lot_alice_withdrawn();
        assert_eq!(
            lot.next_bid_amount(time_now, Fraction::new(0, 1)),
            None,
            "expected none for withdrawn lot",
        );
    }

    #[test]
    fn test_lot_potential_claimer_id() {
        let (lot, _) = create_lot_alice();
        assert_eq!(lot.potential_claimer_id(), None);

        let (lot, _) = create_lot_alice_with_bids();
        assert_eq!(lot.potential_claimer_id(), Some("dan".parse().unwrap()));
    }

    #[test]
    fn test_lot_status() {
        let (lot, time_now) = create_lot_alice();
        assert_eq!(lot.status(time_now), LotStatus::OnSale);
        let (lot, time_now) = create_lot_alice_sale_failure();
        assert_eq!(lot.status(time_now), LotStatus::SaleFailure);

        let (lot, time_now) = create_lot_alice_with_bids();
        assert_eq!(lot.status(time_now), LotStatus::OnSale);

        let (lot, time_now) = create_lot_alice_with_bids_sale_success();
        assert_eq!(lot.status(time_now), LotStatus::SaleSuccess);

        let (lot, time_now) = create_lot_alice_buy_now_bid();
        assert_eq!(lot.status(time_now), LotStatus::SaleSuccess);

        let (lot, time_now) = create_lot_alice_withdrawn();
        assert_eq!(lot.status(time_now), LotStatus::Withdrawn);
    }

    #[test]
    fn test_lot_clean_up() {
        let (mut lot, _) = create_lot_alice_with_bids();
        lot.clean_up();
        assert!(lot.bids.is_empty(), "expected bids empty after clean up");
    }

    #[test]
    fn test_lot_validate_claim_by_seller() {
        let (lot, time_now) = create_lot_alice_withdrawn();
        let seller_id: AccountId = "bob".parse().unwrap();
        lot.validate_claim(&seller_id, time_now);
    }

    #[test]
    #[should_panic(expected = "claim by seller: expected status withdrawn")]
    fn test_lot_validate_claim_by_seller_fail_lot_active() {
        let (lot, time_now) = create_lot_alice();
        let seller_id: AccountId = "bob".parse().unwrap();
        lot.validate_claim(&seller_id, time_now);
    }

    #[test]
    #[should_panic(expected = "claim by seller: expected status withdrawn")]
    fn test_lot_validate_claim_by_seller_fail_lot_sale_success() {
        let (lot, time_now) = create_lot_alice_with_bids_sale_success();
        let seller_id: AccountId = "bob".parse().unwrap();
        lot.validate_claim(&seller_id, time_now);
    }

    #[test]
    #[should_panic(expected = "claim by seller: wrong claimer")]
    fn test_lot_validate_claim_by_seller_fail_wrong_claimer() {
        let (lot, _) = create_lot_alice_withdrawn();
        let fake_seller_id: AccountId = "carol".parse().unwrap();
        lot.validate_claim_by_seller(&fake_seller_id);
    }

    #[test]
    fn test_lot_validate_claim_by_bidder() {
        let (lot, time_now) = create_lot_alice_with_bids_sale_success(); // dan is the last bidder
        let bidder_id: AccountId = "dan".parse().unwrap();
        lot.validate_claim(&bidder_id, time_now);
    }

    #[test]
    #[should_panic(expected = "claim by bidder: expected status sale success")]
    fn test_lot_validate_claim_by_bidder_fail_active() {
        let (lot, time_now) = create_lot_alice_with_bids(); // dan is the last bidder
        let bidder_id: AccountId = "dan".parse().unwrap();
        lot.validate_claim(&bidder_id, time_now);
    }

    #[test]
    #[should_panic(expected = "claim by bidder: wrong claimer")]
    fn test_lot_validate_claim_by_bidder_fail_wrong_bidder() {
        let (lot, time_now) = create_lot_alice_with_bids_sale_success(); // dan is the last bidder
        let bidder_id: AccountId = "carol".parse().unwrap();
        lot.validate_claim(&bidder_id, time_now);
    }

    #[test]
    fn test_lot_withdraw() {
        let (mut lot, _) = create_lot_alice();
        let withdrawer_id: AccountId = "bob".parse().unwrap();
        lot.withdraw(&withdrawer_id);
        assert_eq!(lot.is_withdrawn, true, "expected lot to be withdrawn");
    }

    #[test]
    #[should_panic(expected = "withdraw: already withdrawn")]
    fn test_lot_withdraw_fail_already_withdrawn() {
        let (mut lot, _tm) = create_lot_alice_withdrawn();
        let withdrawer_id: AccountId = "bob".parse().unwrap();
        lot.withdraw(&withdrawer_id);
    }

    #[test]
    #[should_panic(expected = "withdraw: expected no bids")]
    fn test_lot_withdraw_fail_has_bids() {
        let (mut lot, _) = create_lot_alice_with_bids(); // dan is the last bidder
        let withdrawer_id: AccountId = "bob".parse().unwrap();
        lot.withdraw(&withdrawer_id);
    }

    #[test]
    #[should_panic(expected = "withdraw: wrong withdrawer")]
    fn test_lot_withdraw_fail_wrong_withdrawer() {
        let (mut lot, _) = create_lot_alice();
        let not_withdrawer_id: AccountId = "alice".parse().unwrap();
        lot.withdraw(&not_withdrawer_id);
    }

    #[test]
    fn test_lot_place_bid() {
        let (mut lot, time_now) = create_lot_alice();
        let bid = Bid {
            bidder_id: "dan".parse().unwrap(),
            amount: to_yocto("3"),
            timestamp: time_now,
        };
        lot.place_bid(&bid, Fraction::new(0, 1));
        assert_eq!(lot.bids.len(), 1, "{}", "expected bids size 1");
    }

    #[test]
    #[should_panic(expected = "bid: expected status active")]
    fn test_lot_place_bid_fail_inactive() {
        let (mut lot, time_now) = create_lot_alice_sale_failure();
        let bid = Bid {
            bidder_id: "dan".parse().unwrap(),
            amount: to_yocto("3"),
            timestamp: time_now,
        };
        lot.place_bid(&bid, Fraction::new(0, 1));
    }

    #[test]
    #[should_panic(expected = "bid: expected bigger bid")]
    fn test_lot_place_bid_fail_too_small() {
        let (mut lot, time_now) = create_lot_alice_with_bids();
        let bid = Bid {
            bidder_id: "dan".parse().unwrap(),
            amount: to_yocto("3"),
            timestamp: time_now,
        };
        lot.place_bid(&bid, Fraction::new(0, 1));
    }

    #[test]
    #[should_panic(expected = "bid: seller and lot cannot bid")]
    fn test_lot_place_bid_fail_bid_from_seller() {
        let (mut lot, time_now) = create_lot_alice();
        let bid = Bid {
            bidder_id: "bob".parse().unwrap(),
            amount: to_yocto("3"),
            timestamp: time_now,
        };
        lot.place_bid(&bid, Fraction::new(0, 1));
    }

    #[test]
    #[should_panic(expected = "bid: seller and lot cannot bid")]
    fn test_lot_place_bid_fail_bid_from_lot() {
        let (mut lot, time_now) = create_lot_alice();
        let bid = Bid {
            bidder_id: "alice".parse().unwrap(),
            amount: to_yocto("3"),
            timestamp: time_now,
        };
        lot.place_bid(&bid, Fraction::new(0, 1));
    }
}
