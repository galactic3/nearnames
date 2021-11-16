use crate::*;

pub const ERR_LOT_SELLS_SELF: &str = "Expected lot id not equal to seller id";
pub const ERR_LOT_PRICE_RESERVE_GREATER_THAN_BUY_NOW: &str =
    "Expected reserve_price greater or equal buy_now_price";

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Lot {
    pub lot_id: LotId,
    pub seller_id: ProfileId,
    pub reserve_price: Balance,
    pub buy_now_price: Balance,
    pub start_timestamp: Timestamp,
    pub finish_timestamp: Timestamp,
}

impl Lot {
    pub fn is_active(&self, now: Timestamp) -> bool {
        now <= self.finish_timestamp
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

        Lot {
            lot_id,
            seller_id,
            reserve_price,
            buy_now_price,
            start_timestamp: time_now,
            finish_timestamp: time_now + duration,
        }
    }

    pub(crate) fn internal_lot_extract(&mut self, lot_id: &LotId) -> Lot {
        self.lots.remove(&lot_id).unwrap()
    }

    pub(crate) fn internal_lot_save(&mut self, lot: &Lot) {
        assert!(self.lots.insert(&lot.lot_id, lot).is_none());
    }
}

#[near_bindgen]
impl Contract {
    pub fn lot_list(&self) -> Vec<LotView> {
        let now = env::block_timestamp();
        self.lots.values().map(|v| (&v, now).into()).collect()
    }
}
