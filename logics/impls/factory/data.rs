use ink_env::Hash;
use ink_prelude::vec::Vec;
use openbrush::{
    storage::Mapping,
    traits::AccountId,
};

pub const STORAGE_KEY: u32 = openbrush::storage_unique_key!(Data);

#[derive(Default, Debug)]
#[openbrush::upgradeable_storage(STORAGE_KEY)]
pub struct Data {
    pub get_pool: Mapping<AccountId, Mapping<AccountId, Mapping<u8, AccountId>>>,
    pub all_pools: Vec<AccountId>,
    pub owner: AccountId,
		pub fee_amount_tick_spacing: Mapping<u8, u8>
}
