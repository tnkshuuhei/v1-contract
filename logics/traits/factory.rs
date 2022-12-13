use crate::traits::pair::PairError;
use ink_env::Hash;
use openbrush::traits::AccountId;

#[openbrush::wrapper]
pub type FactoryRef = dyn Factory;

#[openbrush::trait_definition]
pub trait Factory {
		#[ink(message)]
		fn owner(&self) -> u8;
		
		#[ink(message)]
		fn fee_amount_tickspacing(&self,fee: u8) -> u8;

		
		#[ink(message)]
		fn get_pool(&self, token_a: AccountId, token_b: AccountId, fee:u8) -> Option<AccountId>;
		
    #[ink(message)]
    fn create_pool(
			&mut self,
			token_a: AccountId,
			token_b: AccountId,
			fee:u8
			tick_spacing: i8,
			pool_contract: AccountId
    ) -> Result<AccountId, FactoryError>;
		
		#[ink(message)]
		fn set_owner(&self, _owner: AccountId) -> AccountId;
		
		#[ink(message)]
		fn enable_fee_amount(mut &self, fee:u8, tickspacing:u8) -> AccountId;

}

#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum FactoryError {
    PairError(PairError),
    CallerIsNotFeeSetter,
    ZeroAddress,
    IdenticalAddresses,
    PairExists,
		TickSpacingIsZero,
}

impl From<PairError> for FactoryError {
    fn from(error: PairError) -> Self {
        FactoryError::PairError(error)
    }
}
