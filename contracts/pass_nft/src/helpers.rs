use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{to_json_binary, Addr, MessageInfo,  CosmosMsg, StdResult, WasmMsg};
use crate::msg::{ExecuteMsg, QueryMsg, PassMsg};
use crate::error::ContractError;



/// CwTemplateContract is a wrapper around Addr that provides a lot of helpers
/// for working with this.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct CwTemplateContract(pub Addr);

impl CwTemplateContract {
    pub fn addr(&self) -> Addr {
        self.0.clone()
    }

    pub fn call<T: Into<ExecuteMsg>>(&self, msg: T) -> StdResult<CosmosMsg> {
        let msg = to_json_binary(&msg.into())?;
        Ok(WasmMsg::Execute {
            contract_addr: self.addr().into(),
            msg,
            funds: vec![],
        }
        .into())
    }
}


// custom helpers 

pub fn convert_query_msg(msg: QueryMsg) -> Result<cw721_base_soulbound::QueryMsg<PassMsg>, ContractError> {
    match msg {
        QueryMsg::OwnerOf { token_id, include_expired } => 
            Ok(cw721_base_soulbound::QueryMsg::<PassMsg>::OwnerOf { token_id, include_expired }),
        QueryMsg::Approval { token_id, spender, include_expired } => 
            Ok(cw721_base_soulbound::QueryMsg::<PassMsg>::Approval { token_id, spender, include_expired }),
        // Add other base query conversions...
        _ => Err(ContractError::Custom("Unsupported operation".to_string())),
    }
}

// Function to validate payment amount in uxion
pub fn validate_payment(info: &MessageInfo, required_price: u128) -> Result<(), ContractError> {
    // Find payment in uxion denomination
    let payment = info
        .funds
        .iter()
        .find(|c| c.denom == "uxion")
        .ok_or_else(|| ContractError::Custom("No uxion payment found".to_string()))?;

    // Check if payment amount meets required price
    if payment.amount.u128() < required_price {
        return Err(ContractError::Custom(format!(
            "Insufficient payment. Required: {} uxion, provided: {} uxion",
            required_price,
            payment.amount
        )));
    }

    Ok(())
}