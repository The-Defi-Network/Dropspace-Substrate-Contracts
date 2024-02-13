#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[openbrush::implementation(Ownable, PSP37, PSP37Metadata, PSP37Mintable)]
#[openbrush::contract]
pub mod dropspace_sale {
    use ink::primitives::AccountId as Address;
    use ink_prelude::format;
    use ink_prelude::string::String as PreludeString;
    use openbrush::{
        contracts::psp37::{psp37, PSP37Error},
        modifiers,
        traits::{Id, PSP37, Storage},
    };

    #[ink(storage)]
    #[derive(Default, Storage)]
    pub struct Contract {
        #[storage_field]
        psp37: psp37::Data,
        #[storage_field]
        ownable: ownable::Data,
        #[storage_field]
        metadata: metadata::Data,
        base_uri: PreludeString,
        mint_price: u128,
        mint_fee: u128,
        withdraw_wallet: Option<Address>,
        dev_wallet: Option<Address>,
        sale_time: u64,
    }

    impl Contract {
        #[ink(constructor)]
        pub fn new(
            name: PreludeString,
            symbol: PreludeString,
            base_uri: PreludeString,
            mint_price: u128,
            mint_fee: u128,
            withdraw_wallet: Option<Address>,
            dev_wallet: Option<Address>,
            sale_time: u64,
        ) -> Self {
            let mut _instance = Self {
                base_uri,
                mint_price,
                mint_fee,
                withdraw_wallet,
                dev_wallet,
                sale_time,
                ..Default::default()
            };

            ownable::Internal::_init_with_owner(&mut _instance, Self::env().caller());
            let collection_id = PSP37::collection_id(&_instance);
            metadata::Internal::_set_attribute(
                &mut _instance,
                collection_id.clone(),
                String::from("name"),
                String::from(name),
            );
            metadata::Internal::_set_attribute(
                &mut _instance,
                collection_id,
                String::from("symbol"),
                String::from(symbol),
            );
            _instance
        }

        #[ink(message)]
        pub fn mint_token(&mut self) -> Result<(), PSP37Error> {
            let current_supply: u128 = psp37::PSP37::total_supply(self, Id::U128(1));
            if current_supply >= 1 {
                return Err(PSP37Error::Custom(String::from(
                    "DropspaceSale::mint_token: Token ID 1 already minted",
                )));
            }
            psp37::Internal::_mint_to(self, Self::env().caller(), Id::U128(1))?;
            Ok(())
        }

        #[ink(message)]
        pub fn reserve(&mut self, amount: u128) -> Result<(), PSP37Error> {
            if amount != 1 {
                return Err(PSP37Error::Custom(String::from(
                    "DropspaceSale::reserve: Invalid amount, only 1 token ID 1 allowed",
                )));
            }
            let __ = self.mint_token()?;
            Ok(())
        }

        #[ink(message, payable)]
        pub fn buy(&mut self, amount: u128) -> Result<(), PSP37Error> {
            if amount != 1 {
                return Err(PSP37Error::Custom(String::from(
                    "DropspaceSale::buy: Invalid amount, only 1 token ID 1 allowed",
                )));
            }
            let total_price = self.mint_price.saturating_add(self.mint_fee);

            if self.env().block_timestamp() < self.sale_time {
                return Err(PSP37Error::Custom(String::from(
                    "DropspaceSale::buy: Sale hasn't started yet",
                )));
            }

            if self.env().transferred_value() < total_price {
                return Err(PSP37Error::Custom(String::from(
                    "DropspaceSale::buy: Wrong amount paid.",
                )));
            }

            let __ = self.mint_token()?;
            if let Some(withdraw_wallet) = self.withdraw_wallet {
                self.env()
                    .transfer(withdraw_wallet, self.mint_price)
                    .map_err(|_| {
                        PSP37Error::Custom(String::from("Transfer to owner wallet failed"))
                    })?;
            } else {
                return Err(PSP37Error::Custom(String::from("Owner wallet not set")));
            }

            if let Some(dev_wallet) = self.dev_wallet {
                if self.mint_fee > 0 {
                    self.env()
                        .transfer(dev_wallet, self.mint_fee)
                        .map_err(|_| {
                            PSP37Error::Custom(String::from("Transfer to dev wallet failed"))
                        })?;
                }
            } else {
                return Err(PSP37Error::Custom(String::from("Developer wallet not set")));
            }

            Ok(())
        }

        #[ink(message)]
        #[modifiers(only_owner)]
        pub fn set_base_uri(&mut self, uri: PreludeString) -> Result<(), PSP37Error> {
            self.base_uri = uri;
            Ok(())
        }

        #[ink(message)]

        #[ink(message)]
        #[modifiers(only_owner)]
        pub fn set_mint_price(&mut self, mint_price: u128) -> Result<(), PSP37Error> {
            self.mint_price = mint_price;
            Ok(())
        }

        #[ink(message)]
        #[modifiers(only_owner)]
        pub fn set_sale_time(&mut self, sale_time: u64) -> Result<(), PSP37Error> {
            self.sale_time = sale_time;
            Ok(())
        }

        #[ink(message)]
        #[modifiers(only_owner)]
        pub fn toggle_sale_active(&mut self) -> Result<(), PSP37Error> {
            if self.sale_time() != 0 {
                self.sale_time = 0;
            } else {
                self.sale_time = u64::MAX;
            }
            Ok(())
        }

        #[ink(message)]
        #[modifiers(only_owner)]

        #[ink(message)]
        #[modifiers(only_owner)]
        pub fn set_withdraw_wallet(
            &mut self,
            withdraw_wallet: Option<Address>,
        ) -> Result<(), PSP37Error> {
            self.withdraw_wallet = withdraw_wallet;
            Ok(())
        }

        #[ink(message)]
        pub fn token_uri(&self, token_id: u128) -> Result<PreludeString, PSP37Error> {
            let base_uri = self.base_uri.clone();
            Ok(format!("{base_uri}{token_id}"))
        }



        #[ink(message)]
        pub fn get_account_balance(&self) -> u128 {
            self.env().balance()
        }

        #[ink(message)]
        pub fn mint_price(&self) -> u128 {
            self.mint_price
        }

        #[ink(message)]
        pub fn mint_fee(&self) -> u128 {
            self.mint_fee
        }

        #[ink(message)]
        pub fn dev_wallet(&self) -> Option<Address> {
            self.dev_wallet
        }

        #[ink(message)]
        pub fn withdraw_wallet(&self) -> Option<Address> {
            self.withdraw_wallet
        }

        #[ink(message)]
        pub fn sale_time(&self) -> u64 {
            self.sale_time
        }

        #[ink(message)]
        pub fn sale_active(&self) -> bool {
            self.sale_time <= self.env().block_timestamp()
        }

        #[ink(message)]
        pub fn base_uri(&self) -> PreludeString {
            self.base_uri.clone()
        }

        #[ink(message)]
        #[modifiers(only_owner)]
        pub fn withdraw(&mut self) -> Result<(), PSP37Error> {
            let contract_balance = self.get_account_balance();

            if contract_balance > 0 {
                match self.env().transfer(Self::env().caller(), contract_balance) {
                    Ok(_) => Ok(()),
                    Err(_) => Err(PSP37Error::Custom(String::from("Withdrawal failed"))),
                }
            } else {
                Err(PSP37Error::Custom(String::from("No funds to withdraw")))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[rustfmt::skip]
    use super::*;
    use dropspace_sale::Contract;
    use ink::{env::DefaultEnvironment as Environment, primitives::AccountId};
    use openbrush::contracts::ownable::Ownable;
    use openbrush::contracts::psp34::extensions::metadata::psp34metadata_external::PSP34Metadata;
    use openbrush::contracts::psp34::{psp34, Id, PSP37Error};

    fn default_accounts() -> ink::env::test::DefaultAccounts<ink::env::DefaultEnvironment> {
        ink::env::test::default_accounts::<ink::env::DefaultEnvironment>()
    }

    struct ContractParam {
        name: String,
        symbol: String,
        base_uri: String,
        mint_price: u128,
        mint_fee: u128,
        withdraw_wallet: Option<AccountId>,
        dev_wallet: Option<AccountId>,
        sale_time: u64,
    }

    impl Default for ContractParam {
        fn default() -> ContractParam {
            ContractParam {
                name: "Test".to_string(),
                symbol: "TST".to_string(),
                base_uri: "https://example.com/token/".to_string(),
                mint_price: 1000,
                mint_fee: 10,
                withdraw_wallet: None,
                dev_wallet: None,
                sale_time: 0,
            }
        }
    }

    fn get_contract(args: &ContractParam) -> Contract {
        return Contract::new(
            args.name.to_string(),
            args.symbol.to_string(),
            args.base_uri.to_string(),
            args.mint_price,
            args.mint_fee,
            args.withdraw_wallet,
            args.dev_wallet,
            args.sale_time,
        );
    }

    #[ink::test]
    fn new_works() {
        let accounts = default_accounts();

        let params = ContractParam {
            withdraw_wallet: Some(accounts.django),
            dev_wallet: Some(accounts.alice),
            ..Default::default()
        };
        let contract = get_contract(&params);

        assert_eq!(contract.mint_price(), params.mint_price);
        assert_eq!(contract.mint_fee(), params.mint_fee);
        assert_eq!(contract.dev_wallet(), params.dev_wallet);
        assert_eq!(contract.withdraw_wallet(), params.withdraw_wallet);
        assert_eq!(contract.sale_time(), params.sale_time);

        let id = psp34::PSP34::collection_id(&contract);
        assert_eq!(
            PSP34Metadata::get_attribute(&contract, id.clone(), String::from("name")),
            Some(params.name)
        );
        assert_eq!(
            PSP34Metadata::get_attribute(&contract, id.clone(), String::from("symbol")),
            Some(params.symbol)
        );
        assert_eq!(contract.base_uri(), params.base_uri);
    }

    #[ink::test]
    fn mint_token_works() {
        let accounts = default_accounts();
        let mut contract = get_contract(&ContractParam::default());

        // Minting token should succeed
        assert_eq!(contract.mint_token(), Ok(()));

        // Minting token again should fail since only one token with ID 1 is allowed
        assert_eq!(
            contract.mint_token(),
            Err(PSP37Error::Custom(String::from(
                "DropspaceSale::mint_token: Token ID 1 already minted"
            )))
        );
    }

    #[ink::test]
    fn reserve_works() {
        let accounts = default_accounts();
        let mut contract = get_contract(&ContractParam::default());

        // Reserving one token should succeed
        assert_eq!(contract.reserve(1), Ok(()));

        // Reserving more than one token should fail
        assert_eq!(
            contract.reserve(2),
            Err(PSP37Error::Custom(String::from(
                "DropspaceSale::reserve: Invalid amount, only 1 token ID 1 allowed"
            )))
        );
    }

    #[ink::test]
    fn buy_works() {
        let accounts = default_accounts();
        let mut contract = get_contract(&ContractParam::default());

        // Buying one token should succeed
        assert_eq!(contract.buy(1), Ok(()));

        // Buying more than one token should fail
        assert_eq!(
            contract.buy(2),
            Err(PSP37Error::Custom(String::from(
                "DropspaceSale::buy: Invalid amount, only 1 token ID 1 allowed"
            )))
        );
    }

    #[ink::test]
    fn sale_active_works() {
        let accounts = default_accounts();
        let params = ContractParam {
            withdraw_wallet: Some(accounts.django),
            dev_wallet: Some(accounts.alice),
            ..Default::default()
        };
        let mut contract = get_contract(&params);

        // Set the block timestamp to simulate sale time passing
        ink::env::test::set_block_timestamp::<ink::env::DefaultEnvironment>(12345678);
        assert_eq!(contract.sale_active(), true);

        let __ = contract.set_sale_time(12345679);

        // After the sale time, sale should be active
        assert_eq!(contract.sale_active(), false);

        let __ = contract.set_sale_time(0);
        assert_eq!(contract.sale_active(), true);
    }

    #[ink::test]
    fn toggle_sale_active_works() {
        let accounts = default_accounts();
        let params = ContractParam {
            withdraw_wallet: Some(accounts.django),
            dev_wallet: Some(accounts.alice),
            ..Default::default()
        };

        // Set owner
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.charlie);

        let mut contract = get_contract(&params);

        // Ensure that only the owner can toggle sale active
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
        assert_eq!(
            contract.toggle_sale_active(),
            Err(PSP37Error::Custom(String::from("O::CallerIsNotOwner")))
        );

        assert_eq!(contract.sale_active(), true);
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.charlie);
        assert_eq!(contract.toggle_sale_active(), Ok(()));
        assert_eq!(contract.sale_active(), false);

        // Simulate the owner calling the function
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.charlie);

        // Toggle sale active, which should set the sale time to 0
        assert_eq!(contract.toggle_sale_active(), Ok(()));
        assert_eq!(contract.sale_time(), 0);
        assert_eq!(contract.sale_active(), true);

        // Toggle sale active again, which should set the sale time to u64::MAX
        assert_eq!(contract.toggle_sale_active(), Ok(()));
        assert_eq!(contract.sale_time(), u64::MAX);
        assert_eq!(contract.sale_active(), false);
    }

    #[ink::test]
    fn withdraw_works() {
        let accounts = default_accounts();
        let mut contract = Contract::new(
            "Test".to_string(),
            "TST".to_string(),
            "https://example.com/token/".to_string(),
            100000,
            10,
            1000,
            10,
            Some(accounts.django),
            Some(accounts.alice),
            Id::U8(0),
            88888888,
        );

        // Simulate buying a token
        ink::env::test::set_account_balance::<ink::env::DefaultEnvironment>(accounts.alice, 0);
        ink::env::test::set_account_balance::<ink::env::DefaultEnvironment>(accounts.charlie, 0);
        ink::env::test::set_account_balance::<ink::env::DefaultEnvironment>(
            accounts.bob,
            100_000_000,
        );

        assert_eq!(contract.toggle_sale_active(), Ok(()));
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);

        assert_eq!(
            ink::env::pay_with_call!(contract.buy(vec![(Id::U8(1), 100), (Id::U8(2), 200)]), 2020),
            Ok(())
        );

        // Check that owner's balance has increased by 10000 units
        let dev_balance =
            ink::env::test::get_account_balance::<ink::env::DefaultEnvironment>(accounts.charlie)
                .unwrap_or_default();

        // Simulate the owner calling the withdraw function
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);
        assert_eq!(contract.withdraw(), Ok(()));
    }
}