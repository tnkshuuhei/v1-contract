#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

use ink_lang as ink;

#[openbrush::contract]
mod factory {
    use ink_lang::{
        codegen::{
            EmitEvent,
            Env,
        },
        ToAccountId,
    };
		
		#[ink(event)]
		pub struct OwnerChanged {
			pub old_owner: AccountId,
			pub new_owner: AccountId,
		}

    #[ink(event)]
    pub struct PoolCreated {
			#[ink(topic)]
			pub token_0: AccountId,
			#[ink(topic)]
			pub token_1: AccountId,
			pub fee: u8,
			pub tickspacing: i8,
			pub pool: AccountId,
    }
		#[ink(event)]
		pub struct FeeAmountEnabled {
			#[ink(topic)]
			pub fee: u8,
			
			#[ink(topic)]
			pub tickspacing: i8,
		}
    #[ink(storage)]
    #[derive(Default, SpreadAllocate, Storage)]
    pub struct FactoryContract {
        #[storage_field]
        factory: data::Data,
    }

    impl Factory for FactoryContract {}

    impl factory::Internal for FactoryContract {
        fn _instantiate_pool(&mut self, salt_bytes: &[u8]) -> Result<AccountId, FactoryError> {
            let pair_hash = self.factory.pair_contract_code_hash;
            let pair = PairContractRef::new()
                .endowment(0)
                .code_hash(pair_hash)
                .salt_bytes(&salt_bytes[..4])
                .instantiate()
                .map_err(|_| FactoryError::PairInstantiationFailed)?;
            Ok(pair.to_account_id())
        }

        fn _emit_create_pool_event(
            &self,
            token_0: AccountId,
            token_1: AccountId,
						fee: u8,
						tick_spacing: i8,
						pool: AccountId,
        ) {
            EmitEvent::<FactoryContract>::emit_event(
                self.env(),
                PoolCreated {
                    token_0,
                    token_1,
                    fee,
                    tick_spacing,
										pool
                },
            )
        }
    }

    impl FactoryContract {
        #[ink(constructor)]
        pub fn new(fee_to_setter: AccountId, pair_code_hash: Hash) -> Self {
            ink_lang::codegen::initialize_contract(|instance: &mut Self| {
                instance.factory.pair_contract_code_hash = pair_code_hash;
								let caller = Self::env().caller();
								fn _emit_owner_changed(
									&self,
									old_owner: AccountId,
									new_owner: AccountId
							) {
									EmitEvent::<FactoryContract>::emit_event(
											self.env(),
											OwnerChanged {
												old_owner, //ZeroAccount
												caller
											},
									)
							}
							// TODO feeamountenabled event
            })
        }
    }
    #[cfg(test)]
    mod tests {
        use ink_env::{
            test::default_accounts,
            Hash,
        };
        use openbrush::traits::AccountIdExt;

        use super::*;

        #[ink_lang::test]
        fn initialize_works() {
            let accounts = default_accounts::<ink_env::DefaultEnvironment>();
            let factory = FactoryContract::new(accounts.alice, Hash::default());
            assert!(factory.factory.fee_to.is_zero());
        }
    }
}
