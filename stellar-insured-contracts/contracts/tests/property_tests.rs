use proptest::prelude::*;
use soroban_sdk::{Env, Address};

use crate::Contract;

proptest! {
    #[test]
    fn total_claim_amount_never_negative(amount in 1u32..10000) {
        let env = Env::default();
        let contract_id = env.register_contract(None, Contract);
        let client = crate::ContractClient::new(&env, &contract_id);

        let user = Address::generate(&env);

        client.create_claim(&user, &amount);

        let stored = client.get_total_claims();

        prop_assert!(stored >= 0);
    }
}