use cosmwasm_std::{attr, from_json, Event, Response, Uint128};
use cw721::Cw721ReceiveMsg;

use crate::{
    error::ContractError,
    math::add_u32,
    msg::Cw721ReceiveInnerMsg,
    state::{
        AccountInfo, StakingInfo, ACCOUNT_INFOS, CONFIG_ALLOWED_CW721_ADDRS, IS_STAKING_ENABLED,
        OWNER_STAKED_TOKEN_IDS, STAKED_NFT_COUNT, STAKING_INFOS,
    },
};

use super::Context;

pub fn exec_cw721_receive(
    ctx: Context,
    cw721_receive_msg: Cw721ReceiveMsg,
) -> Result<Response, ContractError> {
    let Context { deps, env, info } = ctx;
    let Cw721ReceiveMsg {
        sender: staker_addr,
        token_id,
        msg: execute_msg,
    } = cw721_receive_msg;

    // Ensure the sender is an authorized cw721 NFT collection
    if !CONFIG_ALLOWED_CW721_ADDRS.has(deps.storage, &info.sender) {
        return Err(ContractError::NotAuthorized {
            reason: format!("CW721 address {} not authorized", info.sender),
        });
    }

    let staker_addr = deps.api.addr_validate(&staker_addr)?;
    let mut resp = Response::new().add_attribute("action", "receive_nft");

    match from_json::<Cw721ReceiveInnerMsg>(execute_msg.as_slice())? {
        Cw721ReceiveInnerMsg::Stake {} => {
            // Abort if staking is disabled
            if !IS_STAKING_ENABLED.load(deps.storage)? {
                return Err(ContractError::NotAuthorized {
                    reason: format!("Staking is currently disabled"),
                });
            }

            // Init stacking metadata for this NFT
            STAKING_INFOS.update(
                deps.storage,
                (&info.sender, &token_id),
                |maybe_info| -> Result<_, ContractError> {
                    if maybe_info.is_some() {
                        return Err(ContractError::NotAuthorized {
                            reason: format!("Token ID {} already staked", token_id),
                        });
                    }
                    Ok(StakingInfo {
                        synced_at: env.block.time,
                        owner: staker_addr.to_owned(),
                        staked_at: env.block.time,
                        n_periods_vested: 0,
                    })
                },
            )?;

            // Upsert AccountInfo for staker
            ACCOUNT_INFOS.update(deps.storage, &staker_addr, |maybe_info| -> Result<_, ContractError> {
                let mut info = maybe_info.unwrap_or_else(|| AccountInfo {
                    rewards_claimed: Uint128::zero(),
                    staked_nft_count: 0,
                });
                info.staked_nft_count = add_u32(info.staked_nft_count, 1)?;
                Ok(info)
            })?;

            // Insert into storage that tracks which NFT token IDs are
            // associated with each staker
            // OWNER_STAKED_TOKEN_IDS.save(deps.storage, (&staker_addr, &info.sender, &token_id), &0)?;
            OWNER_STAKED_TOKEN_IDS.update(
                deps.storage,
                (&staker_addr, &info.sender),
                |maybe_token_ids| -> Result<_, ContractError> {
                    let mut ids = maybe_token_ids.unwrap_or_default();
                    ids.push(token_id.to_owned());
                    Ok(ids)
                },
            )?;

            // Increment total NFT's staked
            STAKED_NFT_COUNT.update(deps.storage, |n| -> Result<_, ContractError> { add_u32(n, 1) })?;

            resp = resp.add_event(Event::new("stake").add_attributes(vec![
                attr("cw721_addr", info.sender),
                attr("cw721_token_id", token_id),
                attr("staker_addr", staker_addr),
            ]));
        },
    }

    Ok(resp)
}
