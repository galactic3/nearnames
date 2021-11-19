use crate::*;

pub const ERR_LOT_SELLS_SELF: &str = "Expected lot id not equal to seller id";
pub const ERR_LOT_PRICE_RESERVE_GREATER_THAN_BUY_NOW: &str =
    "Expected reserve_price greater or equal buy_now_price";
pub const ERR_LOT_BID_LOT_NOT_ACTIVE: &str = "Expected lot to be active, cannot bid";
pub const ERR_LOT_BID_BID_TOO_SMALL: &str = "Expected bigger bid, try again";
pub const ERR_LOT_BID_SELLER_BIDS_SELF: &str = "Expected bidder_id is not equal to seller_id";

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

    pub bids: Vector<Bid>,
}

impl Lot {
    pub fn is_active(&self, time_now: Timestamp) -> bool {
        if time_now >= self.finish_timestamp {
            return false;
        }
        if let Some(last_bid_amount) = self.last_bid_amount() {
            if last_bid_amount >= self.buy_now_price {
                return false;
            }
        }

        true
    }

    pub fn last_bid_amount(&self) -> Option<Balance> {
        if self.bids.is_empty() {
            None
        } else {
            Some(self.bids.get(self.bids.len() - 1).unwrap().amount)
        }
    }

    pub fn next_bid_amount(&self, time_now: Timestamp) -> Option<Balance> {
        if !self.is_active(time_now) {
            return None;
        }
        if let Some(last_bid_amount) = self.last_bid_amount() {
            Some(std::cmp::max(self.reserve_price, last_bid_amount + 1))
        } else {
            Some(self.reserve_price)
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct LotView {
    pub lot_id: LotId,
    pub seller_id: ProfileId,
    pub reserve_price: WrappedBalance,
    pub buy_now_price: WrappedBalance,
    pub start_timestamp: WrappedTimestamp,
    pub finish_timestamp: WrappedTimestamp,
    pub is_active: bool,
}

impl From<(&Lot, Timestamp)> for LotView {
    fn from(args: (&Lot, Timestamp)) -> Self {
        let (lot, now) = args;
        Self {
            lot_id: lot.lot_id.clone(),
            seller_id: lot.seller_id.clone(),
            reserve_price: lot.reserve_price.into(),
            buy_now_price: lot.buy_now_price.into(),
            start_timestamp: lot.start_timestamp.into(),
            finish_timestamp: lot.finish_timestamp.into(),
            is_active: lot.is_active(now),
        }
    }
}

impl Contract {
    pub(crate) fn internal_lot_create(
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
            ERR_LOT_PRICE_RESERVE_GREATER_THAN_BUY_NOW,
        );

        // TODO: do we still nid to hash the key
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
            bids: Vector::new(prefix),
        }
    }

    pub(crate) fn internal_lot_extract(&mut self, lot_id: &LotId) -> Lot {
        self.lots.remove(&lot_id).unwrap()
    }

    pub(crate) fn internal_lot_save(&mut self, lot: &Lot) {
        assert!(self.lots.insert(&lot.lot_id, lot).is_none());
    }

    pub(crate) fn internal_lot_bid(&mut self, lot_id: &LotId, bid: &Bid) {
        let mut lot = self.internal_lot_extract(lot_id);
        assert!(
            lot.is_active(bid.timestamp),
            "{}",
            ERR_LOT_BID_LOT_NOT_ACTIVE
        );
        assert!(
            bid.amount >= lot.next_bid_amount(bid.timestamp).unwrap(),
            "{}",
            ERR_LOT_BID_BID_TOO_SMALL
        );
        assert_ne!(
            lot.seller_id, bid.bidder_id,
            "{}",
            ERR_LOT_BID_SELLER_BIDS_SELF
        );
        assert_ne!(
            lot.lot_id, bid.bidder_id,
            "{}",
            ERR_LOT_BID_SELLER_BIDS_SELF
        );

        lot.bids.push(bid);
        self.internal_lot_save(&lot);
    }
}

#[near_bindgen]
impl Contract {
    pub fn lot_list(&self) -> Vec<LotView> {
        let now = env::block_timestamp();
        self.lots.values().map(|v| (&v, now).into()).collect()
    }

    pub fn lot_offer(
        &mut self,
        seller_id: ValidAccountId,
        reserve_price: Balance,
        buy_now_price: Balance,
        duration: Duration,
    ) -> bool {
        let lot_id: LotId = env::predecessor_account_id();
        let seller_id: ProfileId = seller_id.into();
        let time_now = env::block_timestamp();

        let lot = Contract::internal_lot_create(
            lot_id,
            seller_id,
            reserve_price,
            buy_now_price,
            time_now,
            duration,
        );
        self.internal_lot_save(&lot);
        true
    }
}
