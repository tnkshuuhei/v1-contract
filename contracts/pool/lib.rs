#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

#[openbrush::contract]
pub mod pair {
    use ink_lang::codegen::{
        EmitEvent,
        Env,
    };
    use ink_prelude::vec::Vec;
    use ink_storage::traits::SpreadAllocate;
    use openbrush::{
        contracts::{
            ownable::*,
            psp22::*,
            reentrancy_guard,
        },
        traits::Storage,
    };
    use uniswap_v2::{
        ensure,
        impls::pair::*,
        traits::pair::*,
    };

		/////////////////////////////////////////////////////////////////////////////////////////////////////
		// poolEvent
		
    /// Emitted when liquidity is minted for a given position
    /// sender The address that minted the liquidity
    /// owner The owner of the position and recipient of any minted liquidity
    /// tickLower The lower tick of the position
    /// tickUpper The upper tick of the position
    /// amount The amount of liquidity minted to the position range
    /// amount0 How much token0 was required for the minted liquidity
    /// amount1 How much token1 was required for the minted liquidity
		
    #[ink(event)]
    pub struct Mint {
        #[ink(topic)]
        pub sender: AccountId,
				pub tick_lower: u8,
				pub tick_upper: u8,
				pub amount: Balance,
        pub amount_0: Balance,
        pub amount_1: Balance,
    }

    /// @notice Emitted when fees are collected by the owner of a position
    /// @dev Collect events may be emitted with zero amount0 and amount1 when the caller chooses not to collect fees
    /// @param owner The owner of the position for which fees are collected
    /// @param tickLower The lower tick of the position
    /// @param tickUpper The upper tick of the position
    /// @param amount0 The amount of token0 fees collected
    /// @param amount1 The amount of token1 fees collected
    #[ink(event)]
		pub struct Collect {
			#[ink(topic)]
			pub owner: AccountId,
			pub recipient: AccountId,
			pub tick_lower: u8,
			pub tick_upper: u8,
			pub amount_0: Balance,
			pub amount_1: Balance,
		}
    /// @notice Emitted when a position's liquidity is removed
    /// @dev Does not withdraw any fees earned by the liquidity position, which must be withdrawn via #collect
    /// @param owner The owner of the position for which liquidity is removed
    /// @param tickLower The lower tick of the position
    /// @param tickUpper The upper tick of the position
    /// @param amount The amount of liquidity to remove
    /// @param amount0 The amount of token0 withdrawn
    /// @param amount1 The amount of token1 withdrawn
    #[ink(event)]
    pub struct Burn {
        #[ink(topic)]
        pub sender: AccountId,
				pub tick_lower: u8,
				pub tick_upper: u8,
				pub amount: Balance,
        pub amount_0: Balance,
        pub amount_1: Balance,
    }
    /// @notice Emitted by the pool for any swaps between token0 and token1
    /// @param sender The address that initiated the swap call, and that received the callback
    /// @param recipient The address that received the output of the swap
    /// @param amount0 The delta of the token0 balance of the pool
    /// @param amount1 The delta of the token1 balance of the pool
    /// @param sqrtPriceX96 The sqrt(price) of the pool after the swap, as a Q64.96
    /// @param liquidity The liquidity of the pool after the swap
    /// @param tick The log base 1.0001 of price of the pool after the swap
    #[ink(event)]
    pub struct Swap {
        #[ink(topic)]
        pub sender: AccountId,
				pub recipient: AccountId,
				pub amount_0: Balance,
				pub amount_1: Balance,
				//TODO pub sqrtPriceX96: u128, 
				pub liquidity: u128,
				pub tick: u8,
    }
    /// @notice Emitted by the pool for any flashes of token0/token1
    /// @param sender The address that initiated the swap call, and that received the callback
    /// @param recipient The address that received the tokens from flash
    /// @param amount0 The amount of token0 that was flashed
    /// @param amount1 The amount of token1 that was flashed
    /// @param paid0 The amount of token0 paid for the flash, which can exceed the amount0 plus the fee
    /// @param paid1 The amount of token1 paid for the flash, which can exceed the amount1 plus the fee
		#[ink(event)]
		pub struct Flash {
			#[ink(topic)]
			pub sender : AccountId,
			pub recipient: AccountId,
			pub amount0: Balance,
			pub amount1: Balance,
			pub paid0: Balance,
			pub paid1: Balance,
		}
    /// @notice Emitted by the pool for increases to the number of observations that can be stored
    /// @dev observationCardinalityNext is not the observation cardinality until an observation is written at the index
    /// just before a mint/swap/burn.
    /// @param observationCardinalityNextOld The previous value of the next observation cardinality
    /// @param observationCardinalityNextNew The updated value of the next observation cardinality
		#[ink(event)]
    pub struct IncreaseObservationCardinalityNext(
			pub observation_cardinality_next_old: u8,
			pub observation_cardinality_next_new: u8,
	);

	/// @notice Emitted when the protocol fee is changed by the pool
	/// @param feeProtocol0Old The previous value of the token0 protocol fee
	/// @param feeProtocol1Old The previous value of the token1 protocol fee
	/// @param feeProtocol0New The updated value of the token0 protocol fee
	/// @param feeProtocol1New The updated value of the token1 protocol fee
	#[ink(event)]
	pub struct SetFeeProtocol{
		pub fee_protocol0_old: u8,
		pub fee_protocol1_old: u8,
		pub fee_protocol0_new: u8,
		pub fee_protocol1_new: u8,
	}

	/// @notice Emitted when the collected protocol fees are withdrawn by the factory owner
	/// @param sender The address that collects the protocol fees
	/// @param recipient The address that receives the collected protocol fees
	/// @param amount0 The amount of token0 protocol fees that is withdrawn
	/// @param amount0 The amount of token1 protocol fees that is withdrawn
	#[ink(event)]
	pub struct CollectProtocol{
		pub sender: AccountId, 
		pub recipient: AccountId, 
		pub amount0: Balance, 
		pub amount1: Balance, 
	}
	/////////////////////////////////////////////////////////////////////////////////////////////////////
    #[ink(event)]
    pub struct Sync {
        reserve_0: Balance,
        reserve_1: Balance,
    }

    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        to: Option<AccountId>,
        value: Balance,
    }

    #[ink(storage)]
    #[derive(Default, SpreadAllocate, Storage)]
    pub struct PairContract {
        #[storage_field]
        psp22: psp22::Data,
        #[storage_field]
        ownable: ownable::Data,
        #[storage_field]
        guard: reentrancy_guard::Data,
        #[storage_field]
        pair: data::Data,
    }

    impl PSP22 for PairContract {
        #[ink(message)]
        fn transfer_from(
            &mut self,
            from: AccountId,
            to: AccountId,
            value: Balance,
            data: Vec<u8>,
        ) -> Result<(), PSP22Error> {
            let caller = self.env().caller();
            let allowance = self._allowance(&from, &caller);

            // In uniswapv2 max allowance never decrease
            if allowance != u128::MAX {
                ensure!(allowance >= value, PSP22Error::InsufficientAllowance);
                self._approve_from_to(from, caller, allowance - value)?;
            }
            self._transfer_from_to(from, to, value, data)?;
            Ok(())
        }
    }

    impl psp22::Internal for PairContract {
        // in uniswapv2 no check for zero account
        fn _mint_to(&mut self, account: AccountId, amount: Balance) -> Result<(), PSP22Error> {
            let mut new_balance = self._balance_of(&account);
            new_balance += amount;
            self.psp22.balances.insert(&account, &new_balance);
            self.psp22.supply += amount;
            self._emit_transfer_event(None, Some(account), amount);
            Ok(())
        }

        fn _burn_from(&mut self, account: AccountId, amount: Balance) -> Result<(), PSP22Error> {
            let mut from_balance = self._balance_of(&account);

            ensure!(from_balance >= amount, PSP22Error::InsufficientBalance);

            from_balance -= amount;
            self.psp22.balances.insert(&account, &from_balance);
            self.psp22.supply -= amount;
            self._emit_transfer_event(Some(account), None, amount);
            Ok(())
        }

        fn _approve_from_to(
            &mut self,
            owner: AccountId,
            spender: AccountId,
            amount: Balance,
        ) -> Result<(), PSP22Error> {
            self.psp22.allowances.insert(&(&owner, &spender), &amount);
            self._emit_approval_event(owner, spender, amount);
            Ok(())
        }

        fn _transfer_from_to(
            &mut self,
            from: AccountId,
            to: AccountId,
            amount: Balance,
            _data: Vec<u8>,
        ) -> Result<(), PSP22Error> {
            let from_balance = self._balance_of(&from);

            ensure!(from_balance >= amount, PSP22Error::InsufficientBalance);

            self.psp22.balances.insert(&from, &(from_balance - amount));
            let to_balance = self._balance_of(&to);
            self.psp22.balances.insert(&to, &(to_balance + amount));

            self._emit_transfer_event(Some(from), Some(to), amount);
            Ok(())
        }

        fn _emit_transfer_event(
            &self,
            from: Option<AccountId>,
            to: Option<AccountId>,
            amount: Balance,
        ) {
            self.env().emit_event(Transfer {
                from,
                to,
                value: amount,
            });
        }
    }

    impl Ownable for PairContract {}

    impl pair::Internal for PairContract {
        fn _emit_mint_event(&self, sender: AccountId, amount_0: Balance, amount_1: Balance) {
            self.env().emit_event(Mint {
                sender,
                amount_0,
                amount_1,
            })
        }

        fn _emit_burn_event(
            &self,
            sender: AccountId,
            amount_0: Balance,
            amount_1: Balance,
            to: AccountId,
        ) {
            self.env().emit_event(Burn {
                sender,
                amount_0,
                amount_1,
                to,
            })
        }

        fn _emit_swap_event(
            &self,
            sender: AccountId,
            amount_0_in: Balance,
            amount_1_in: Balance,
            amount_0_out: Balance,
            amount_1_out: Balance,
            to: AccountId,
        ) {
            self.env().emit_event(Swap {
                sender,
                amount_0_in,
                amount_1_in,
                amount_0_out,
                amount_1_out,
                to,
            })
        }

        fn _emit_sync_event(&self, reserve_0: Balance, reserve_1: Balance) {
            self.env().emit_event(Sync {
                reserve_0,
                reserve_1,
            })
        }
    }

    impl Pair for PairContract {}

    impl PairContract {
        #[ink(constructor)]
        pub fn new() -> Self {
            ink_lang::codegen::initialize_contract(|instance: &mut Self| {
                let caller = instance.env().caller();
                instance._init_with_owner(caller);
                instance.pair.factory = caller;
            })
        }
    }
    #[cfg(test)]
    mod tests {
        use ink_env::AccountId;

        use super::*;

        #[ink_lang::test]
        fn initialize_works() {
            let mut pair = PairContract::new();
            let token_0 = AccountId::from([0x03; 32]);
            let token_1 = AccountId::from([0x04; 32]);
            assert_eq!(pair.initialize(token_0, token_1), Ok(()));
        }
    }
}