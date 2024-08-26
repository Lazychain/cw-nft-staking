use crate::error::ContractError;
use crate::execute::claim::exec_claim;
use crate::execute::cw721_receive::exec_cw721_receive;
use crate::execute::{unstake::exec_unstake, Context};
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::query::account::query_account;
use crate::query::config::query_config;
use crate::query::metadata::query_metadata;
use crate::query::ReadonlyContext;
use crate::state;
use cosmwasm_std::{entry_point, to_json_binary};
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response};
use cw2::set_contract_version;

const CONTRACT_NAME: &str = "crates.io:cw-contract";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(state::init(Context { deps, env, info }, msg)?)
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let ctx = Context { deps, env, info };
    match msg {
        ExecuteMsg::ReceiveNft(cw721_receive_msg) => exec_cw721_receive(ctx, cw721_receive_msg),
        ExecuteMsg::Unstake { cw721_addr, token_id } => exec_unstake(ctx, cw721_addr, token_id),
        ExecuteMsg::Claim {} => exec_claim(ctx),
    }
}

#[entry_point]
pub fn query(
    deps: Deps,
    env: Env,
    msg: QueryMsg,
) -> Result<Binary, ContractError> {
    let ctx = ReadonlyContext { deps, env };
    let result = match msg {
        QueryMsg::Account { address } => to_json_binary(&query_account(ctx, address)?),
        QueryMsg::Config {} => to_json_binary(&query_config(ctx)?),
        QueryMsg::Metadata {} => to_json_binary(&query_metadata(ctx)?),
    }?;
    Ok(result)
}

#[entry_point]
pub fn migrate(
    deps: DepsMut,
    _env: Env,
    _msg: MigrateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::default())
}
