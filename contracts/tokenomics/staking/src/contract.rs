use cosmwasm_std::{
    attr, entry_point, from_binary, to_binary, wasm_execute, Addr, Binary, CosmosMsg, Deps,
    DepsMut, Env, MessageInfo, Reply, ReplyOn, Response, StdResult, SubMsg, SubMsgResult, SubMsgResponse, Uint128, WasmMsg,
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

// ... (konstanta dan fungsi lainnya)

/// Receives a message of type [`Cw20ReceiveMsg`] and processes it depending on the received template.
///
/// * **cw20_msg** CW20 message to process.
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn receive_cw20(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    let config: Config = CONFIG.load(deps.storage)?;

    let recipient = cw20_msg.sender;
    let mut amount = cw20_msg.amount;

    let mut total_deposit = query_token_balance(
        &deps.querier,
        &config.astro_token_addr,
        env.contract.address.clone(),
    )?;
    let total_shares = query_supply(&deps.querier, &config.xastro_token_addr)?;

    match from_binary(&cw20_msg.msg)? {
        Cw20HookMsg::Enter => {
            let mut messages = vec![];
            if info.sender != config.astro_token_addr {
                return Err(ContractError::Unauthorized {});
            }

            // Convert the amount to Uint128 with the correct decimal places (6 in this case)
            let amount_with_decimals = Uint128::from(amount.u128() * 1_000_000);

            // Check if the total deposit after the current stake exceeds the limit (21 million tokens)
            let total_deposit_after_stake = total_deposit
                .checked_add(amount_with_decimals)
                .ok_or(ContractError::ArithmeticError {})?;

            let total_deposit_limit = Uint128::new(21_000_000 * 1_000_000); // 21 million with 6 decimal places

            if total_deposit_after_stake > total_deposit_limit {
                return Err(ContractError::ExceedsTotalDepositLimit {});
            }

            // Continue with the rest of the logic for entering the stake
            // ...

            Ok(Response::new().add_messages(messages).add_attributes(vec![
                attr("action", "enter"),
                attr("recipient", recipient),
                attr("astro_amount", cw20_msg.amount),
                attr("xastro_amount", amount_with_decimals),
            ]))
        }
        // Handle other cases if needed
        _ => Err(ContractError::InvalidCw20Hook {}),
    }
} // tutup match

// ... (fungsi lainnya)

 messages.push(wasm_execute(
 config.xastro_token_addr.clone(),
                    &Cw20ExecuteMsg::Mint {
                        recipient: env.contract.address.to_string(),
                        amount: MINIMUM_STAKE_AMOUNT,
                    },
                    vec![],
                )?);

                amount
            } else {
                amount = amount
                    .checked_mul(total_shares)?
                    .checked_div(total_deposit)?;

                if amount.is_zero() {
                    return Err(ContractError::StakeAmountTooSmall {});
                }

                amount
            };

            messages.push(wasm_execute(
                config.xastro_token_addr,
                &Cw20ExecuteMsg::Mint {
                    recipient: recipient.clone(),
                    amount: mint_amount,
                },
                vec![],
            )?);

            Ok(Response::new().add_messages(messages).add_attributes(vec![
                attr("action", "enter"),
                attr("recipient", recipient),
                attr("astro_amount", cw20_msg.amount),
                attr("xastro_amount", mint_amount),
            ]))
        }
        Cw20HookMsg::Leave {} => {
            if info.sender != config.xastro_token_addr {
                return Err(ContractError::Unauthorized {});
            }

            let what = amount
                .checked_mul(total_deposit)?
                .checked_div(total_shares)?;

            // Burn share
            let res = Response::new()
                .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: config.xastro_token_addr.to_string(),
                    msg: to_binary(&Cw20ExecuteMsg::Burn { amount })?,
                    funds: vec![],
                }))
                .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: config.astro_token_addr.to_string(),
                    msg: to_binary(&Cw20ExecuteMsg::Transfer {
                        recipient: recipient.clone(),
                        amount: what,
                    })?,
                    funds: vec![],
                }));

            Ok(res.add_attributes(vec![
                attr("action", "leave"),
                attr("recipient", recipient),
                attr("xastro_amount", cw20_msg.amount),
                attr("astro_amount", what),
            ]))
        }
    }
}

/// Exposes all the queries available in the contract.
///
/// ## Queries
/// * **QueryMsg::Config {}** Returns the staking contract configuration using a [`ConfigResponse`] object.
///
/// * **QueryMsg::TotalShares {}** Returns the total ITO supply using a [`Uint128`] object.
///
/// * **QueryMsg::Config {}** Returns the amount of ASTRO that's currently in the staking pool using a [`Uint128`] object.
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

/// ## Description
/// Used for migration of contract. Returns the default object of type [`Response`].
/// ## Params
/// * **_deps** is the object of type [`DepsMut`].
///
/// * **_env** is the object of type [`Env`].
///
/// * **_msg** is the object of type [`MigrateMsg`].
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    let contract_version = get_contract_version(deps.storage)?;

    match contract_version.contract.as_ref() {
        "ito-staking" => match contract_version.version.as_ref() {
            "1.0.0" | "1.0.1" | "1.0.2" => {}
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
