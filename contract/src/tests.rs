pub use super::*;

pub use near_sdk::test_utils::VMContextBuilder;
pub use near_sdk::{testing_env, VMContext};
pub use near_sdk_sim::{to_nanos, to_ts, to_yocto};

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

pub fn build_contract() -> Contract {
    Contract::new(
        FractionView { num: 1, denom: 10 },
        FractionView { num: 1, denom: 5 },
        FractionView { num: 4, denom: 5 },
    )
}
