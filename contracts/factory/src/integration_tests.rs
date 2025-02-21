#[cfg(test)]
mod tests {
    use crate::msg::{InstantiateMsg, ExecuteMsg, QueryMsg, CollectionResponse};
    use cosmwasm_std::{Addr, Empty};
    use cw_multi_test::{App, Contract, ContractWrapper, Executor};

    // Add NFT contract constructor
    pub fn nft_contract() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            pass_nft::contract::execute,
            pass_nft::contract::instantiate,
            pass_nft::contract::query,
        );
        Box::new(contract)
    }

    pub fn factory_contract() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        ).with_reply(crate::contract::reply);
        Box::new(contract)
    }

    fn setup_contracts() -> (App, Addr, Addr, Addr, Addr) {
        let mut app = App::default();
        let admin = Addr::unchecked("admin");
        let artist = Addr::unchecked("artist");
        let minter = Addr::unchecked("minter");
        let payment_addr = Addr::unchecked("payment");
        let house_percentage: u32 = 30;
        let artist_percentage: u32 = 70;

        // First store the NFT contract code
        let nft_code_id = app.store_code(nft_contract());

        // Then store and instantiate the factory
        let factory_code_id = app.store_code(factory_contract());
        let factory_addr = app
            .instantiate_contract(
                factory_code_id,
                admin.clone(),
                &InstantiateMsg {
                    nft_code_id,
                    price: 100u128,
                    duration: 2592000,
                    grace_period: 259200,
                    payment_address: payment_addr.clone(),
                    artist_percentage,
                    house_percentage
                },
                &[],
                "factory",
                None,
            )
            .unwrap();

        println!("Contract setup completed: factory at {}", factory_addr);
        (app, factory_addr, admin, artist, minter)
    }

    #[test]
fn test_create_collection() {
    let (mut app, factory_addr, admin, artist, minter) = setup_contracts();

    let symbol = "TEST".to_string();

    // Notice the symbol is now "TEST" instead of "test"
    let msg = ExecuteMsg::CreateCollection {
        name: "Test Collection".to_string(),
        symbol,
        artist: artist.clone(),
        minter: minter.clone(),
        collection_info: "Test Collection Metadata".to_string(),
    };

    println!("Executing create collection with msg: {:?}", msg);
    
    let res = app.execute_contract(
        admin.clone(),
        factory_addr.clone(),
        &msg,
        &[]
    );
    println!("Create collection result: {:?}", res);
    assert!(res.is_ok());

    // After successful creation, verify the collection
    let query_res: CollectionResponse = app
        .wrap()
        .query_wasm_smart(
            &factory_addr,
            &QueryMsg::Collection { 
                artist: artist.to_string() 
            }
        )
        .unwrap();

    println!("Collection query result: {:?}", query_res);
    
    // Verify the collection details
    let collection = query_res.collection.unwrap();
    assert_eq!(collection.symbol, "TEST");
    assert_eq!(collection.name, "Test Collection");
    assert_eq!(collection.artist, artist);
    assert_eq!(collection.minter, minter);
}

// Let's also add a test specifically for symbol validation
#[test]
fn test_symbol_validation() {
    let (mut app, factory_addr, admin, artist, minter) = setup_contracts();

    // Test cases for invalid symbols
    let invalid_symbols = vec![
        "test",      // lowercase
        "Test",      // mixed case
        "TEST ",     // with space
        "TEST-1",    // with special character
    ];

    for symbol in invalid_symbols {
        let msg = ExecuteMsg::CreateCollection {
            name: "Test Collection".to_string(),
            symbol: symbol.to_string(),
            artist: artist.clone(),
            minter: minter.clone(),
            collection_info: "Test Collection Metadata".to_string(),
        };

        let res = app.execute_contract(
            admin.clone(),
            factory_addr.clone(),
            &msg,
            &[]
        );
        
        println!("Testing invalid symbol '{}': {:?}", symbol, res);
        assert!(res.is_err(), "Symbol '{}' should be invalid", symbol);
    }
}

}