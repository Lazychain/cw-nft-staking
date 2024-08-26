use crate::{
    error::ContractError,
    math::{add_u128, sub_u32},
    state::{
        vest_one_nft, ACCOUNT_INFOS, CONFIG_MIN_VESTING_PERIODS, CONFIG_REWARDS_AMOUNT, CONFIG_REWARDS_TOKEN,
        CONFIG_VESTING_PERIOD_SECONDS, OWNER_STAKED_TOKEN_IDS, STAKED_NFT_COUNT, STAKING_INFOS, TOTAL_REWARDS_CLAIMED,
    },
};
use cosmwasm_std::{attr, to_json_binary, Addr, Response, SubMsg, WasmMsg};
use cw721::Cw721ExecuteMsg;

use super::Context;

pub fn exec_unstake(
    ctx: Context,
    cw721_addr: Addr,
    token_id: String,
) -> Result<Response, ContractError> {
    let Context { deps, info, env } = ctx;

    let mut token_ids = OWNER_STAKED_TOKEN_IDS.load(deps.storage, (&info.sender, &cw721_addr))?;

    // Ensure that only the NFT owner can unstake their own NFT, and remove the
    // token ID from the cw721's token_ids vec.
    if let Some(idx) = token_ids.iter().position(|id| **id == token_id) {
        token_ids.remove(idx);
    } else {
        return Err(ContractError::NotAuthorized {
            reason: format!("Staked NFT not owned by sender"),
        });
    }

    let mut resp = Response::new().add_attributes(vec![
        attr("action", "unstake"),
        attr("cw721_addr", cw721_addr.to_owned()),
        attr("token_id", token_id.to_owned()),
    ]);

    // Check if there are any outstanding vested rewards for this NFT and if so,
    // send them to the owner.
    let (amount_vested, _, staking_info) = vest_one_nft(
        deps.storage,
        CONFIG_VESTING_PERIOD_SECONDS.load(deps.storage)? as u64,
        CONFIG_REWARDS_AMOUNT.load(deps.storage)?,
        env.block.time,
        &cw721_addr,
        &token_id,
    )?;

    // Ensure that the NFT has been staked for the minimum vesting period count
    let min_vesting_periods = CONFIG_MIN_VESTING_PERIODS.load(deps.storage)? as u64;
    if staking_info.n_periods_vested < min_vesting_periods as u32 {
        return Err(ContractError::NotAuthorized {
            reason: format!(
                "NFT is cannot be unstaked until it has vested at least {} times",
                min_vesting_periods
            ),
        });
    }

    // Send oustanding rewards to NFT owner
    if !amount_vested.is_zero() {
        let token = CONFIG_REWARDS_TOKEN.load(deps.storage)?;
        resp = resp.add_submessage(token.transfer(&info.sender, amount_vested)?);
    }

    // Decrement account's count of staked NFTs
    ACCOUNT_INFOS.update(deps.storage, &info.sender, |maybe_info| -> Result<_, ContractError> {
        let mut account_info = maybe_info.unwrap();
        account_info.staked_nft_count = sub_u32(account_info.staked_nft_count, 1)?;
        Ok(account_info)
    })?;

    // Decrement global total NFT's staked
    STAKED_NFT_COUNT.update(deps.storage, |n| -> Result<_, ContractError> { sub_u32(n, 1) })?;

    // Increment global total claimed
    TOTAL_REWARDS_CLAIMED.update(deps.storage, |n| -> Result<_, ContractError> {
        add_u128(n, amount_vested)
    })?;

    // Remove staking metadata for the unstaked NFT
    STAKING_INFOS.remove(deps.storage, (&cw721_addr, &token_id));

    // Send NFT back to owner
    Ok(resp.add_submessage(SubMsg::new(WasmMsg::Execute {
        contract_addr: cw721_addr.to_string(),
        msg: to_json_binary(&Cw721ExecuteMsg::TransferNft {
            recipient: info.sender.to_string(),
            token_id,
        })?,
        funds: vec![],
    })))
}
