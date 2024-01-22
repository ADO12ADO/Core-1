use cosmwasm_std::{
    attr, entry_point, from_binary, to_binary, wasm_execute, Addr, Binary, CosmosMsg, Deps,
    DepsMut, Env, MessageInfo, Reply, ReplyOn, Response, StdError, StdResult, SubMsg,
    SubMsgResponse, SubMsgResult, Uint128, WasmMsg,
};
use cw_utils::parse_instantiate_response_data;

use crate::error::ContractError;
use crate::state::{Config, CONFIG};
use astroport::staking::{
    ConfigResponse, Cw20HookMsg, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg,
};
use cw2::{get_contract_version, set_contract_version};
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg, MinterResponse};

use astroport::querier::{query_supply, query_token_balance};
use astroport::xastro_token::InstantiateMsg as TokenInstantiateMsg;

/// Contract name that is used for migration.
const CONTRACT_NAME: &str = "ito-staking";
/// Contract version that is used for migration.
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// ADO information.
const TOKEN_NAME: &str = "Staked Ito";
const TOKEN_SYMBOL: &str = "ITO";

/// `reply` call code ID used for sub-messages.
const INSTANTIATE_TOKEN_REPLY_ID: u64 = 1;

/// Minimum initial xastro share
pub(crate) const MINIMUM_STAKE_AMOUNT: Uint128 = Uint128::new(1_000);

/// ... (other constants)

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    CONFIG.save(
        deps.storage,
        &Config {
            astro_token_addr: deps.api.addr_validate(&msg.astro_token_addr)?,
            xastro_token_addr: Addr::unchecked(),
            owner: deps.api.addr_validate(&msg.owner)?,
            deposit_token_addr: deps.api.addr_validate(&msg.deposit_token_addr)?,
        },
    )?;

    let sub_msg: Vec<SubMsg> = vec![SubMsg {
        msg: WasmMsg::Instantiate {
            admin: Some(msg.owner),
            code_id: msg.token_code_id,
            msg: to_binary(&TokenInstantiateMsg {
                name: TOKEN_NAME.to_string(),
                symbol: TOKEN_SYMBOL.to_string(),
                decimals: 6,
                initial_balances: vec![],
                mint: Some(MinterResponse {
                    minter: env.contract.address.to_string(),
                    cap: None,
                }),
                marketing: msg.marketing,
            })?,
            funds: vec![],
            label: String::from("Staked Ito Token"),
        }
        .into(),
        id: INSTANTIATE_TOKEN_REPLY_ID,
        gas_limit: None,
        reply_on: ReplyOn::Success,
    }];

    Ok(Response::new().add_submessages(sub_msg))
}

/// ... (other functions)

// ... (query function)
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        // ... (other variants)
        ExecuteMsg::UpdateDepositTokenAddr { new_deposit_token_addr } => {
            let mut config: Config = CONFIG.load(deps.storage)?;
            if info.sender != config.owner {
                return Err(ContractError::Unauthorized {});
            }

            config.deposit_token_addr = deps.api.addr_validate(&new_deposit_token_addr)?;

            CONFIG.save(deps.storage, &config)?;

            Ok(Response::new())
        }
        // ... (other variants)
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    match msg {
        Reply {
            id: INSTANTIATE_TOKEN_REPLY_ID,
            result:
                SubMsgResult::Ok(SubMsgResponse {
                    data: Some(data), ..
                }),
        } => {
            let mut config = CONFIG.load(deps.storage)?;

            if config.xastro_token_addr != Addr::unchecked() {
                return Err(ContractError::Unauthorized {});
            }

            let init_response = parse_instantiate_response_data(data.as_slice())
                .map_err(|e| StdError::generic_err(format!("{e}")))?;

            config.xastro_token_addr = deps.api.addr_validate(&init_response.contract_address)?;

            CONFIG.save(deps.storage, &config)?;

            Ok(Response::new())
        }
        _ => Err(ContractError::FailedToParseReply {}),
    }
}

// ... (other functions)
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    let contract_version = get_contract_version(deps.storage)?;

    match contract_version.contract.as_ref() {
        "ito-staking" => match contract_version.version.as_ref() {
            "1.1.0" | "1.1.1" | "1.1.2" => {}
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
