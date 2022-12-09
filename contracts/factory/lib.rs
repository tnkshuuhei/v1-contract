#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

use ink_lang as ink;

#[ink::contract]
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
			pub tickspacing: u8,
			pub pool: AccountId,
			pub pool_len: u64,
    }
		#[ink(event)]
		pub struct FeeAmountEnabled {
			#[ink(topic)]
			pub fee: u8,
			
			#[ink(topic)]
			pub tickspacing: u8,
		}

    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    #[ink(storage)]
    pub struct Factory {
        /// Stores a single `bool` value on the storage.
        value: bool,
    }

    impl Factory {
        /// Constructor that initializes the `bool` value to the given `init_value`.
        #[ink(constructor)]
        pub fn new(init_value: bool) -> Self {
            Self { value: init_value }
        }

        /// Constructor that initializes the `bool` value to `false`.
        ///
        /// Constructors can delegate to other constructors.
        #[ink(constructor)]
        pub fn default() -> Self {
            Self::new(Default::default())
        }

        /// A message that can be called on instantiated contracts.
        /// This one flips the value of the stored `bool` from `true`
        /// to `false` and vice versa.
        #[ink(message)]
        pub fn flip(&mut self) {
            self.value = !self.value;
        }

        /// Simply returns the current value of our `bool`.
        #[ink(message)]
        pub fn get(&self) -> bool {
            self.value
        }
    }

    /// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
    /// module and test functions are marked with a `#[test]` attribute.
    /// The below code is technically just normal Rust code.
    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// Imports `ink_lang` so we can use `#[ink::test]`.
        use ink_lang as ink;

        /// We test if the default constructor does its job.
        #[ink::test]
        fn default_works() {
            let factory = Factory::default();
            assert_eq!(factory.get(), false);
        }

        /// We test a simple use case of our contract.
        #[ink::test]
        fn it_works() {
            let mut factory = Factory::new(false);
            assert_eq!(factory.get(), false);
            factory.flip();
            assert_eq!(factory.get(), true);
        }
    }
}
