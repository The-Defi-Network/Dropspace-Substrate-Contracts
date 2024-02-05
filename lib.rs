#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[openbrush::implementation(Ownable, PSP37, PSP37Metadata, PSP37Mintable)]
#[openbrush::contract]
pub mod dropspace_sale {
    use ink::prelude::vec;
    use ink::primitives::AccountId as Address;
    use ink_prelude::format;
    use ink_prelude::string::String as PreludeString;
    use openbrush::{
        contracts::{
            ownable::{self},
            psp37::{
                self,
                extensions::metadata::{self},
                Id,
            },
        },
        modifiers,
        storage::Mapping,
        traits::{Storage, String},
    };

    #[derive(Default, Storage)]
    #[ink(storage)]
    pub struct Contract {
        #[storage_field]
        psp37: psp37::Data,
        denied_ids: Mapping<Id, ()>,
        #[storage_field]
        ownable: ownable::Data,
        #[storage_field]
        metadata: metadata::Data,
        base_uri: PreludeString,
        mint_per_tx: u128,
        mint_price: u128,
        mint_fee: u128,
        withdraw_wallet: Option<Address>,
        dev_wallet: Option<Address>,
        sale_time: u64,
        id_lists: u128,
        supply_limit: u128,
    }

    impl Contract {
        #[ink(constructor)]
        pub fn new(
            name: PreludeString,
            symbol: PreludeString,
            base_uri: PreludeString,
            supply_limit: u128,
            mint_per_tx: u128,
            mint_price: u128,
            mint_fee: u128,
            withdraw_wallet: Option<Address>,
            dev_wallet: Option<Address>,
            id: Id,
            sale_time: u64,
        ) -> Self {
            let mut _instance = Self {
                base_uri,
                supply_limit,
                mint_per_tx,
                mint_price,
                mint_fee,
                withdraw_wallet,
                dev_wallet,
                sale_time,
                id_lists: 0,
                ..Default::default()
            };

            ownable::Internal::_init_with_owner(&mut _instance, Self::env().caller());
            let _ = metadata::Internal::_set_attribute(
                &mut _instance,
                &id,
                &String::from("name"),
                &String::from(name),
            );

            let _ = metadata::Internal::_set_attribute(
                &mut _instance,
                &id,
                &String::from("symbol"),
                &String::from(symbol),
            );

            _instance
        }

        #[ink(message)]
        pub fn deny(&mut self, id: Id) {
            self.denied_ids.insert(&id, &());
        }

        #[ink(message)]
        pub fn mint_tokens(&mut self, id: Id, amount: Balance) -> Result<(), PSP37Error> {
            if self.denied_ids.get(&id).is_some() {
                return Err(PSP37Error::Custom(String::from("Id is denied")));
            }
            self.id_lists += 1;
            psp37::Internal::_mint_to(self, Self::env().caller(), vec![(id, amount)])
        }

        #[ink(message)]
        pub fn reserve(&mut self, reserve_datas: Vec<(Id, Balance)>) -> Result<(), PSP37Error> {
            if self.id_lists < self.supply_limit {
                for (id, amount) in reserve_datas.iter() {
                    let _ = self.mint_tokens(id.clone(), amount.clone());
                }
                Ok(())
            } else {
                return Err(PSP37Error::Custom(String::from(
                    "DropspaceSale::reserve: Supply limit reached",
                )));
            }
        }

        #[ink(message, payable)]
        pub fn buy(&mut self, buy_datas: Vec<(Id, Balance)>) -> Result<(), PSP37Error> {
            let amount = buy_datas.len() as u128;
            let total_price = amount.saturating_mul(self.mint_price.saturating_add(self.mint_fee));

            if self.env().block_timestamp() < self.sale_time {
                return Err(PSP37Error::Custom(String::from(
                    "DropspaceSale::buy: Sale hasn't started yet",
                )));
            }

            if self.id_lists.saturating_add(amount) > self.supply_limit {
                return Err(PSP37Error::Custom(String::from(
                    "DropspaceSale::buy: Supply limit reached",
                )));
            }

            if amount > self.mint_per_tx {
                return Err(PSP37Error::Custom(String::from(
                    "DropspaceSale::buy: Can't exceed amount of mints per tx",
                )));
            }

            if self.env().transferred_value() < total_price {
                return Err(PSP37Error::Custom(String::from(
                    "DropspaceSale::buy: Wrong amount paid.",
                )));
            }

            for (id, amount) in buy_datas.iter() {
                let _ = self.mint_tokens(id.clone(), amount.clone());
            }

            if let Some(withdraw_wallet) = self.withdraw_wallet {
                self.env()
                    .transfer(withdraw_wallet, amount.saturating_mul(self.mint_price))
                    .map_err(|_| {
                        PSP37Error::Custom(String::from("Transfer to owner wallet failed"))
                    })?;
            } else {
                return Err(PSP37Error::Custom(String::from("Owner wallet not set")));
            }

            if let Some(dev_wallet) = self.dev_wallet {
                if amount.saturating_mul(self.mint_fee) > 0 {
                    self.env()
                        .transfer(dev_wallet, amount.saturating_mul(self.mint_fee))
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
        #[modifiers(only_owner)]
        pub fn set_mint_per_tx(&mut self, mint_per_tx: u128) -> Result<(), PSP37Error> {
            self.mint_per_tx = mint_per_tx;
            Ok(())
        }

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
        pub fn set_supply_limit(&mut self, supply_limit: u128) -> Result<(), PSP37Error> {
            if self.id_lists > supply_limit {
                return Err(PSP37Error::Custom(String::from(
                    "DropspaceSale::set_total_supply: Supply limit is lesser than current supply",
                )));
            }
            self.supply_limit = supply_limit;
            Ok(())
        }

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
        pub fn supply_limit(&self) -> u128 {
            self.supply_limit
        }

        #[ink(message)]
        pub fn mint_per_tx(&self) -> u128 {
            self.mint_per_tx
        }

        #[ink(message)]
        pub fn get_account_balance(&self) -> u128 {
            self.env().balance()
        }

        #[ink(message)]
        pub fn get_id_lists(&self) -> u128 {
            self.id_lists
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

        #[ink(message)]
        #[modifiers(only_owner)]
        pub fn transfer_ownership(
            &mut self,
            new_owner: Option<AccountId>,
        ) -> Result<(), PSP37Error> {
            let _ = ownable::OwnableImpl::transfer_ownership(self, new_owner);
            Ok(())
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
    use openbrush::contracts::psp37::extensions::metadata::psp37metadata_external::PSP37Metadata;
    use openbrush::contracts::psp37::{psp37, Id, PSP37Error};
    use openbrush::traits::{Balance, StorageAsMut};
    use openbrush::{
        modifiers,
        storage::Mapping,
        traits::{Storage, String},
    };

    fn default_accounts() -> ink::env::test::DefaultAccounts<ink::env::DefaultEnvironment> {
        ink::env::test::default_accounts::<ink::env::DefaultEnvironment>()
    }

    struct ContractParam {
        name: String,
        symbol: String,
        base_uri: String,
        supply_limit: u128,
        mint_per_tx: u128,
        mint_price: u128,
        mint_fee: u128,
        withdraw_wallet: Option<AccountId>,
        dev_wallet: Option<AccountId>,
        sale_time: u64,
        id_lists: u128,
        denied_ids: Mapping<Id, ()>,
    }

    impl Default for ContractParam {
        fn default() -> ContractParam {
            ContractParam {
                name: "Test".to_string(),
                symbol: "TST".to_string(),
                base_uri: "https://example.com/token/".to_string(),
                supply_limit: 100000,
                mint_per_tx: 10,
                mint_price: 1000,
                mint_fee: 10,
                withdraw_wallet: None,
                dev_wallet: None,
                sale_time: 0,
                id_lists: 0,
                denied_ids: Mapping::default(),
            }
        }
    }
    fn get_contract(args: &ContractParam) -> Contract {
        return Contract::new(
            args.name.to_string(),
            args.symbol.to_string(),
            args.base_uri.to_string(),
            args.supply_limit,
            args.mint_per_tx,
            args.mint_price,
            args.mint_fee,
            args.withdraw_wallet,
            args.dev_wallet,
            Id::U8(0),
            0,
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

        assert_eq!(contract.supply_limit(), params.supply_limit);
        assert_eq!(contract.mint_per_tx(), params.mint_per_tx);
        assert_eq!(contract.mint_price(), params.mint_price);
        assert_eq!(contract.mint_fee(), params.mint_fee);
        assert_eq!(contract.dev_wallet(), params.dev_wallet);
        assert_eq!(contract.withdraw_wallet(), params.withdraw_wallet);
        assert_eq!(contract.sale_time(), params.sale_time);
        assert_eq!(contract.sale_active(), true);
        assert_eq!(
            PSP37Metadata::get_attribute(&contract, Id::U8(0), String::from("name")),
            Some(params.name)
        );
        assert_eq!(
            PSP37Metadata::get_attribute(&contract, Id::U8(0), String::from("symbol")),
            Some(params.symbol)
        );
        assert_eq!(contract.base_uri(), params.base_uri);
    }

    #[ink::test]
    fn reserve_works() {
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
            12345678,
        );

        assert_eq!(
            contract.reserve(vec![(Id::U8(1), 100), (Id::U8(2), 200)]),
            Ok(())
        );
    }

    #[ink::test]
    fn buy_works() {
        let accounts = default_accounts();

        // Set owner
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.charlie);

        let params = ContractParam {
            withdraw_wallet: Some(accounts.django),
            dev_wallet: Some(accounts.eve),
            ..Default::default()
        };
        let mut contract = get_contract(&params);

        // current owner checking
        assert_eq!(Ownable::owner(&contract), Some(accounts.charlie));
        ink::env::debug_println!(" Dev Fee={:?}", contract.mint_fee());
        ink::env::test::set_account_balance::<ink::env::DefaultEnvironment>(
            accounts.bob,
            100_000_000,
        );
        ink::env::test::set_account_balance::<ink::env::DefaultEnvironment>(accounts.django, 0);
        ink::env::test::set_account_balance::<ink::env::DefaultEnvironment>(accounts.eve, 0);
        ink::env::debug_println!(
            " Withdraw Wallet (django) bal={:?}",
            ink::env::test::get_account_balance::<ink::env::DefaultEnvironment>(accounts.django)
        );
        ink::env::debug_println!(
            " Dev Wallet (eve) bal={:?}",
            ink::env::test::get_account_balance::<ink::env::DefaultEnvironment>(accounts.eve)
        );

        let qty = vec![(Id::U8(1), 100), (Id::U8(2), 200)];
        let required_value = qty.len() as u128 * (params.mint_price + params.mint_fee);

        // Setting the caller for the next contract call
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);

        assert_eq!(
            ink::env::pay_with_call!(contract.buy(qty), required_value),
            Ok(())
        );

        println!("bobs account {}", contract.get_id_lists());
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