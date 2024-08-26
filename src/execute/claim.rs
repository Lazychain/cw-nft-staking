use crate::{
    error::ContractError,
    math::add_u128,
    state::{vest_nfts, VestingResults, ACCOUNT_INFOS, CONFIG_REWARDS_TOKEN, STAKING_INFOS, TOTAL_REWARDS_CLAIMED},
};
use cosmwasm_std::{attr, Response};

use super::Context;

pub fn exec_claim(ctx: Context) -> Result<Response, ContractError> {
    let Context { deps, info, env } = ctx;
    let mut account = ACCOUNT_INFOS.load(deps.storage, &info.sender)?;
    let mut resp = Response::new().add_attributes(vec![attr("action", "claim")]);

    let VestingResults {
        amount_vested,
        vested_staking_infos,
    } = vest_nfts(deps.storage, env.block.time, &info.sender, &account)?;

    if !amount_vested.is_zero() {
        // Send staker total amount vested
        let token = CONFIG_REWARDS_TOKEN.load(deps.storage)?;
        resp = resp.add_submessage(token.transfer(&info.sender, amount_vested)?);

        // Increment account's historical total amount claimed
        account.rewards_claimed = add_u128(account.rewards_claimed, amount_vested)?;
        ACCOUNT_INFOS.save(deps.storage, &info.sender, &account)?;
    }

    // Persist updates to vested NFT staking infos
    for ((cw721_addr, token_id), info) in vested_staking_infos.iter() {
        STAKING_INFOS.save(deps.storage, (cw721_addr, token_id), info)?;
    }

    // Increment global total claimed
    TOTAL_REWARDS_CLAIMED.update(deps.storage, |n| -> Result<_, ContractError> {
        add_u128(n, amount_vested)
    })?;

    Ok(resp)
}
