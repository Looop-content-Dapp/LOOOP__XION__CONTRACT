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
            ConfigResponse, ValidityResponse, ArtistInfoResponse, PassResponse
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
    const COLLECTION_SYMBOL: &str = "TEST";

    // Helper function to setup contract
    fn setup_contract() -> OwnedDeps<MockStorage, MockApi, MockQuerier> {
        let mut deps = mock_dependencies();
        let artist = Addr::unchecked(ARTIST);
        let minter = Addr::unchecked(USER);
        let payment_address = Addr::unchecked(PAYMENT_ADDR);
        let collection_info = "Test Collection".to_string();

        let msg = InstantiateMsg {
            name: COLLECTION_NAME.to_string(),
            symbol: COLLECTION_SYMBOL.to_string(),
            artist,
            minter, 
            collection_info,
            pass_price: PASS_PRICE,
            pass_duration: PASS_DURATION,
            grace_period: GRACE_PERIOD,
            payment_address,
        };

        let info = mock_info(ARTIST, &[]);
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        deps
    }

    #[test]
    fn test_initialization() {
        let mut deps = mock_dependencies();
        let artist = Addr::unchecked(ARTIST);
        let minter = Addr::unchecked(USER);
        let payment_address = Addr::unchecked(PAYMENT_ADDR);
        let collection_info = "Test Collection".to_string();

        let msg = InstantiateMsg {
            name: COLLECTION_NAME.to_string(),
            symbol: COLLECTION_SYMBOL.to_string(),
            artist,
            minter: minter.clone(),
            collection_info,
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
        assert_eq!(config.pass_price, PASS_PRICE);
        assert_eq!(config.pass_duration, PASS_DURATION);
        assert_eq!(config.grace_period, GRACE_PERIOD);
        assert_eq!(config.payment_address, payment_address);
    }

    #[test]
    fn test_mint_pass() {
        let mut deps = setup_contract();
        
        // Try minting without payment
        let info = mock_info(USER, &[]);
        let msg = ExecuteMsg::Extension { 
            msg: PassMsg::MintPass { 
                owner_address: USER.to_string(),
            } 
        };
        let err = execute(deps.as_mut(), mock_env(), info, msg.clone()).unwrap_err();
        assert_eq!(err.to_string(), "No uxion payment found");

        // Mint with correct payment
        let info = mock_info(USER, &coins(PASS_PRICE, "uxion"));
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert!(res.attributes.iter().any(|attr| attr.key == "action" && attr.value == "mint_pass"));

        // Get token_id from response attributes
        let token_id = res.attributes
            .iter()
            .find(|attr| attr.key == "token_id")
            .map(|attr| attr.value.clone())
            .unwrap();

        // Query pass validity
        let msg = QueryMsg::Extension { 
            msg: PassQuery::CheckValidity { token_id: token_id.clone() } 
        };
        let res = query(deps.as_ref(), mock_env(), msg).unwrap();
        let validity: ValidityResponse = from_json(&res).unwrap();
        
        assert!(validity.is_valid);
        assert!(!validity.in_grace_period);

        // Test GetUserPass query
        let msg = QueryMsg::Extension {
            msg: PassQuery::GetUserPass {
                symbol: COLLECTION_SYMBOL.to_string(),
                owner: USER.to_string(),
            }
        };
        let res = query(deps.as_ref(), mock_env(), msg).unwrap();
        let pass_info: PassResponse = from_json(&res).unwrap();

        println!("Pass Info: {:?}", pass_info);
        assert_eq!(pass_info.token_id, token_id);
        assert_eq!(pass_info.collection_name, COLLECTION_NAME);
        assert!(pass_info.is_valid);
    }

    #[test]
    fn test_renew_pass() {
        let mut deps = setup_contract();

        // First mint a pass
        let info = mock_info(USER, &coins(PASS_PRICE, "uxion"));
        let mint_msg = ExecuteMsg::Extension { 
            msg: PassMsg::MintPass { 
                owner_address: USER.to_string(),
            } 
        };
        let mint_res = execute(deps.as_mut(), mock_env(), info, mint_msg).unwrap();
        
        // Get token_id from mint response
        let token_id = mint_res.attributes
            .iter()
            .find(|attr| attr.key == "token_id")
            .map(|attr| attr.value.clone())
            .unwrap();

        // Renew with correct payment
        let info = mock_info(USER, &coins(PASS_PRICE, "uxion"));
        let msg = ExecuteMsg::Extension { 
            msg: PassMsg::RenewPass { token_id: token_id.clone() } 
        };
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert!(res.attributes.iter().any(|attr| attr.key == "action" && attr.value == "renew_pass"));
    }

    #[test]
    fn test_burn_expired_pass() {
        let mut deps = setup_contract();

        // Mint a pass
        let info = mock_info(USER, &coins(PASS_PRICE, "uxion"));
        let mint_msg = ExecuteMsg::Extension { 
            msg: PassMsg::MintPass { 
                owner_address: USER.to_string(),
            } 
        };
        let mint_res = execute(deps.as_mut(), mock_env(), info.clone(), mint_msg).unwrap();
        
        // Get token_id from mint response
        let token_id = mint_res.attributes
            .iter()
            .find(|attr| attr.key == "token_id")
            .map(|attr| attr.value.clone())
            .unwrap();

        // Move time past expiration and grace period
        let mut env = mock_env();
        env.block.time = env.block.time.plus_seconds(PASS_DURATION + GRACE_PERIOD + 1);

        // Burn expired pass
        let burn_msg = ExecuteMsg::Extension { 
            msg: PassMsg::BurnExpiredPass { token_id } 
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
                owner_address: USER.to_string(),
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