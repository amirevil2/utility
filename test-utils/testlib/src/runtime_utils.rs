use unc_chain_configs::Genesis;
use unc_crypto::PublicKey;
use unc_primitives::account::{AccessKey, Account};
use unc_primitives::hash::hash;
use unc_primitives::state_record::StateRecord;
use unc_primitives::types::{AccountId, Balance};

pub fn alice_account() -> AccountId {
    "alice".parse().unwrap()
}
pub fn bob_account() -> AccountId {
    "bob".parse().unwrap()
}
pub fn carol_account() -> AccountId {
    "carl".parse().unwrap()
}
pub fn dan_account() -> AccountId {
    "dan".parse().unwrap()
}

pub fn x_dot_y_dot_z_account() -> AccountId {
    "x.y.z".parse().unwrap()
}

/// Pre-deploy in genesis the standard test contract for a given account.
///
/// This contract contains various functions useful for testing and its code is available in
/// `/home/utility/runtime/unc-test-contracts/test-contract-rs/src/lib.rs`
pub fn add_test_contract(genesis: &mut Genesis, account_id: &AccountId) {
    add_contract(genesis, account_id, unc_test_contracts::rs_contract().to_vec())
}

/// Pre-deploy in genesis any contract for a given account.
pub fn add_contract(genesis: &mut Genesis, account_id: &AccountId, code: Vec<u8>) {
    let mut is_account_record_found = false;
    let hash = hash(&code);
    let records = genesis.force_read_records().as_mut();
    for record in records.iter_mut() {
        if let StateRecord::Account { account_id: record_account_id, ref mut account } = record {
            if record_account_id == account_id {
                is_account_record_found = true;
                account.set_code_hash(hash);
            }
        }
    }
    if !is_account_record_found {
        records.push(StateRecord::Account {
            account_id: account_id.clone(),
            account: Account::new(0, 0, 0, hash, 0),
        });
    }
    records.push(StateRecord::Contract { account_id: account_id.clone(), code });
}

/// Add an account with a specified access key & balance to the genesis state records.
pub fn add_account_with_access_key(
    genesis: &mut Genesis,
    account_id: AccountId,
    balance: Balance,
    public_key: PublicKey,
    access_key: AccessKey,
) {
    let records = genesis.force_read_records().as_mut();
    records.push(StateRecord::Account {
        account_id: account_id.clone(),
        account: Account::new(balance, 0, 0, Default::default(), 0),
    });
    records.push(StateRecord::AccessKey { account_id, public_key, access_key });
}
