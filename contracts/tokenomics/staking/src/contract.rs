use cosmwasm_std::{
    attr, entry_point, from_binary, to_binary, wasm_execute, Addr, Binary, CosmosMsg, Deps,
    DepsMut, Env, MessageInfo, Reply, ReplyOn, Response, StdError, StdResult, SubMsg,
    SubMsgResponse, SubMsgResult, Uint128, WasmMsg,
};
use cw_utils::parse_instantiate_response_data;

use crate::error::ContractError;
use crate::state::{Config, CONFIG};
use astroport::staking::{
    ConfigResponse, Cw20HookMsg, Cw20ExecuteMsg, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg,
};
use cw2::{get_contract_version, set_contract_version};
use cw20::{Cw20ReceiveMsg, MinterResponse};

use astroport::querier::{query_supply, query_token_balance};
use astroport::xastro_token::InstantiateMsg as TokenInstantiateMsg;
// Contract name that is used for migration.
const CONTRACT_NAME: &str = "ito-staking";
// Contract version that is used for migration.
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// ITO information.
const TOKEN_NAME: &str = "Staked Ito";
const TOKEN_SYMBOL: &str = "ITO";

// A `reply` call code ID used for sub-messages.
const INSTANTIATE_TOKEN_REPLY_ID: u64 = 1;

// Minimum initial xastro share
pub(crate) const MINIMUM_STAKE_AMOUNT: Uint128 = Uint128::new(1_000);

// ... (other constants and structs) ...
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // Store config
    let mut config = Config {
        astro_token_addr: deps.api.addr_validate(&msg.deposit_token_addr)?,
        xastro_token_addr: Addr::unchecked(""),
    };

    // Execute the message to get the actual token address
    let reply_id = INSTANTIATE_TOKEN_REPLY_ID;
    let sub_msg: SubMsg = SubMsg {
        msg: WasmMsg::Execute {
            contract_addr: msg.deposit_token_addr.clone(),
            msg: to_binary(&Cw20ExecuteMsg::UpdateConfig {
                astro_token_addr: Some(config.astro_token_addr.clone()),
            })?,
            funds: vec![],
        },
        id: reply_id,
        gas_limit: None,
        reply_on: ReplyOn::Success,
    };

    Ok(Response::new().add_submessages(vec![sub_msg]))
}
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Receive(msg) => receive_cw20(deps, env, info, msg),
    }
}
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    match msg {
        Reply {
            id: INSTANTIATE_TOKEN_REPLY_ID,
            result: SubMsgResult::Ok(SubMsgResponse { data, .. }),
        } => {
            // Parse the response data and update the config
            let response_data = parse_instantiate_response_data(data.as_slice())
                .map_err(|e| StdError::generic_err(format!("{e}")))?;

            config.astro_token_addr = deps.api.addr_validate(&response_data.contract_address)?;

            CONFIG.save(deps.storage, &config)?;

            Ok(Response::new())
        }
        _ => Err(ContractError::FailedToParseReply {}),
    }
}
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let config = CONFIG.load(deps.storage)?;
    match msg {
        QueryMsg::Config {} => Ok(to_binary(&ConfigResponse {
            deposit_token_addr: config.astro_token_addr,
            share_token_addr: config.xastro_token_addr,
        })?),
        QueryMsg::TotalShares {} => {
            to_binary(&query_supply(&deps.querier, &config.xastro_token_addr)?)
        }
        QueryMsg::TotalDeposit {} => to_binary(&query_token_balance(
            &deps.querier,
            &config.astro_token_addr,
            env.contract.address,
        )?),
    }
}
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    let contract_version = get_contract_version(deps.storage)?;

    match contract_version.contract.as_ref() {
        "ito-staking" => match contract_version.version.as_ref() {
            "1.1.0" | "1.0.1" | "1.0.2" => {}
            _ => return Err(ContractError::MigrationError {}),
        },
        _ => return Err(ContractError::MigrationError {}),
    }

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new()
        .add_attribute("previous_contract_name", &contract_version.contract)
        .add_attribute("previous_contract_version", &contract_version.version)
        .add_attribute("new_contract_name", CONTRACT_NAME)
        .add_attribute("new_contract_version", CONTRACT_VERSION))
}
