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
    const MINTER: &str = "minter";
    const NFT_CODE_ID: u64 = 123;
    const PASS_PRICE: u128 = 10;
    const PASS_DURATION: u64 = 2592000;   // 30 days
    const GRACE_PERIOD: u64 = 259200; 
    const HOUSE_ROYALTY: u32 = 30;
    const ARTIST_ROYALTY : u32 = 70;    

    // Helper function to instantiate the contract
    fn setup_contract(deps: DepsMut) -> Response {
        let msg = InstantiateMsg {
            nft_code_id: NFT_CODE_ID,
            price: PASS_PRICE,
            duration: PASS_DURATION,
            grace_period: GRACE_PERIOD,
            payment_address: Addr::unchecked(OWNER),
            house_percentage: HOUSE_ROYALTY,
            artist_percentage: ARTIST_ROYALTY
        };
        
        let info = mock_info(OWNER, &[]);
        let env = mock_env();

        instantiate(deps, env, info, msg).unwrap()
    }

    // Helper function to create a collection message
    fn create_collection_msg(name: String, symbol: String, artist: Addr, minter: Addr) -> ExecuteMsg {
        ExecuteMsg::CreateCollection {
            name,
            symbol,
            artist,
            minter,
            collection_info: "Test Collection Metadata".to_string(), // Now using proper String
        }
    }

    mod factory_tests {
        use super::*;

        #[test]
        fn proper_initialization() {
            let mut deps = mock_dependencies();
            let response = setup_contract(deps.as_mut());
            assert_eq!(0, response.messages.len());

            // Query and verify config
            let config: ConfigResponse = from_json(
                &query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap()
            ).unwrap();

            println!("Config Response: {:?}", config);
            assert_eq!(config.nft_code_id, NFT_CODE_ID);
            assert_eq!(config.admin, OWNER);
            assert_eq!(config.total_collections, 0);
        }

        #[test]
        fn test_create_collection() {
            let mut deps = mock_dependencies();
            setup_contract(deps.as_mut());
            
            let artist = Addr::unchecked(ARTIST);
            let minter = Addr::unchecked(MINTER);

            let msg = create_collection_msg(
                "Drake Collection".to_string(),
                "DRAKE".to_string(),
                artist.clone(),
                minter.clone(),
            );

            // Execute as admin (not artist)
            let info = mock_info(OWNER, &[]);
            let response = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
            
            println!("Create Collection Response: {:?}", response);
            
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
            println!("Created Collection: {:?}", collection);
            
            assert_eq!(collection.name, "Drake Collection");
            assert_eq!(collection.symbol, "DRAKE");
            assert_eq!(collection.artist, artist);
            assert_eq!(collection.minter, minter);
            assert_eq!(collection.collection_info, "Test Collection Metadata");
        }

        #[test]
        fn test_unauthorized_collection_creation() {
            let mut deps = mock_dependencies();
            setup_contract(deps.as_mut());

            let msg = create_collection_msg(
                "Test Collection".to_string(),
                "TEST".to_string(),
                Addr::unchecked(ARTIST),
                Addr::unchecked(MINTER),
            );

            // Try to create collection as artist (should fail)
            let info = mock_info(ARTIST, &[]);
            let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
            assert_eq!(err, ContractError::Unauthorized {});
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
            let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
            println!("Update Code ID Response: {:?}", res);

            // Verify code ID was updated
            let config: ConfigResponse = from_json(
                &query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap()
            ).unwrap();
            assert_eq!(config.nft_code_id, new_code_id);
        }
    }
}