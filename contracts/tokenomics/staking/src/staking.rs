// File: astroport/staking.rs

use cosmwasm_std::{Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult, WasmMsg};

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    // ... other variants

    UpdateDepositTokenAddr {
        new_deposit_token_addr: String,
    },
    // ... other variants
}

impl ExecuteMsg {
    // ... any additional helper methods or associated functions
}
