use cosmwasm_schema::write_api;

use astroport::Vault-ADO-Token::{InstantiateMsg, QueryMsg};
use cw20_base::msg::ExecuteMsg;

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        query: QueryMsg,
        execute: ExecuteMsg
    }
}
