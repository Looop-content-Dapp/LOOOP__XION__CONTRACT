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
                    payment_address: payment_addr,
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
        for user in [&user1, &user2] {
            let balance = app
                .wrap()
                .query_balance(user.clone(), "uxion")
                .unwrap();
            println!("Balance of {}: {}", user, balance.amount);
        }

        // Mint passes
        println!("\n=== Minting Passes ===");
        let mint_users = vec![&user1, &user2];

        for user in mint_users {
            let msg = ExecuteMsg::Extension { 
                msg: PassMsg::MintPass { 
                    owner_address: user.to_string()
                }
            };
            
            let res = app.execute_contract(
                user.clone(),
                contract_addr.clone(),
                &msg,
                &[Coin::new(PASS_PRICE, "uxion")],
            )
            .unwrap();

            // Get token_id from response attributes
            let token_id = res.events
                .iter()
                .flat_map(|e| &e.attributes)
                .find(|attr| attr.key == "token_id")
                .map(|attr| attr.value.clone())
                .unwrap();

            println!("{} minted pass {}", user, token_id);

            // Query pass details after minting
            let pass_details: PassResponse = app
                .wrap()
                .query_wasm_smart(
                    contract_addr.clone(),
                    &QueryMsg::Extension { 
                        msg: PassQuery::GetUserPass { 
                            symbol: COLLECTION_SYMBOL.to_string(),
                            owner: user.to_string()
                        }
                    },
                )
                .unwrap();
            println!("Pass details for {}: {:?}", user, pass_details);

            // Also check validity
            let validity: ValidityResponse = app
                .wrap()
                .query_wasm_smart(
                    contract_addr.clone(),
                    &QueryMsg::Extension { 
                        msg: PassQuery::CheckValidity { 
                            token_id: token_id.clone()
                        }
                    },
                )
                .unwrap();
            println!("Pass {} validity: {:?}", token_id, validity);
        }

        // Test pass renewal
        println!("\n=== Testing Pass Renewal ===");
        // Query user1's pass first
        let user1_pass: PassResponse = app
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

        let msg = ExecuteMsg::Extension { 
            msg: PassMsg::RenewPass { 
                token_id: user1_pass.token_id.clone()
            }
        };
        
        app.execute_contract(
            user1.clone(),
            contract_addr.clone(),
            &msg,
            &[Coin::new(PASS_PRICE, "uxion")],
        )
        .unwrap();
        println!("User1 renewed pass {}", user1_pass.token_id);

        // Test burning expired pass
        println!("\n=== Testing Pass Burning ===");
        // Move time forward past expiration and grace period
        app.update_block(|block| {
            block.time = block.time.plus_seconds(PASS_DURATION + GRACE_PERIOD + 1);
        });

        // Get user2's pass
        let user2_pass: PassResponse = app
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

        let msg = ExecuteMsg::Extension { 
            msg: PassMsg::BurnExpiredPass { 
                token_id: user2_pass.token_id.clone()
            }
        };
        
        app.execute_contract(
            user2.clone(),
            contract_addr.clone(),
            &msg,
            &[],
        )
        .unwrap();
        println!("User2 burned pass {}", user2_pass.token_id);

        // Check final balances
        println!("\n=== Final Balances ===");
        for user in [&user1, &user2] {
            let balance = app
                .wrap()
                .query_balance(user.clone(), "uxion")
                .unwrap();
            println!("Final balance of {}: {}", user, balance.amount);
        }
    }
}