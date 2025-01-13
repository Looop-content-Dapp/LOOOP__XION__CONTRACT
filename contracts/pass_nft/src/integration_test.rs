#[cfg(test)]
mod tests {
    use cosmwasm_std::{Coin, Empty, Addr};
    use cw_multi_test::{App, Contract, ContractWrapper, Executor};
    use crate::contract::{execute, instantiate, query};
    use crate::msg::{InstantiateMsg, ExecuteMsg, QueryMsg, PassMsg, ValidityResponse, ConfigResponse};
    use crate::msg::PassQuery;

    const PASS_PRICE: u128 = 10;
    const PASS_DURATION: u64 = 1200; // 20 minutes
    const GRACE_PERIOD: u64 = 300;   // 5 minutes

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
                    symbol: "PASS".to_string(),
                    artist: artist.to_string(),
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
        let mint_actions = vec![
            (&user1, "pass1"),
            (&user2, "pass2"),
        ];

        for (user, token_id) in mint_actions {
            let token_id = format!("pass-{}", token_id);
            let msg = ExecuteMsg::Extension { 
                msg: PassMsg::MintPass { 
                    token_id: token_id.clone()
                }
            };
            
            app.execute_contract(
                user.clone(),
                contract_addr.clone(),
                &msg,
                &[Coin::new(PASS_PRICE, "uxion")],
            )
            .unwrap();
            println!("{} minted pass {}", user, token_id);

            // Query pass validity after minting
            let validity: ValidityResponse = app
                .wrap()
                .query_wasm_smart(
                    contract_addr.clone(),
                    &QueryMsg::Extension { 
                        msg: PassQuery::CheckValidity { 
                            token_id: format!("{}-{}", config.symbol.to_lowercase(), token_id)
                        }
                    },
                )
                .unwrap();
            println!("Pass {} validity: {:?}", token_id, validity);
        }

        // Test pass renewal
        println!("\n=== Testing Pass Renewal ===");
        let token_id = format!("{}-{}", config.symbol.to_lowercase(), "pass-pass1");
        let msg = ExecuteMsg::Extension { 
            msg: PassMsg::RenewPass { 
                token_id: token_id.clone()
            }
        };
        
        app.execute_contract(
            user1.clone(),
            contract_addr.clone(),
            &msg,
            &[Coin::new(PASS_PRICE, "uxion")],
        )
        .unwrap();
        println!("User1 renewed {}", token_id);

        // Test burning expired pass
        println!("\n=== Testing Pass Burning ===");
        // Move time forward past expiration and grace period
        app.update_block(|block| {
            block.time = block.time.plus_seconds(PASS_DURATION + GRACE_PERIOD + 1);
        });

        let token_id = format!("{}-{}", config.symbol.to_lowercase(), "pass-pass2");
        let msg = ExecuteMsg::Extension { 
            msg: PassMsg::BurnExpiredPass { 
                token_id: token_id.clone()
            }
        };
        
        app.execute_contract(
            user2.clone(),
            contract_addr.clone(),
            &msg,
            &[],
        )
        .unwrap();
        println!("User2 burned {}", token_id);

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