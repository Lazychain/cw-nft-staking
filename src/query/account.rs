use cosmwasm_std::Addr;

use crate::{error::ContractError, responses::AccountResponse};

use super::ReadonlyContext;

pub fn query_account(
    ctx: ReadonlyContext,
    _address: Addr,
) -> Result<AccountResponse, ContractError> {
    let ReadonlyContext { .. } = ctx;
    Ok(AccountResponse {})
}
