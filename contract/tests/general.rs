use near_sdk::{AccountId, Balance};
use near_sdk_sim::{call, deploy, init_simulator, to_yocto, view, ContractAccount, UserAccount};

use marketplace::{ContractContract, LotView, ERR_LOT_SELLS_SELF};

near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
    COUNTER_BYTES => "res/marketplace.wasm",
}

fn init() -> (UserAccount, ContractAccount<ContractContract>) {
    let root = init_simulator(None);

    // Deploy the compiled Wasm bytes
    let counter: ContractAccount<ContractContract> = deploy!(
        contract: ContractContract,
        contract_id: "marketplace".to_string(),
        bytes: &COUNTER_BYTES,
        signer_account: root
    );

    (root, counter)
}

// useless for now, will be helpful later
fn create_user(root: &UserAccount, name: &str) -> UserAccount {
    root.create_user(name.parse().unwrap(), to_yocto("10"))
}

const DAY_NANOSECONDS: u64 = 10u64.pow(9) * 60 * 60 * 24;

#[test]
fn simulate_lot_offer_self() {
    let (root, contract) = init();
    let alice: UserAccount = create_user(&root, "alice");
    // let bob: UserAccount = create_user(&root, "bob");

    let account_id: AccountId = alice.account_id.clone();

    let result = call!(
        alice,
        contract.lot_offer(
            alice.account_id.clone(),
            to_yocto("3").into(),
            to_yocto("10").into(),
            DAY_NANOSECONDS * 10
        )
    );
    assert!(format!("{:?}", result.status()).contains(ERR_LOT_SELLS_SELF));
    assert!(!result.is_ok(), "Should panic");
}
