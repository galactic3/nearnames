use crate::*;

pub const NO_DEPOSIT: Balance = 0;
pub const GAS_EXT_CALL_UNLOCK: u64 = 40_000_000_000_000;
pub const GAS_EXT_CALL_CLEAN_UP: u64 = 100_000_000_000_000;

pub const ERR_LOT_CLEAN_UP_STILL_ACTIVE: &str = "UNREACHABLE: cannot clean up still active lot";
pub const ERR_LOT_CLEAN_UP_UNLOCK_FAILED: &str = "Expected unlock promise to be successful";
pub const ERR_INTERNAL_LOT_SAVE_ALREADY_EXISTS: &str = "internal_lot_save: lot already exists";
pub const ERR_INTERNAL_LOT_EXTRACT_NOT_EXIST: &str = "internal_lot_extract: lot does not exist";

#[derive(Deserialize, Serialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct LotView {
    pub lot_id: LotId,
    pub seller_id: ProfileId,
    pub last_bidder_id: Option<ProfileId>,
    pub reserve_price: WrappedBalance,
    pub buy_now_price: WrappedBalance,
    pub start_timestamp: WrappedTimestamp,
    pub finish_timestamp: WrappedTimestamp,
    pub last_bid_amount: Option<WrappedBalance>,
    pub next_bid_amount: Option<WrappedBalance>,
    pub is_active: bool,
    pub is_withdrawn: bool,
    pub status: String,
}

// TODO: convert to regular meethod
impl From<(&Lot, Timestamp, &Contract)> for LotView {
    fn from(args: (&Lot, Timestamp, &Contract)) -> Self {
        let (lot, now, contract) = args;
        let last_bid = lot.last_bid();

        Self {
            lot_id: lot.lot_id.clone(),
            seller_id: lot.seller_id.clone(),
            last_bidder_id: last_bid.as_ref().map(|x| x.bidder_id.clone()),
            reserve_price: lot.reserve_price.into(),
            buy_now_price: lot.buy_now_price.into(),
            start_timestamp: lot.start_timestamp.into(),
            finish_timestamp: lot.finish_timestamp.into(),
            last_bid_amount: last_bid.as_ref().map(|x| x.amount.into()),
            next_bid_amount: lot
                .next_bid_amount(now, contract.bid_step.clone())
                .map(|x| x.into()),
            is_active: lot.is_active(now),
            is_withdrawn: lot.is_withdrawn,
            status: lot.status(now).to_string(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct BidView {
    pub bidder_id: ProfileId,
    pub amount: WrappedBalance,
    pub timestamp: WrappedTimestamp,
}

impl From<&Bid> for BidView {
    fn from(bid: &Bid) -> Self {
        Self {
            bidder_id: bid.bidder_id.clone(),
            amount: bid.amount.into(),
            timestamp: bid.timestamp.into(),
        }
    }
}

impl Contract {
    pub(crate) fn internal_lot_extract(&mut self, lot_id: &LotId) -> Lot {
        self.lots
            .remove(&lot_id)
            .expect(ERR_INTERNAL_LOT_EXTRACT_NOT_EXIST)
    }

    pub(crate) fn internal_lot_save(&mut self, lot: &Lot) {
        assert!(
            self.lots.insert(&lot.lot_id, lot).is_none(),
            "{}",
            ERR_INTERNAL_LOT_SAVE_ALREADY_EXISTS,
        );
    }
}

#[near_bindgen]
impl Contract {
    pub fn lot_bid_list(&self, lot_id: LotId) -> Vec<BidView> {
        let lot: Lot = self.lots.get(&lot_id).unwrap();

        lot.bids().iter().map(|v| v.into()).collect()
    }

    pub fn lot_list(&self, limit: Option<u64>, offset: Option<u64>) -> Vec<LotView> {
        let now = env::block_timestamp();

        let idx_from = offset.unwrap_or(0);
        let idx_to = limit.map(|x| idx_from + x).unwrap_or(u64::MAX);
        let idx_to = std::cmp::min(idx_to, self.lots.len());

        let values_as_vector = self.lots.values_as_vector();

        (idx_from..idx_to)
            .map(|x| {
                let v = values_as_vector.get(x).unwrap();
                (&v, now, self).into()
            })
            .collect()
    }

    pub fn lot_list_offering_by(
        &self,
        profile_id: ProfileId,
        limit: Option<u64>,
        offset: Option<u64>,
    ) -> Vec<LotView> {
        let profile = self.internal_profile_get(&profile_id);
        let time_now = env::block_timestamp();
        let vector = profile.lots_offering.as_vector();

        let idx_from = offset.unwrap_or(0);
        let idx_to = limit.map(|x| idx_from + x).unwrap_or(u64::MAX);
        let idx_to = std::cmp::min(idx_to, vector.len());

        (idx_from..idx_to)
            .map(|idx| {
                let lot_id = vector.get(idx).unwrap();
                let lot = self.lots.get(&lot_id).unwrap();
                (&lot, time_now, self).into()
            })
            .collect()
    }

    pub fn lot_list_bidding_by(
        &self,
        profile_id: ProfileId,
        limit: Option<u64>,
        offset: Option<u64>,
    ) -> Vec<LotView> {
        let profile = self.internal_profile_get(&profile_id);
        let time_now = env::block_timestamp();
        let vector = profile.lots_bidding.as_vector();

        let idx_from = offset.unwrap_or(0);
        let idx_to = limit.map(|x| idx_from + x).unwrap_or(u64::MAX);
        let idx_to = std::cmp::min(idx_to, vector.len());

        (idx_from..idx_to)
            .map(|idx| {
                let lot_id = vector.get(idx).unwrap();
                let lot = self.lots.get(&lot_id).unwrap();
                (&lot, time_now, self).into()
            })
            .collect()
    }

    pub fn lot_offer(
        &mut self,
        seller_id: ProfileId,
        reserve_price: WrappedBalance,
        buy_now_price: WrappedBalance,
        finish_timestamp: Option<WrappedTimestamp>,
        duration: Option<WrappedDuration>,
    ) -> bool {
        let lot_id: LotId = env::predecessor_account_id();
        let seller_id: ProfileId = seller_id.into();
        let reserve_price: Balance = reserve_price.into();
        let buy_now_price: Balance = buy_now_price.into();

        let start_timestamp: Timestamp = env::block_timestamp();
        let finish_timestamp: Timestamp = if let Some(finish_timestamp) = finish_timestamp {
            finish_timestamp.into()
        } else {
            let duration: Duration = duration.unwrap().into();
            start_timestamp + duration
        };

        let lot = Lot::new(
            lot_id.clone(),
            seller_id.clone(),
            reserve_price,
            buy_now_price,
            start_timestamp,
            finish_timestamp,
        );
        self.internal_lot_save(&lot);

        // update associations
        {
            let mut profile = self.internal_profile_extract(&seller_id);
            profile.lots_offering.insert(&lot_id);
            self.internal_profile_save(&profile);
        }

        true
    }

    #[payable]
    pub fn lot_bid(&mut self, lot_id: ProfileId) -> bool {
        let bidder_id: ProfileId = env::predecessor_account_id();
        let amount: Balance = env::attached_deposit();
        let timestamp = env::block_timestamp();
        let bid: Bid = Bid {
            bidder_id: bidder_id.clone(),
            amount,
            timestamp,
        };

        let mut lot = self.internal_lot_extract(&lot_id);
        let prev_bid: Option<Bid> = lot.last_bid();
        lot.place_bid(&bid, self.bid_step);
        self.internal_lot_save(&lot);

        // update associations
        let mut bidder = self.internal_profile_extract(&bidder_id);
        bidder.lots_bidding.insert(&lot_id);
        self.internal_profile_save(&bidder);

        let (to_prev_bidder, to_seller) = calc_lot_bid_rewards(
            prev_bid.as_ref().map(|x| x.amount),
            bid.amount,
            self.seller_rewards_commission,
            self.prev_bidder_commission_share,
        );
        if let Some(to_prev_bidder) = to_prev_bidder {
            self.internal_profile_rewards_transfer(
                &prev_bid.as_ref().unwrap().bidder_id,
                to_prev_bidder,
            );
        }
        self.internal_profile_rewards_transfer(&lot.seller_id, to_seller);

        true
    }

    pub fn lot_claim(&mut self, lot_id: LotId, public_key: PublicKey) -> Promise {
        let claimer_id: ProfileId = env::predecessor_account_id();
        let time_now = env::block_timestamp();
        let lot: Lot = self.lots.get(&lot_id).unwrap();

        lot.validate_claim(&claimer_id, time_now);

        ext_lock_contract::unlock(
            public_key,
            lot_id.clone(),
            NO_DEPOSIT,
            GAS_EXT_CALL_UNLOCK.into(),
        )
        .then(ext_self_contract::lot_after_claim_clean_up(
            lot_id.clone(),
            env::current_account_id(),
            NO_DEPOSIT,
            GAS_EXT_CALL_CLEAN_UP.into(),
        ))
    }

    #[private]
    pub fn lot_after_claim_clean_up(&mut self, lot_id: LotId) -> bool {
        assert!(is_promise_success(), "{}", ERR_LOT_CLEAN_UP_UNLOCK_FAILED);
        let time_now = env::block_timestamp();
        let mut lot: Lot = self.internal_lot_extract(&lot_id);

        assert!(
            !lot.is_active(time_now),
            "{}",
            ERR_LOT_CLEAN_UP_STILL_ACTIVE
        );

        let bidder_ids_unique: HashSet<ProfileId> = lot
            .bids()
            .into_iter()
            .map(|x| x.bidder_id.clone())
            .collect();

        bidder_ids_unique.iter().for_each(|bidder_id| {
            // TODO: validate bid exists
            let mut profile = self.internal_profile_extract(&bidder_id);
            profile.lots_bidding.remove(&lot_id);
            self.internal_profile_save(&profile);
        });
        {
            let mut seller = self.internal_profile_extract(&lot.seller_id);
            seller.lots_offering.remove(&lot_id);
            self.internal_profile_save(&seller);
        }

        lot.clean_up();
        // lot is already deleted from lots storage, returning to persist changes

        // intentionally not inserting the lot back
        true
    }

    pub fn lot_withdraw(&mut self, lot_id: LotId) -> bool {
        let lot_id: ProfileId = lot_id.into();
        let withdrawer_id: ProfileId = env::predecessor_account_id();
        let mut lot = self.internal_lot_extract(&lot_id);
        lot.withdraw(&withdrawer_id);
        self.internal_lot_save(&lot);

        true
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::{testing_env, VMContext};
    use near_sdk_sim::{to_nanos, to_ts, to_yocto};

    use crate::lot::tests::*;
    use crate::tests::build_contract;

    pub fn get_context_view(time_now: Timestamp) -> VMContext {
        VMContextBuilder::new()
            .is_view(true)
            .block_timestamp(time_now)
            .build()
    }

    pub fn get_context_call(time_now: Timestamp, caller_id: &LotId) -> VMContext {
        VMContextBuilder::new()
            .predecessor_account_id(caller_id.clone())
            .is_view(false)
            .block_timestamp(time_now)
            .build()
    }

    pub fn get_context_pay(
        time_now: Timestamp,
        caller_id: &ProfileId,
        attached_deposit: Balance,
    ) -> VMContext {
        VMContextBuilder::new()
            .predecessor_account_id(caller_id.clone())
            .is_view(false)
            .attached_deposit(attached_deposit)
            .block_timestamp(time_now)
            .build()
    }

    pub fn create_lot_x_sells_y_api(
        contract: &mut Contract,
        seller_id: &ProfileId,
        lot_id: &LotId,
    ) -> Lot {
        let reserve_price = to_yocto("2");
        let buy_now_price = to_yocto("10");
        let start_timestamp = to_ts(10);
        let finish_timestamp = to_ts(17);

        testing_env!(get_context_call(start_timestamp, lot_id));
        contract.lot_offer(
            seller_id.clone(),
            reserve_price.into(),
            buy_now_price.into(),
            Some(WrappedTimestamp::from(finish_timestamp)),
            None,
        );

        contract.lots.get(&lot_id).unwrap()
    }

    pub fn api_lot_bid(contract: &mut Contract, lot_id: &LotId, bid: &Bid) {
        testing_env!(get_context_pay(bid.timestamp, &bid.bidder_id, bid.amount));
        contract.lot_bid(lot_id.clone());
    }

    #[test]
    fn test_api_internal_save() {
        let mut contract = build_contract();
        let (lot, _) = create_lot_alice();
        contract.internal_lot_save(&lot);
        let lot_extracted = contract.internal_lot_extract(&lot.lot_id);
        assert_eq!(lot_extracted.seller_id, lot.seller_id);
    }

    #[test]
    #[should_panic(expected = "internal_lot_save: lot already exists")]
    fn test_api_internal_save_fail_already_exists() {
        let mut contract = build_contract();
        let (lot, _) = create_lot_alice();

        contract.internal_lot_save(&lot);
        contract.internal_lot_save(&lot);
    }

    #[test]
    fn test_api_internal_extract() {
        let mut contract = build_contract();
        for i in 0..3 {
            let lot = create_lot_x_sells_y(
                &format!("seller{}", i).parse().unwrap(),
                &format!("lot{}", i).parse().unwrap(),
            );
            contract.internal_lot_save(&lot);
        }

        let lot_extracted = contract.internal_lot_extract(&"lot1".parse().unwrap());
        assert_eq!(lot_extracted.seller_id, "seller1".parse().unwrap());

        assert_eq!(contract.lots.len(), 2, "wrong lots len after extract");
    }

    #[test]
    #[should_panic(expected = "internal_lot_extract: lot does not exist")]
    fn test_api_internal_extract_fail_not_exists() {
        let mut contract = build_contract();
        let (lot, _) = create_lot_alice();
        contract.internal_lot_save(&lot);

        contract.internal_lot_extract(&"nonexistent".parse().unwrap());
    }

    #[test]
    fn test_api_lot_bid_list() {
        let mut contract = build_contract();
        let (lot, time_now) = create_lot_alice_with_bids();
        contract.internal_lot_save(&lot);

        testing_env!(get_context_view(time_now));
        let response: Vec<BidView> = contract.lot_bid_list("alice".parse().unwrap());
        let expected: Vec<BidView> = vec![
            BidView {
                bidder_id: "carol".parse().unwrap(),
                amount: WrappedBalance::from(to_yocto("3")),
                timestamp: WrappedTimestamp::from(to_ts(11)),
            },
            BidView {
                bidder_id: "dan".parse().unwrap(),
                amount: WrappedBalance::from(to_yocto("6")),
                timestamp: WrappedTimestamp::from(to_ts(12)),
            },
        ];

        assert_eq!(response.len(), expected.len(), "wrong bids length");
        assert_eq!(response[0].bidder_id, expected[0].bidder_id);
        assert_eq!(response[0].amount, expected[0].amount);
        assert_eq!(response[0].timestamp, expected[0].timestamp);

        assert_eq!(response[1].bidder_id, expected[1].bidder_id);
        assert_eq!(response[1].amount, expected[1].amount);
        assert_eq!(response[1].timestamp, expected[1].timestamp);
    }

    #[test]
    fn test_api_lot_list_empty() {
        let contract = build_contract();
        testing_env!(get_context_view(to_ts(10)));
        let response: Vec<LotView> = contract.lot_list(None, None);
        assert_eq!(response.len(), 0, "expected empty lot list");
    }

    #[test]
    fn test_api_lot_list_fields_generic_active() {
        let mut contract = build_contract();
        let (lot, time_now) = create_lot_alice_with_bids();
        contract.internal_lot_save(&lot);

        testing_env!(get_context_view(time_now));
        let response: Vec<LotView> = contract.lot_list(None, None);
        assert_eq!(response.len(), 1, "expected response length 1");
        let response = response.into_iter().next().unwrap();

        assert_eq!(response.lot_id, "alice".parse().unwrap());
        assert_eq!(response.seller_id, "bob".parse().unwrap());
        assert_eq!(response.start_timestamp, (to_ts(10)).into());
        assert_eq!(response.finish_timestamp, (to_ts(17)).into());
        assert_eq!(response.reserve_price, to_yocto("2").into());
        assert_eq!(response.buy_now_price, to_yocto("10").into());
        assert_eq!(response.last_bid_amount, Some(to_yocto("6").into()));
        assert_eq!(response.next_bid_amount, Some(to_yocto("7.2").into()));
        assert_eq!(response.is_active, true);
        assert_eq!(response.is_withdrawn, false);
        assert_eq!(response.status, "OnSale");
    }

    #[test]
    fn test_api_lot_fields_status_withdrawn() {
        let mut contract = build_contract();
        let (lot, time_now) = create_lot_alice_withdrawn();
        contract.internal_lot_save(&lot);

        testing_env!(get_context_view(time_now));
        let response: Vec<LotView> = contract.lot_list(None, None);
        assert_eq!(response.len(), 1, "expected response length 1");
        let response = response.into_iter().next().unwrap();

        assert_eq!(response.last_bid_amount, None);
        assert_eq!(response.next_bid_amount, None);
        assert_eq!(response.is_active, false);
        assert_eq!(response.is_withdrawn, true);
        assert_eq!(response.status, "Withdrawn");
    }

    #[test]
    fn test_api_lot_fields_status_sale_success() {
        let mut contract = build_contract();
        let (lot, time_now) = create_lot_alice_with_bids_sale_success();
        contract.internal_lot_save(&lot);

        testing_env!(get_context_view(time_now));
        let response: Vec<LotView> = contract.lot_list(None, None);
        assert_eq!(response.len(), 1, "expected response length 1");
        let response = response.into_iter().next().unwrap();

        assert_eq!(response.last_bid_amount, Some((to_yocto("6")).into()));
        assert_eq!(response.next_bid_amount, None);
        assert_eq!(response.is_active, false);
        assert_eq!(response.is_withdrawn, false);
        assert_eq!(response.status, "SaleSuccess");
    }

    #[test]
    fn test_api_lot_fields_status_sale_failure() {
        let mut contract = build_contract();
        let (lot, time_now) = create_lot_alice_sale_failure();
        contract.internal_lot_save(&lot);

        testing_env!(get_context_view(time_now));
        let response: Vec<LotView> = contract.lot_list(None, None);
        assert_eq!(response.len(), 1, "expected response length 1");
        let response = response.into_iter().next().unwrap();

        assert_eq!(response.last_bid_amount, None);
        assert_eq!(response.next_bid_amount, None);
        assert_eq!(response.is_active, false);
        assert_eq!(response.is_withdrawn, false);
        assert_eq!(response.status, "SaleFailure");
    }

    #[test]
    fn test_api_lot_list_present_limit_offset() {
        let mut contract = build_contract();

        for i in 0..3 {
            let lot = create_lot_x_sells_y(
                &"seller".parse().unwrap(),
                &format!("lot{}", i).parse().unwrap(),
            );
            contract.internal_lot_save(&lot);
        }

        testing_env!(get_context_view(to_ts(16)));
        {
            let result = contract.lot_list(None, None);
            assert_eq!(result.len(), 3, "wrong lot list size");
            assert_eq!(result[0].lot_id, "lot0".parse().unwrap());
            assert_eq!(result[1].lot_id, "lot1".parse().unwrap());
            assert_eq!(result[2].lot_id, "lot2".parse().unwrap());
        }
        {
            let result = contract.lot_list(Some(2), None);
            assert_eq!(result.len(), 2, "wrong lot list size");
            assert_eq!(result[0].lot_id, "lot0".parse().unwrap());
            assert_eq!(result[1].lot_id, "lot1".parse().unwrap());
        }
        {
            let result = contract.lot_list(None, Some(2));
            assert_eq!(result.len(), 1, "wrong lot list size");
            assert_eq!(result[0].lot_id, "lot2".parse().unwrap());
        }
        {
            let result = contract.lot_list(Some(2), Some(1));
            assert_eq!(result.len(), 2, "wrong lot list size");
            assert_eq!(result[0].lot_id, "lot1".parse().unwrap());
            assert_eq!(result[1].lot_id, "lot2".parse().unwrap());
        }
        {
            let result = contract.lot_list(Some(5), Some(100));
            assert_eq!(result.len(), 0, "wrong lot list size");
        }
    }

    #[test]
    fn test_api_lot_list_offering_by_limit_offset() {
        let mut contract = build_contract();
        let seller_id: ProfileId = "seller".parse().unwrap();
        for i in 0..3 {
            create_lot_x_sells_y_api(
                &mut contract,
                &seller_id,
                &format!("lot{}", i).parse().unwrap(),
            );
        }

        testing_env!(get_context_view(to_ts(16)));
        {
            let result = contract.lot_list_offering_by(seller_id.clone(), None, None);
            assert_eq!(result.len(), 3, "wrong lot list size");
            assert_eq!(result[0].lot_id, "lot0".parse().unwrap());
            assert_eq!(result[1].lot_id, "lot1".parse().unwrap());
            assert_eq!(result[2].lot_id, "lot2".parse().unwrap());
        }
        {
            let result = contract.lot_list_offering_by(seller_id.clone(), Some(2), None);
            assert_eq!(result.len(), 2, "wrong lot list size");
            assert_eq!(result[0].lot_id, "lot0".parse().unwrap());
            assert_eq!(result[1].lot_id, "lot1".parse().unwrap());
        }
        {
            let result = contract.lot_list_offering_by(seller_id.clone(), None, Some(2));
            assert_eq!(result.len(), 1, "wrong lot list size");
            assert_eq!(result[0].lot_id, "lot2".parse().unwrap());
        }
        {
            let result = contract.lot_list_offering_by(seller_id.clone(), Some(2), Some(1));
            assert_eq!(result.len(), 2, "wrong lot list size");
            assert_eq!(result[0].lot_id, "lot1".parse().unwrap());
            assert_eq!(result[1].lot_id, "lot2".parse().unwrap());
        }
        {
            let result = contract.lot_list_offering_by(seller_id.clone(), Some(5), Some(100));
            assert_eq!(result.len(), 0, "wrong lot list size");
        }
        {
            let result = contract.lot_list_offering_by("nonexistent".parse().unwrap(), None, None);
            assert_eq!(result.len(), 0, "should be zero for non existing profile");
        }
    }

    #[test]
    fn test_api_lot_list_bidding_by_limit_offset() {
        let mut contract = build_contract();
        let seller_id: ProfileId = "seller".parse().unwrap();
        let bidder_id: ProfileId = "bidder".parse().unwrap();
        for i in 0..3 {
            let lot_id = format!("lot{}", i).parse().unwrap();
            create_lot_x_sells_y_api(&mut contract, &seller_id, &lot_id);
            api_lot_bid(
                &mut contract,
                &lot_id,
                &Bid {
                    bidder_id: bidder_id.clone(),
                    amount: to_yocto("6"),
                    timestamp: to_ts(11),
                },
            );
        }

        testing_env!(get_context_view(to_ts(16)));
        {
            let result = contract.lot_list_bidding_by(bidder_id.clone(), None, None);
            assert_eq!(result.len(), 3, "wrong lot list size");
            assert_eq!(result[0].lot_id, "lot0".parse().unwrap());
            assert_eq!(result[1].lot_id, "lot1".parse().unwrap());
            assert_eq!(result[2].lot_id, "lot2".parse().unwrap());
        }
        {
            let result = contract.lot_list_bidding_by(bidder_id.clone(), Some(2), None);
            assert_eq!(result.len(), 2, "wrong lot list size");
            assert_eq!(result[0].lot_id, "lot0".parse().unwrap());
            assert_eq!(result[1].lot_id, "lot1".parse().unwrap());
        }
        {
            let result = contract.lot_list_bidding_by(bidder_id.clone(), None, Some(2));
            assert_eq!(result.len(), 1, "wrong lot list size");
            assert_eq!(result[0].lot_id, "lot2".parse().unwrap());
        }
        {
            let result = contract.lot_list_bidding_by(bidder_id.clone(), Some(2), Some(1));
            assert_eq!(result.len(), 2, "wrong lot list size");
            assert_eq!(result[0].lot_id, "lot1".parse().unwrap());
            assert_eq!(result[1].lot_id, "lot2".parse().unwrap());
        }
        {
            let result = contract.lot_list_bidding_by(bidder_id.clone(), Some(5), Some(100));
            assert_eq!(result.len(), 0, "wrong lot list size");
        }
        {
            let result = contract.lot_list_bidding_by("nonexistent".parse().unwrap(), None, None);
            assert_eq!(result.len(), 0, "should be zero for non existing profile");
        }
    }

    #[test]
    fn test_api_lot_offer() {
        let mut contract = build_contract();

        let lot_id: ProfileId = "alice".parse().unwrap();
        let seller_id: ProfileId = "bob".parse().unwrap();
        let reserve_price = to_yocto("2");
        let buy_now_price = to_yocto("10");
        let start_timestamp = to_ts(10);
        let finish_timestamp = to_ts(17);

        testing_env!(get_context_call(start_timestamp, &lot_id));
        contract.lot_offer(
            seller_id.clone(),
            reserve_price.into(),
            buy_now_price.into(),
            Some(WrappedTimestamp::from(finish_timestamp)),
            None,
        );

        let result = contract.internal_lot_extract(&lot_id);

        assert_eq!(result.lot_id, lot_id.clone());
        assert_eq!(result.seller_id, seller_id.clone());
        assert_eq!(result.start_timestamp, start_timestamp);
        assert_eq!(result.finish_timestamp, finish_timestamp);
        assert_eq!(result.reserve_price, reserve_price.into());
        assert_eq!(result.buy_now_price, buy_now_price.into());
    }

    #[test]
    fn test_api_lot_offer_duration() {
        let mut contract = build_contract();

        let lot_id: ProfileId = "alice".parse().unwrap();
        let seller_id: ProfileId = "bob".parse().unwrap();
        let reserve_price = to_yocto("2");
        let buy_now_price = to_yocto("10");
        let duration = to_nanos(7);
        let time_now = to_ts(10);

        testing_env!(get_context_call(time_now, &lot_id));
        contract.lot_offer(
            seller_id.clone(),
            reserve_price.into(),
            buy_now_price.into(),
            None,
            Some(WrappedDuration::from(duration)),
        );

        let result = contract.internal_lot_extract(&lot_id);

        assert_eq!(result.lot_id, lot_id.clone());
        assert_eq!(result.seller_id, seller_id.clone());
        assert_eq!(result.start_timestamp, time_now);
        assert_eq!(result.finish_timestamp, time_now + duration);
        assert_eq!(result.reserve_price, reserve_price.into());
        assert_eq!(result.buy_now_price, buy_now_price.into());
    }

    fn check_rewards(contract: &Contract, profile_id: &ProfileId) -> Balance {
        contract.internal_profile_get(profile_id).rewards_available()
    }

    #[test]
    pub fn test_api_lot_bid_rewards() {
        let mut contract = build_contract();
        let (lot, _) = create_lot_alice();
        contract.internal_lot_save(&lot);

        let alice: LotId = "alice".parse().unwrap();
        let bob: ProfileId = "bob".parse().unwrap();
        let carol: ProfileId = "carol".parse().unwrap();
        let dan: ProfileId = "dan".parse().unwrap();

        let first_bid_amount = to_yocto("6");
        let second_bid_amount = to_yocto("8");
        let one_bid_seller_reward = to_yocto("5.4");
        let two_bids_seller_reward = to_yocto("7.2");
        let first_bidder_reward = to_yocto("6.16");

        api_lot_bid(
            &mut contract,
            &alice,
            &Bid {
                bidder_id: carol.clone(),
                amount: first_bid_amount,
                timestamp: to_ts(11),
            },
        );

        let lot = contract.lots.get(&"alice".parse().unwrap()).unwrap();
        assert_eq!(lot.bids().len(), 1, "expected one bid");
        let last_bid = lot.last_bid().unwrap();
        assert_eq!(last_bid.amount, first_bid_amount, "wrong first bid");
        assert_eq!(last_bid.bidder_id, carol, "wrong first bidder");
        assert_eq!(last_bid.timestamp, to_ts(11), "expected start as timestamp");
        assert_eq!(check_rewards(&contract, &alice), to_yocto("0"));
        assert_eq!(check_rewards(&contract, &bob), one_bid_seller_reward);
        assert_eq!(check_rewards(&contract, &carol), to_yocto("0"));

        api_lot_bid(
            &mut contract,
            &alice,
            &Bid {
                bidder_id: dan.clone(),
                amount: second_bid_amount,
                timestamp: to_ts(12),
            },
        );

        let lot = contract.lots.get(&"alice".parse().unwrap()).unwrap();
        assert_eq!(lot.bids().len(), 2, "expected two bids");
        let last_bid = lot.last_bid().unwrap();
        assert_eq!(last_bid.amount, second_bid_amount, "wrong amount");
        assert_eq!(last_bid.bidder_id, dan, "wrong bidder");
        assert_eq!(last_bid.timestamp, to_ts(12), "wrong timestamp");

        assert_eq!(check_rewards(&contract, &alice), to_yocto("0"));
        assert_eq!(check_rewards(&contract, &bob), two_bids_seller_reward);
        assert_eq!(check_rewards(&contract, &carol), first_bidder_reward);
        assert_eq!(check_rewards(&contract, &dan), to_yocto("0"));
    }

    #[test]
    pub fn test_api_lot_list_bidding_by_offering_by_fields() {
        let alice: LotId = "alice".parse().unwrap();
        let bob: ProfileId = "bob".parse().unwrap();
        let carol: ProfileId = "carol".parse().unwrap();

        let mut contract = build_contract();
        create_lot_x_sells_y_api(&mut contract, &bob, &alice);

        api_lot_bid(
            &mut contract,
            &"alice".parse().unwrap(),
            &Bid {
                bidder_id: carol.clone(),
                amount: to_yocto("6"),
                timestamp: to_ts(11),
            },
        );

        {
            let result = contract.lot_list_offering_by(bob.clone(), None, None);
            assert_eq!(result.len(), 1, "expected 1 lot");
            let result = result.first().unwrap();
            assert_eq!(&result.lot_id, &alice, "wrong lot_id",);
            assert_eq!(
                result.last_bidder_id,
                Some(carol.clone()),
                "wrong_last_bidder",
            );
            assert_eq!(result.status, "OnSale".to_string(), "wrong status",);

            let result = contract.lot_list_offering_by("carol".parse().unwrap(), None, None);
            assert_eq!(result.len(), 0, "wrong lots_offering list for craol");
        }

        {
            let result = contract.lot_list_bidding_by(carol.clone(), None, None);
            assert_eq!(result.len(), 1, "expected 1 lot");
            let result = result.first().unwrap();
            assert_eq!(&result.lot_id, &alice, "wrong lot_id",);
            assert_eq!(
                result.last_bidder_id,
                Some(carol.clone()),
                "wrong_last_bidder",
            );
            assert_eq!(result.status, "OnSale".to_string(), "wrong status",);

            let result = contract.lot_list_bidding_by("bob".parse().unwrap(), None, None);
            assert_eq!(result.len(), 0, "wrong lots_offering list for craol");
        }
    }

    #[test]
    #[should_panic(expected = "bid: expected bigger bid")]
    pub fn test_api_lot_bid_fail_small_bid() {
        let mut contract = build_contract();
        let (lot, time_now) = create_lot_alice();
        contract.internal_lot_save(&lot);

        api_lot_bid(
            &mut contract,
            &"alice".parse().unwrap(),
            &Bid {
                bidder_id: "carol".parse().unwrap(),
                amount: to_yocto("1"),
                timestamp: time_now,
            },
        );
    }

    #[test]
    #[should_panic(expected = "bid: expected status active")]
    pub fn test_api_lot_bid_fail_inactive() {
        let mut contract = build_contract();
        let (lot, time_now) = create_lot_alice_with_bids_sale_success();
        contract.internal_lot_save(&lot);

        api_lot_bid(
            &mut contract,
            &"alice".parse().unwrap(),
            &Bid {
                bidder_id: "carol".parse().unwrap(),
                amount: to_yocto("10"),
                timestamp: time_now,
            },
        );
    }

    const NEW_PUBLIC_KEY: &str = "ed25519:KEYKEYKEYKEYKEYKEYKEYKEYKEYKEYKEYKEYKEYKEYK";

    #[test]
    pub fn test_api_lot_claim_success() {
        let mut contract = build_contract();
        let (lot, time_now) = create_lot_alice_buy_now_bid();
        contract.internal_lot_save(&lot);
        testing_env!(get_context_call(time_now, &"carol".parse().unwrap()));
        let public_key: PublicKey = NEW_PUBLIC_KEY.parse().unwrap();

        contract.lot_claim("alice".parse().unwrap(), public_key);
    }

    #[test]
    pub fn test_api_lot_claim_success_by_seller_withdrawn() {
        let mut contract = build_contract();
        let (lot, time_now) = create_lot_alice_withdrawn();
        contract.internal_lot_save(&lot);
        testing_env!(get_context_call(time_now, &"bob".parse().unwrap()));
        let public_key: PublicKey = NEW_PUBLIC_KEY.parse().unwrap();

        contract.lot_claim("alice".parse().unwrap(), public_key);
    }

    #[test]
    #[should_panic(expected = "claim by bidder: expected status sale success")]
    pub fn test_api_lot_claim_fail_still_active() {
        let mut contract = build_contract();
        let (lot, time_now) = create_lot_alice_with_bids();
        contract.internal_lot_save(&lot);
        testing_env!(get_context_call(time_now, &"dan".parse().unwrap()));
        let public_key: PublicKey = NEW_PUBLIC_KEY.parse().unwrap();

        contract.lot_claim("alice".parse().unwrap(), public_key);
    }

    #[test]
    #[should_panic(expected = "claim by bidder: wrong claimer")]
    pub fn test_api_lot_claim_fail_wrong_claimer() {
        let mut contract = build_contract();
        let (lot, time_now) = create_lot_alice_buy_now_bid();
        contract.internal_lot_save(&lot);
        testing_env!(get_context_call(time_now, &"dan".parse().unwrap()));
        let public_key: PublicKey = NEW_PUBLIC_KEY.parse().unwrap();

        contract.lot_claim("alice".parse().unwrap(), public_key);
    }

    #[test]
    pub fn test_api_lot_withdraw_success() {
        let mut contract = build_contract();
        let (lot, time_now) = create_lot_alice();
        contract.internal_lot_save(&lot);

        testing_env!(get_context_call(time_now, &"bob".parse().unwrap()));
        contract.lot_withdraw("alice".parse().unwrap());
    }

    #[test]
    #[should_panic(expected = "withdraw: expected no bids")]
    fn test_api_lot_withdraw_fail_has_bids() {
        let mut contract = build_contract();
        let (lot, time_now) = create_lot_alice_with_bids();
        contract.internal_lot_save(&lot);

        testing_env!(get_context_call(time_now, &"bob".parse().unwrap()));
        contract.lot_withdraw("alice".parse().unwrap());
    }
}
