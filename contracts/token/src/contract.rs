use cosmwasm_std::{
    attr, ensure_eq, ensure_ne, entry_point, to_json_binary, Binary, Deps, DepsMut, Env,
    MessageInfo, Reply, Response, SubMsg, Uint128,
};
use lido_helpers::answer::{attr_coin, response};
use lido_staking_base::{
    msg::token::{ConfigResponse, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg},
    state::token::{CORE_ADDRESS, DENOM},
};
use neutron_sdk::{
    bindings::{msg::NeutronMsg, query::NeutronQuery},
    query::token_factory::query_full_denom,
};

use crate::error::{ContractError, ContractResult};

pub const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const CREATE_DENOM_REPLY_ID: u64 = 1;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response<NeutronMsg>> {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let core = deps.api.addr_validate(&msg.core_address)?;
    CORE_ADDRESS.save(deps.storage, &core)?;

    DENOM.save(deps.storage, &msg.subdenom)?;
    let create_denom_msg = SubMsg::reply_on_success(
        NeutronMsg::submit_create_denom(&msg.subdenom),
        CREATE_DENOM_REPLY_ID,
    );

    Ok(response(
        "instantiate",
        CONTRACT_NAME,
        [attr("core_address", core), attr("subdenom", msg.subdenom)],
    )
    .add_submessage(create_denom_msg))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> ContractResult<Response<NeutronMsg>> {
    let core = CORE_ADDRESS.load(deps.storage)?;
    ensure_eq!(info.sender, core, ContractError::Unauthorized);

    match msg {
        ExecuteMsg::Mint { amount, receiver } => mint(deps, amount, receiver),
        ExecuteMsg::Burn {} => burn(deps, info),
    }
}

fn mint(
    deps: DepsMut<NeutronQuery>,
    amount: Uint128,
    receiver: String,
) -> ContractResult<Response<NeutronMsg>> {
    ensure_ne!(amount, Uint128::zero(), ContractError::NothingToMint);

    let denom = DENOM.load(deps.storage)?;
    let mint_msg = NeutronMsg::submit_mint_tokens(&denom, amount, &receiver);

    Ok(response(
        "execute-mint",
        CONTRACT_NAME,
        [
            attr_coin("amount", amount, denom),
            attr("receiver", receiver),
        ],
    )
    .add_message(mint_msg))
}

fn burn(deps: DepsMut<NeutronQuery>, info: MessageInfo) -> ContractResult<Response<NeutronMsg>> {
    let denom = DENOM.load(deps.storage)?;
    let amount = cw_utils::must_pay(&info, &denom)?;

    let burn_msg = NeutronMsg::submit_burn_tokens(&denom, amount);

    Ok(response(
        "execute-burn",
        CONTRACT_NAME,
        [attr_coin("amount", amount, denom)],
    )
    .add_message(burn_msg))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps<NeutronQuery>, _env: Env, msg: QueryMsg) -> ContractResult<Binary> {
    match msg {
        QueryMsg::Config {} => {
            let core_address = CORE_ADDRESS.load(deps.storage)?.into_string();
            let denom = DENOM.load(deps.storage)?;
            Ok(to_json_binary(&ConfigResponse {
                core_address,
                denom,
            })?)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(
    _deps: DepsMut,
    _env: Env,
    _msg: MigrateMsg,
) -> ContractResult<Response<NeutronMsg>> {
    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    msg: Reply,
) -> ContractResult<Response<NeutronMsg>> {
    match msg.id {
        CREATE_DENOM_REPLY_ID => {
            let subdenom = DENOM.load(deps.storage)?;
            let full_denom = query_full_denom(deps.as_ref(), env.contract.address, subdenom)?;
            DENOM.save(deps.storage, &full_denom.denom)?;

            Ok(response(
                "reply-create-denom",
                CONTRACT_NAME,
                [attr("denom", full_denom.denom)],
            ))
        }
        id => Err(ContractError::UnknownReplyId { id }),
    }
}
