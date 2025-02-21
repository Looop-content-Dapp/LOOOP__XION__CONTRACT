#[cfg(test)]
mod tests {
    use cosmwasm_std::{Coin, Empty, Addr};
    use cw_multi_test::{App, Contract, ContractWrapper, Executor};
    use crate::contract::{execute, instantiate, query};
    use crate::msg::{InstantiateMsg, ExecuteMsg, QueryMsg, PassMsg, ValidityResponse, ConfigResponse, PassResponse};
    use crate::msg::PassQuery;

    const PASS_PRICE: u128 = 10;
    const PASS_DURATION: u64 = 1200; // 20 minutes
    const GRACE_PERIOD: u64 = 300;   // 5 minutes
    const COLLECTION_SYMBOL: &str = "TEST";
    const HOUSE_ROYALTY: u32 = 30;
    const ARTIST_ROYALTY : u32 = 70; 

    fn contract_pass() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new_with_empty(execute, instantiate, query);
        Box::new(contract)
    }

    #[test]
    fn test_pass_flow() {
        // Setup test accounts
        let mut app = App::default();
        let owner = Addr::unchecked("owner");
        let user1 = Addr::unchecked("user1");
        let user2 = Addr::unchecked("user2");
        let artist = Addr::unchecked("artist");
        let payment_addr = Addr::unchecked("payment_addr");
        let collection_info = "Test Collection".to_string();

        // Initialize balances
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

        println!("\n=== Starting Pass Flow Test ===");
        println!("Setting up users:");
        println!("Owner: {}", owner);
        println!("Users: {}, {}", user1, user2);
        println!("Payment Address: {}", payment_addr);
        // println!("Artist Bal: {}", artist);

        // Upload and instantiate contract
        let contract_id = app.store_code(contract_pass());
        let contract_addr = app
            .instantiate_contract(
                contract_id,
                owner.clone(),
                &InstantiateMsg {
                    name: "Test Pass".to_string(),
                    symbol: COLLECTION_SYMBOL.to_string(),
                    artist: artist.clone(),
                    minter: user1.clone(),
                    collection_info,
                    pass_price: PASS_PRICE,
                    pass_duration: PASS_DURATION,
                    grace_period: GRACE_PERIOD,
                    payment_address: payment_addr.clone(),
                    artist_percentage: ARTIST_ROYALTY,
                    house_percentage: HOUSE_ROYALTY
                },
                &[],
                "music-pass",
                None,
            )
            .unwrap();

        // Query initial config
        let config: ConfigResponse = app
            .wrap()
            .query_wasm_smart(contract_addr.clone(), &QueryMsg::Extension { msg: PassQuery::GetConfig {} })
            .unwrap();
        
        println!("\n=== Initial Contract Config ===");
        println!("Pass Price: {}", config.pass_price);
        println!("Pass Duration: {}", config.pass_duration);
        println!("Grace Period: {}", config.grace_period);

        // Print initial balances
        println!("\n=== Initial Balances ===");
        for user in [&user1, &user2, &artist, &payment_addr] {
            let balance = app
                .wrap()
                .query_balance(user.clone(), "uxion")
                .unwrap();
            println!("Balance of {}: {}", user, balance.amount);
        }

        // Mint passes
        println!("\n=== Minting Passes ===");

        // User 1 mints first
        println!("Attempting first mint for user1...");
        let mint_msg_1 = ExecuteMsg::Extension { 
            msg: PassMsg::MintPass { 
                owner_address: user1.to_string()
            }
        };

        let res1 = app.execute_contract(
            user1.clone(),
            contract_addr.clone(),
            &mint_msg_1,
            &[Coin::new(PASS_PRICE, "uxion")],
        ).unwrap();

        // Get token_id from response attributes for first mint
        let token_id_1 = res1.events
            .iter()
            .flat_map(|e| &e.attributes)
            .find(|attr| attr.key == "token_id")
            .map(|attr| attr.value.clone())
            .unwrap();

        println!("User1 minted pass {}", token_id_1);

        // Query first pass details
        let pass_details_1: PassResponse = app
            .wrap()
            .query_wasm_smart(
                contract_addr.clone(),
                &QueryMsg::Extension { 
                    msg: PassQuery::GetUserPass { 
                        symbol: COLLECTION_SYMBOL.to_string(),
                        owner: user1.to_string()
                    }
                },
            )
            .unwrap();
        println!("Pass details for user1: {:?}", pass_details_1);

        // Check validity for first pass
        let validity_1: ValidityResponse = app
            .wrap()
            .query_wasm_smart(
                contract_addr.clone(),
                &QueryMsg::Extension { 
                    msg: PassQuery::CheckValidity { 
                        token_id: token_id_1.clone()
                    }
                },
            )
            .unwrap();
        println!("Pass {} validity: {:?}", token_id_1, validity_1);

        // User 2 mints second
        println!("\nAttempting second mint for user2...");
        let mint_msg_2 = ExecuteMsg::Extension { 
            msg: PassMsg::MintPass { 
                owner_address: user2.to_string()
            }
        };

        let res2 = app.execute_contract(
            user2.clone(),
            contract_addr.clone(),
            &mint_msg_2,
            &[Coin::new(PASS_PRICE, "uxion")],
        ).unwrap();

        // Get token_id from response attributes for second mint
        let token_id_2 = res2.events
            .iter()
            .flat_map(|e| &e.attributes)
            .find(|attr| attr.key == "token_id")
            .map(|attr| attr.value.clone())
            .unwrap();

        println!("User2 minted pass {}", token_id_2);

        // Query second pass details
        let pass_details_2: PassResponse = app
            .wrap()
            .query_wasm_smart(
                contract_addr.clone(),
                &QueryMsg::Extension { 
                    msg: PassQuery::GetUserPass { 
                        symbol: COLLECTION_SYMBOL.to_string(),
                        owner: user2.to_string()
                    }
                },
            )
            .unwrap();
        println!("Pass details for user2: {:?}", pass_details_2);

        // Check validity for second pass
        let validity_2: ValidityResponse = app
            .wrap()
            .query_wasm_smart(
                contract_addr.clone(),
                &QueryMsg::Extension { 
                    msg: PassQuery::CheckValidity { 
                        token_id: token_id_2.clone()
                    }
                },
            )
            .unwrap();
        println!("Pass {} validity: {:?}", token_id_2, validity_2);

        // Test pass renewal
        println!("\n=== Testing Pass Renewal ===");
        let msg = ExecuteMsg::Extension { 
            msg: PassMsg::RenewPass { 
                token_id: token_id_1.clone()
            }
        };
        
        app.execute_contract(
            user1.clone(),
            contract_addr.clone(),
            &msg,
            &[Coin::new(PASS_PRICE, "uxion")],
        )
        .unwrap();
        println!("User1 renewed pass {}", token_id_1);

        // Test burning expired pass
        println!("\n=== Testing Pass Burning ===");
        // Move time forward past expiration and grace period
        app.update_block(|block| {
            block.time = block.time.plus_seconds(PASS_DURATION + GRACE_PERIOD + 1);
        });

        let msg = ExecuteMsg::Extension { 
            msg: PassMsg::BurnExpiredPass { 
                token_id: token_id_2.clone()
            }
        };
        
        app.execute_contract(
            user2.clone(),
            contract_addr.clone(),
            &msg,
            &[],
        )
        .unwrap();
        println!("User2 burned pass {}", token_id_2);

        // Check final balances
        // Print initial balances
        println!("\n=== Final Balances ===");
        for user in [&user1, &user2, &artist, &payment_addr] {
            let balance = app
                .wrap()
                .query_balance(user.clone(), "uxion")
                .unwrap();
            println!("Balance of {}: {}", user, balance.amount);
        }
    }
}