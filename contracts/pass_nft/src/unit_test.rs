#[cfg(test)]
mod tests {
    use cosmwasm_std::{
        from_json, Addr, coins, OwnedDeps,
        testing::{mock_dependencies, mock_env, mock_info, MockApi, MockQuerier, MockStorage},
    };

    use crate::{
        contract::{instantiate, execute, query},
        msg::{
            InstantiateMsg, ExecuteMsg, QueryMsg, PassMsg, PassQuery, 
            ConfigResponse, ValidityResponse, ArtistInfoResponse
        },
    };

    // Constants for testing
    const PASS_PRICE: u128 = 10;
    const PASS_DURATION: u64 = 1200; // 20 minutes
    const GRACE_PERIOD: u64 = 300;   // 5 minutes
    const USER: &str = "user";
    const ARTIST: &str = "artist";
    const PAYMENT_ADDR: &str = "payment_addr";
    const COLLECTION_NAME: &str = "Test Pass";
    const COLLECTION_SYMBOL: &str = "PASS";

    // Helper function to setup contract
    fn setup_contract() -> OwnedDeps<MockStorage, MockApi, MockQuerier> {
        let mut deps = mock_dependencies();
        let artist = Addr::unchecked(ARTIST);
        let payment_address = Addr::unchecked(PAYMENT_ADDR);

        let msg = InstantiateMsg {
            name: COLLECTION_NAME.to_string(),
            symbol: COLLECTION_SYMBOL.to_string(),
            artist: artist.to_string(),
            pass_price: PASS_PRICE,
            pass_duration: PASS_DURATION,
            grace_period: GRACE_PERIOD,
            payment_address: payment_address,
        };

        let info = mock_info(ARTIST, &[]);
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        deps
    }

    // Helper function to format token ID
    fn format_token_id(token_id: &str) -> String {
        format!("{}-{}", COLLECTION_SYMBOL.to_lowercase(), token_id)
    }

    #[test]
    fn test_initialization() {
        let mut deps = mock_dependencies();
        let artist = Addr::unchecked(ARTIST);
        let payment_address = Addr::unchecked(PAYMENT_ADDR);

        let msg = InstantiateMsg {
            name: COLLECTION_NAME.to_string(),
            symbol: COLLECTION_SYMBOL.to_string(),
            artist: artist.to_string(),
            pass_price: PASS_PRICE,
            pass_duration: PASS_DURATION,
            grace_period: GRACE_PERIOD,
            payment_address: payment_address.clone(),
        };

        let info = mock_info(ARTIST, &[]);
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
        
        let res = query(deps.as_ref(), mock_env(), QueryMsg::Extension { 
            msg: PassQuery::GetConfig {} 
        }).unwrap();
        let config: ConfigResponse = from_json(&res).unwrap();
        
        assert_eq!(config.name, COLLECTION_NAME);
        assert_eq!(config.symbol, COLLECTION_SYMBOL);
        assert_eq!(config.artist, artist.to_string());
        assert_eq!(config.pass_price, PASS_PRICE);
        assert_eq!(config.pass_duration, PASS_DURATION);
        assert_eq!(config.grace_period, GRACE_PERIOD);
        assert_eq!(config.payment_address, payment_address.to_string());
    }

    #[test]
    fn test_mint_pass() {
        let mut deps = setup_contract();
        let token_id = format_token_id("pass1");

        // Try minting without payment
        let info = mock_info(USER, &[]);
        let msg = ExecuteMsg::Extension { 
            msg: PassMsg::MintPass { 
                token_id: "pass1".to_string()  // Send unformatted token_id
            } 
        };
        let err = execute(deps.as_mut(), mock_env(), info, msg.clone()).unwrap_err();
        assert_eq!(err.to_string(), "No uxion payment found");

        // Mint with correct payment
        let info = mock_info(USER, &coins(PASS_PRICE, "uxion"));
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert!(res.attributes.iter().any(|attr| attr.key == "action" && attr.value == "mint_pass"));

        // Query pass validity
        let msg = QueryMsg::Extension { 
            msg: PassQuery::CheckValidity { 
                token_id: token_id.clone()
            } 
        };
        let res = query(deps.as_ref(), mock_env(), msg).unwrap();
        let validity: ValidityResponse = from_json(&res).unwrap();
        
        assert!(validity.is_valid);
        assert!(!validity.in_grace_period);
    }

    #[test]
    fn test_renew_pass() {
        let mut deps = setup_contract();
        let base_token_id = "pass1";
        let token_id = format_token_id(base_token_id);

        // First mint a pass
        let info = mock_info(USER, &coins(PASS_PRICE, "uxion"));
        let mint_msg = ExecuteMsg::Extension { 
            msg: PassMsg::MintPass { 
                token_id: base_token_id.to_string()
            } 
        };
        execute(deps.as_mut(), mock_env(), info, mint_msg).unwrap();

        // Renew with correct payment
        let info = mock_info(USER, &coins(PASS_PRICE, "uxion"));
        let msg = ExecuteMsg::Extension { 
            msg: PassMsg::RenewPass { 
                token_id: token_id.clone()
            } 
        };
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert!(res.attributes.iter().any(|attr| attr.key == "action" && attr.value == "renew_pass"));
    }

    #[test]
    fn test_burn_expired_pass() {
        let mut deps = setup_contract();
        let base_token_id = "pass1";
        let token_id = format_token_id(base_token_id);

        // Mint a pass
        let info = mock_info(USER, &coins(PASS_PRICE, "uxion"));
        let mint_msg = ExecuteMsg::Extension { 
            msg: PassMsg::MintPass { 
                token_id: base_token_id.to_string()
            } 
        };
        execute(deps.as_mut(), mock_env(), info.clone(), mint_msg).unwrap();

        // Move time past expiration and grace period
        let mut env = mock_env();
        env.block.time = env.block.time.plus_seconds(PASS_DURATION + GRACE_PERIOD + 1);

        // Burn expired pass
        let burn_msg = ExecuteMsg::Extension { 
            msg: PassMsg::BurnExpiredPass { 
                token_id: token_id.clone()
            } 
        };
        let res = execute(deps.as_mut(), env.clone(), info, burn_msg).unwrap();
        assert!(res.attributes.iter().any(|attr| attr.key == "action" && attr.value == "burn_expired_pass"));
    }

    #[test]
    fn test_artist_info() {
        let mut deps = setup_contract();
        
        // Query artist info before any mints
        let msg = QueryMsg::Extension { 
            msg: PassQuery::GetArtistInfo {} 
        };
        let res = query(deps.as_ref(), mock_env(), msg.clone()).unwrap();
        let info = from_json::<ArtistInfoResponse>(&res).unwrap();
        assert_eq!(info.artist, ARTIST);
        assert_eq!(info.total_passes, 0);
        assert_eq!(info.active_passes, 0);

        // Mint a pass
        let info = mock_info(USER, &coins(PASS_PRICE, "uxion"));
        let mint_msg = ExecuteMsg::Extension { 
            msg: PassMsg::MintPass { 
                token_id: "pass1".to_string()
            } 
        };
        execute(deps.as_mut(), mock_env(), info, mint_msg).unwrap();

        // Query updated artist info
        let res = query(deps.as_ref(), mock_env(), msg).unwrap();
        let info = from_json::<ArtistInfoResponse>(&res).unwrap();
        assert_eq!(info.artist, ARTIST);
        assert_eq!(info.total_passes, 1);
        assert_eq!(info.active_passes, 1);
    }
}