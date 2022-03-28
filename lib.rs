#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod pool {
    use erc20::Erc20Ref;

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        UnknownAsset,
        InsufficientBalance
    }

    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    #[ink(storage)]
    pub struct Pool {
        /// Stores a single `bool` value on the storage.
        token_0: Erc20Ref,
        token_1: Erc20Ref,
        lp: Erc20Ref
    }

    impl Pool {
        /// Constructor that initializes the `bool` value to the given `init_value`.
        #[ink(constructor)]
        pub fn new(
            token_0: Erc20Ref,
            token_1: Erc20Ref,
            lp_code_hash: Hash
        ) -> Self {
            let total_balance = Self::env().balance();
            let version:u32 = 1;
            let salt = version.to_le_bytes();
            let lp = Erc20Ref::new(
                ink_prelude::string::String::from("Pool Token"),
                ink_prelude::string::String::from("LP"),
                0,
                Self::env().account_id()
            )
                .endowment(total_balance / 4)
                .code_hash(lp_code_hash)
                .salt_bytes(salt)
                .instantiate()
                .unwrap_or_else(|error| {
                    panic!(
                        "failed at instantiating the LP contract: {:?}",
                        error
                    )
                });
            Self {
                token_0,
                token_1,
                lp
            }
        }

        #[ink(message)]
        pub fn provide_lp(
            &mut self,
            value0: Balance,
            value1: Balance
        ) -> Result<(),Error> {
            let sender = Self::env().caller();
            self.token_0.transfer_from(sender, Self::env().account_id(), value0).unwrap();
            self.token_1.transfer_from(sender, Self::env().account_id(), value1).unwrap();
            Ok(())
        }

        #[ink(message)]
        pub fn swap(
            &mut self,
            from: AccountId,
            value: Balance
        ) -> Result<(),Error> {
            let sender = Self::env().caller();
            let mut offer_asset;
            let mut ask_asset;
            if from == self.token_0.address() {
                offer_asset = self.token_0.clone();
                ask_asset = self.token_1.clone();
            } else if from == self.token_1.address() {
                offer_asset = self.token_1.clone();
                ask_asset = self.token_0.clone();
            } else {
                return Err(Error::UnknownAsset)
            };
            offer_asset.transfer_from(sender, Self::env().account_id(), value).unwrap();
            let offer_supply = offer_asset.total_supply();
            let ask_supply = ask_asset.total_supply();
            let return_amount = self.compute_swap(ask_supply, offer_supply, value);
            ask_asset.transfer(sender, return_amount).unwrap();
            Ok(())
        }

        #[inline]
        fn compute_swap(
            &self, 
            ask_supply: Balance,
            offer_supply: Balance,
            value: Balance
        ) -> Balance {
            let cp = offer_supply * ask_supply;
            //let divisor = cp.checked_div(offer_supply + value).unwrap();
            let divisor = cp/(offer_supply + value);
            ask_supply - divisor
        }

        #[ink(message)]
        pub fn token0_balance(&self) -> Balance {
            self.token_0.balance_of(Self::env().account_id())
        }

        #[ink(message)]
        pub fn token1_balance(&self) -> Balance {
            self.token_1.balance_of(Self::env().account_id())
        }

        #[ink(message)]
        pub fn lp_balance(&self) -> Balance {
            self.lp.balance_of(Self::env().account_id())
        }

        #[ink(message)]
        pub fn simulate(
            &self,
            from: AccountId,
            value: Balance
        ) -> Balance {
            self.lp.balance_of(Self::env().account_id());
            let offer_asset;
            let ask_asset;
            if from == self.token_0.address() {
                offer_asset = self.token_0.clone();
                ask_asset = self.token_1.clone();
            } else if from == self.token_1.address() {
                offer_asset = self.token_1.clone();
                ask_asset = self.token_0.clone();
            } else {
                return 0;
            }
            let offer_supply = offer_asset.total_supply();
            let ask_supply = ask_asset.total_supply();
            let return_amount = self.compute_swap(ask_supply, offer_supply, value);
            return_amount
        }
    }
}
