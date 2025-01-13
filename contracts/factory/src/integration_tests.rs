#[cfg(test)]
mod tests {
    use crate::msg::{InstantiateMsg, ExecuteMsg, QueryMsg, ConfigResponse, CollectionResponse};
    use cosmwasm_std::{Addr, Coin, Empty};
    use cw_multi_test::{App, Contract, ContractWrapper, Executor};

    // Contract constructors
    pub fn factory_contract() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        )
        .with_reply(crate::contract::reply);
        Box::new(contract)
    }

    pub fn nft_contract() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            pass_nft::contract::execute,
            pass_nft::contract::instantiate,
            pass_nft::contract::query,
        );
        Box::new(contract)
    }

    // Helper to setup contracts
    fn setup_contracts() -> (App, Addr, Addr, Addr, Addr, Addr, u64) {
        let mut app = App::default();
        let owner = Addr::unchecked("owner");  // Contract owner & payment receiver
        let artist = Addr::unchecked("artist");
        let user1 = Addr::unchecked("user1");
        let user2 = Addr::unchecked("user2");

        // Initialize balances for testing
        app.init_modules(|router, _api, storage| {
            router
                .bank
                .init_balance(
                    storage,
                    &user1,
                    vec![Coin::new(1000u128, "uxion")]
                )
                .unwrap();
            router
                .bank
                .init_balance(
                    storage,
                    &user2,
                    vec![Coin::new(1000u128, "uxion")]
                )
                .unwrap();
        });

        // Store NFT contract code
        let nft_code_id = app.store_code(nft_contract());

        // Store and instantiate factory contract
        let factory_code_id = app.store_code(factory_contract());
        let factory_addr = app
            .instantiate_contract(
                factory_code_id,
                owner.clone(),
                &InstantiateMsg {
                    nft_code_id,
                },
                &[],
                "music-pass-factory",
                None,
            )
            .unwrap();

        (app, factory_addr, owner, artist, user1, user2, nft_code_id)
    }

    #[test]
    fn proper_initialization() {
        let (app, factory_addr, owner, _artist, _, _, nft_code_id) = setup_contracts();

        let config: ConfigResponse = app
            .wrap()
            .query_wasm_smart(&factory_addr, &QueryMsg::Config {})
            .unwrap();

        assert_eq!(config.nft_code_id, nft_code_id);
        assert_eq!(config.owner, owner.to_string());
        assert_eq!(config.total_collections, 0);
    }

    #[test]
    fn create_collection() {
        let (mut app, factory_addr, owner, artist, _, _, _) = setup_contracts();

        // Create collection
        let msg = ExecuteMsg::CreateCollection {
            name: String::from("Drake Collection"),
            symbol: String::from("DRAKE"),
            pass_price: 10u128,
            pass_duration: 2592000u64,  // 30 days
            grace_period: 259200u64,    // 3 days
            payment_address: owner.clone(),
        };

        // Artist creates collection
        let res = app.execute_contract(
            artist.clone(),
            factory_addr.clone(),
            &msg,
            &[],
        );
        assert!(res.is_ok());

        // Query the collection
        let res: CollectionResponse = app
            .wrap()
            .query_wasm_smart(
                &factory_addr,
                &QueryMsg::Collection {
                    artist: artist.to_string(),
                },
            )
            .unwrap();

        let collection = res.collection.unwrap();
        assert_eq!(collection.name, "Drake Collection");
        assert_eq!(collection.symbol, "DRAKE");
        assert_eq!(collection.artist, artist);
        assert!(!collection.contract_address.to_string().is_empty());
    }

    #[test]
    fn create_multiple_collections() {
        let (mut app, factory_addr, owner, artist, _, _, _) = setup_contracts();

        // Create first collection
        let msg1 = ExecuteMsg::CreateCollection {
            name: String::from("Drake Collection"),
            symbol: String::from("DRAKE"),
            pass_price: 10u128,
            pass_duration: 2592000u64,
            grace_period: 259200u64,
            payment_address: owner.clone(),
        };

        let res = app.execute_contract(
            artist.clone(),
            factory_addr.clone(),
            &msg1,
            &[],
        );
        assert!(res.is_ok());

        // Try to create collection with same symbol (should fail)
        let msg2 = ExecuteMsg::CreateCollection {
            name: String::from("Different Collection"),
            symbol: String::from("DRAKE"),
            pass_price: 20u128,
            pass_duration: 1296000u64,
            grace_period: 129600u64,
            payment_address: owner.clone(),
        };

        let res = app.execute_contract(
            artist.clone(),
            factory_addr.clone(),
            &msg2,
            &[],
        );
        assert!(res.is_err());

        // Create collection with different symbol
        let msg3 = ExecuteMsg::CreateCollection {
            name: String::from("Different Collection"),
            symbol: String::from("DIFF"),
            pass_price: 20u128,
            pass_duration: 1296000u64,
            grace_period: 129600u64,
            payment_address: owner.clone()
        };

        let res = app.execute_contract(
            artist.clone(),
            factory_addr.clone(),
            &msg3,
            &[],
        );
        assert!(res.is_ok());
    }

    #[test]
    fn test_invalid_symbol() {
        let (mut app, factory_addr, owner, artist, _, _, _) = setup_contracts();

        // Try with lowercase symbol
        let msg = ExecuteMsg::CreateCollection {
            name: String::from("Test Collection"),
            symbol: String::from("drake"),
            pass_price: 10u128,
            pass_duration: 2592000u64,
            grace_period: 259200u64,
            payment_address: owner.clone()
        };

        let res = app.execute_contract(
            artist.clone(),
            factory_addr.clone(),
            &msg,
            &[],
        );
        assert!(res.is_err());

        // Try with symbol containing spaces
        let msg = ExecuteMsg::CreateCollection {
            name: String::from("Test Collection"),
            symbol: String::from("DRAKE TEST"),
            pass_price: 10u128,
            pass_duration: 2592000u64,
            grace_period: 259200u64,
            payment_address: owner.clone()
        };

        let res = app.execute_contract(
            artist.clone(),
            factory_addr.clone(),
            &msg,
            &[],
        );
        assert!(res.is_err());
    }
}