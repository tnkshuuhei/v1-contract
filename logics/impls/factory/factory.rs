use crate::traits::pair::PoolRef;
pub use crate::{
    ensure,
    impls::factory::*,
    traits::factory::*,
};
use ink_env::{
    hash::Blake2x256,
    Hash,
};
use openbrush::{
    modifier_definition,
    modifiers,
    traits::{
        AccountId,
        AccountIdExt,
        Storage,
    },
};

impl<T> Factory for T
where
    T: Internal,
    T: Storage<data::Data>,
{
    default fn all_pairs(&self, pid: u64) -> Option<AccountId> {
        self.data::<data::Data>()
            .all_pairs
            .get(pid as usize)
            .cloned()
    }

    default fn all_pairs_length(&self) -> u64 {
        self.data::<data::Data>().all_pairs.len() as u64
    }

    default fn pair_contract_code_hash(&self) -> Hash {
        self.data::<data::Data>().pair_contract_code_hash
    }

    default fn create_pool(
        &mut self,
        token_a: AccountId,
        token_b: AccountId,
				fee: u8,
    ) -> Result<AccountId, FactoryError> {
        ensure!(token_a != token_b, FactoryError::IdenticalAddresses);
        let token_pair = if token_a < token_b {
            (token_a, token_b)
        } else {
            (token_b, token_a)
        };
        ensure!(!token_pair.0.is_zero(), FactoryError::ZeroAddress);
        ensure!(
            self.data::<data::Data>()
                .get_pool
                .get(&token_pair)
                .is_none(),
            FactoryError::PairExists
        );
				let tick_spacing:i8 = fee_amount_tick_spacing[fee];
				ensure!(tick_spacing != 0, FactoryError::TickSpacingIsZero);

        let salt = Self::env().hash_encoded::<Blake2x256, _>(&token_pair);
        let pool_contract = self._instantiate_pool(salt.as_ref());

        PoolRef::initialize(&pool_contract, token_pair.0, token_pair.1)?;

        self.data::<data::Data>()
            .get_pair
            .insert(&(token_pair.0, token_pair.1), &pool_contract);
        self.data::<data::Data>()
            .get_pair
            .insert(&(token_pair.1, token_pair.0), &pool_contract);
        self.data::<data::Data>().all_pools.push(pool_contract);

        self._emit_create_pool_event(
            token_pair.0,
            token_pair.1,
            fee,
						tick_spacing,
						&pool_contract,
        );
        Ok(pool_contract)
    }

    #[modifiers(only_fee_setter)]
    default fn set_fee_to(&mut self, fee_to: AccountId) -> Result<(), FactoryError> {
        self.data::<data::Data>().fee_to = fee_to;
        Ok(())
    }

    #[modifiers(only_fee_setter)]
    default fn set_fee_to_setter(&mut self, fee_to_setter: AccountId) -> Result<(), FactoryError> {
        self.data::<data::Data>().fee_to_setter = fee_to_setter;
        Ok(())
    }

    default fn fee_to(&self) -> AccountId {
        self.data::<data::Data>().fee_to
    }

    default fn fee_to_setter(&self) -> AccountId {
        self.data::<data::Data>().fee_to_setter
    }

    default fn get_pair(&self, token_a: AccountId, token_b: AccountId) -> Option<AccountId> {
        self.data::<data::Data>().get_pair.get(&(token_a, token_b))
    }
}

pub trait Internal {
    fn _emit_create_pair_event(
        &self,
        _token_0: AccountId,
        _token_1: AccountId,
        _pair: AccountId,
        _pair_len: u64,
    );

    fn _instantiate_pair(&mut self, salt_bytes: &[u8]) -> AccountId;
}

#[modifier_definition]
pub fn only_fee_setter<T, F, R, E>(instance: &mut T, body: F) -> Result<R, E>
where
    T: Storage<data::Data>,
    F: FnOnce(&mut T) -> Result<R, E>,
    E: From<FactoryError>,
{
    if instance.data().fee_to_setter != T::env().caller() {
        return Err(From::from(FactoryError::CallerIsNotFeeSetter))
    }
    body(instance)
}
