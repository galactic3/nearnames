use near_sdk::{AccountId, Balance};
use near_sdk_sim::{call, deploy, init_simulator, to_yocto, view, ContractAccount, UserAccount};

use marketplace::{ContractContract, LotView, WrappedBalance};

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
