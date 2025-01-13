use cosmwasm_schema::write_api;
use  pass_nft::msg::{ExecuteMsg, InstantiateMsg, };
use pass_nft::schema_types::SchemaQueryMsg;

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        execute: ExecuteMsg,
        query: SchemaQueryMsg,
    }
}
