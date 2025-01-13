#[cfg(test)]
mod tests {
    use cosmwasm_std::{
        from_json, testing::{mock_dependencies, mock_env, mock_info},
        Addr, Response, DepsMut,
    };
    use crate::{
        msg::{InstantiateMsg, ExecuteMsg, QueryMsg, ConfigResponse, CollectionResponse},
        error::ContractError,
        contract::{instantiate, query, execute},
    };

    // Constants for testing
    const OWNER: &str = "owner";
    const ARTIST: &str = "artist";
    const NFT_CODE_ID: u64 = 123;
    const PASS_PRICE: u128 = 10;
    const PASS_DURATION: u64 = 2592000;   // 30 days
    const GRACE_PERIOD: u64 = 259200;     // 3 days
    const COLLECTION_NAME: &str = "Drake Collection";
    const COLLECTION_SYMBOL: &str = "DRAKE";

    // Helper function to instantiate the contract
    fn setup_contract(deps: DepsMut) -> Response {
        let msg = InstantiateMsg {
            nft_code_id: NFT_CODE_ID,
        };
        let info = mock_info(OWNER, &[]);
        let env = mock_env();

        instantiate(deps, env, info, msg).unwrap()
    }

    // Helper function to create a collection message
    fn create_collection_msg(name: String, symbol: String) -> ExecuteMsg {
        ExecuteMsg::CreateCollection {
            name,
            symbol,
            pass_price: PASS_PRICE,
            pass_duration: PASS_DURATION,
            grace_period: GRACE_PERIOD,
            payment_address: Addr::unchecked(OWNER),
        }
    }

    mod factory_tests {
        use super::*;

        #[test]
        fn proper_initialization() {
            let mut deps = mock_dependencies();
            let response = setup_contract(deps.as_mut());
            assert_eq!(0, response.messages.len());

            // Query config
            let config: ConfigResponse = from_json(
                &query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap()
            ).unwrap();

            assert_eq!(config.nft_code_id, NFT_CODE_ID);
            assert_eq!(config.owner, OWNER);
            assert_eq!(config.total_collections, 0);
        }

        #[test]
        fn test_create_collection() {
            let mut deps = mock_dependencies();
            setup_contract(deps.as_mut());

            let msg = create_collection_msg(
                COLLECTION_NAME.to_string(),
                COLLECTION_SYMBOL.to_string(),
            );

            // Artist creates collection
            let info = mock_info(ARTIST, &[]);
            let response = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
            
            // Verify submessage for NFT contract instantiation
            assert_eq!(1, response.messages.len());
            
            // Query and verify collection
            let collection: CollectionResponse = from_json(
                &query(
                    deps.as_ref(),
                    mock_env(),
                    QueryMsg::Collection { artist: ARTIST.to_string() }
                ).unwrap()
            ).unwrap();

            let collection = collection.collection.unwrap();
            assert_eq!(collection.name, COLLECTION_NAME);
            assert_eq!(collection.symbol, COLLECTION_SYMBOL);
            assert_eq!(collection.artist, Addr::unchecked(ARTIST));
            assert_eq!(collection.contract_address, Addr::unchecked(""));
        }

        #[test]
        fn test_invalid_symbol() {
            let mut deps = mock_dependencies();
            setup_contract(deps.as_mut());

            // Test lowercase symbol
            let msg = create_collection_msg(
                "Test Collection".to_string(),
                "drake".to_string(),
            );

            let info = mock_info(ARTIST, &[]);
            let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
            assert_eq!(err, ContractError::InvalidSymbol {});

            // Test symbol with spaces
            let msg = create_collection_msg(
                "Test Collection".to_string(),
                "DRAKE TEST".to_string(),
            );

            let info = mock_info(ARTIST, &[]);
            let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
            assert_eq!(err, ContractError::InvalidSymbol {});
        }

        #[test]
        fn test_duplicate_symbol() {
            let mut deps = mock_dependencies();
            setup_contract(deps.as_mut());

            let msg = create_collection_msg(
                COLLECTION_NAME.to_string(),
                COLLECTION_SYMBOL.to_string(),
            );

            // First creation should succeed
            let info = mock_info(ARTIST, &[]);
            let _res = execute(deps.as_mut(), mock_env(), info, msg.clone()).unwrap();

            // Second creation with same symbol should fail
            let info = mock_info(ARTIST, &[]);
            let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
            assert_eq!(err, ContractError::SymbolAlreadyTaken {});
        }

        #[test]
        fn test_update_nft_code_id() {
            let mut deps = mock_dependencies();
            setup_contract(deps.as_mut());

            let new_code_id = 456u64;
            let msg = ExecuteMsg::UpdateNftCodeId { 
                code_id: new_code_id 
            };

            // Non-owner attempt should fail
            let info = mock_info(ARTIST, &[]);
            let err = execute(deps.as_mut(), mock_env(), info, msg.clone()).unwrap_err();
            assert_eq!(err, ContractError::Unauthorized {});

            // Owner attempt should succeed
            let info = mock_info(OWNER, &[]);
            let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

            // Verify code ID was updated
            let config: ConfigResponse = from_json(
                &query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap()
            ).unwrap();
            assert_eq!(config.nft_code_id, new_code_id);
        }
    }
}