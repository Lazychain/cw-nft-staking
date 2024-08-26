use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Order, Response, StdResult, Storage, Timestamp, Uint128};

use crate::{
    error::ContractError,
    execute::Context,
    math::{add_u128, add_u32, mul_u128},
    msg::InstantiateMsg,
    token::Token,
};
use cw_storage_plus::{Item, Map};

pub const CONFIG_MANAGER_ADDR: Item<Addr> = Item::new("manager_addr");
pub const CONFIG_ALLOWED_CW721_ADDRS: Map<&Addr, u8> = Map::new("allowed_cw721_addrs");
pub const CONFIG_REWARDS_TOKEN: Item<Token> = Item::new("rewards_token");
pub const CONFIG_REWARDS_AMOUNT: Item<Uint128> = Item::new("rewards_amount");
pub const CONFIG_VESTING_PERIOD_SECONDS: Item<u32> = Item::new("vesting_period_seconds");
pub const CONFIG_MIN_VESTING_PERIODS: Item<u32> = Item::new("min_vesting_periods");

pub const IS_STAKING_ENABLED: Item<bool> = Item::new("is_staking_enabled");
pub const STAKING_INFOS: Map<(&Addr, &String), StakingInfo> = Map::new("staking_infos");
pub const STAKED_NFT_COUNT: Item<u32> = Item::new("staked_nft_count");
pub const OWNER_STAKED_TOKEN_IDS: Map<(&Addr, &Addr), Vec<String>> = Map::new("owner_staked_token_ids");
pub const TOTAL_REWARDS_CLAIMED: Item<Uint128> = Item::new("total_rewards_claimed");
pub const ACCOUNT_INFOS: Map<&Addr, AccountInfo> = Map::new("account_infos");

#[cw_serde]
pub struct StakingInfo {
    pub staked_at: Timestamp,
    pub synced_at: Timestamp,
    pub n_periods_vested: u32,
    pub owner: Addr,
}

#[cw_serde]
pub struct AccountInfo {
    pub staked_nft_count: u32,
    pub rewards_claimed: Uint128,
}

/// Top-level initialization of contract state
pub fn init(
    ctx: Context,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let Context { deps, .. } = ctx;

    CONFIG_MANAGER_ADDR.save(deps.storage, &deps.api.addr_validate(msg.manager_addr.as_str())?)?;
    CONFIG_REWARDS_AMOUNT.save(deps.storage, &msg.vesting_period_rewards)?;
    CONFIG_REWARDS_TOKEN.save(deps.storage, &msg.rewards_token)?;
    CONFIG_MIN_VESTING_PERIODS.save(deps.storage, &msg.min_vesting_periods)?;
    CONFIG_VESTING_PERIOD_SECONDS.save(deps.storage, &msg.vesting_period_seconds)?;

    for cw721_addr in msg.allowed_cw721_addrs.iter() {
        CONFIG_ALLOWED_CW721_ADDRS.save(deps.storage, &deps.api.addr_validate(cw721_addr.as_str())?, &0)?;
    }

    IS_STAKING_ENABLED.save(deps.storage, &true)?;

    Ok(Response::new().add_attribute("action", "instantiate"))
}

pub struct VestingResults {
    pub amount_vested: Uint128,
    pub vested_staking_infos: Vec<((Addr, String), StakingInfo)>,
}

pub fn vest_nfts(
    store: &dyn Storage,
    block_time: Timestamp,
    owner: &Addr,
    account: &AccountInfo,
) -> Result<VestingResults, ContractError> {
    let vesting_period_seconds = CONFIG_VESTING_PERIOD_SECONDS.load(store)? as u64;
    let rewards_amount: Uint128 = CONFIG_REWARDS_AMOUNT.load(store)?;

    let mut total_amount_vested = Uint128::zero();
    let mut vested_staking_infos: Vec<((Addr, String), StakingInfo)> =
        Vec::with_capacity(account.staked_nft_count as usize);

    for result in OWNER_STAKED_TOKEN_IDS
        .prefix(owner)
        .range(store, None, None, Order::Ascending)
        .collect::<Vec<StdResult<_>>>()
    {
        let (cw721_addr, token_ids) = result?;

        for token_id in token_ids.iter() {
            let (amount_vested, n_periods_vested, mut staking_info) = vest_one_nft(
                store,
                vesting_period_seconds,
                rewards_amount,
                block_time,
                &cw721_addr,
                token_id,
            )?;

            if n_periods_vested > 0 {
                total_amount_vested = add_u128(total_amount_vested, amount_vested)?;
                staking_info.n_periods_vested = add_u32(staking_info.n_periods_vested, n_periods_vested as u32)?;
                staking_info.synced_at = block_time;
                vested_staking_infos.push(((cw721_addr.to_owned(), token_id.to_owned()), staking_info))
            }
        }
    }

    Ok(VestingResults {
        amount_vested: total_amount_vested,
        vested_staking_infos,
    })
}

pub fn vest_one_nft(
    store: &dyn Storage,
    vesting_period_seconds: u64,
    unit_rewards_amount: Uint128,
    block_time: Timestamp,
    cw721_addr: &Addr,
    token_id: &String,
) -> Result<(Uint128, u32, StakingInfo), ContractError> {
    let mut total_amount_vested = Uint128::zero();
    let mut staking_info = STAKING_INFOS.load(store, (&cw721_addr, token_id))?;
    let seconds_since_last_sync = block_time.seconds() - staking_info.synced_at.seconds();
    let n_periods_vested = seconds_since_last_sync % vesting_period_seconds;

    if n_periods_vested > 0 {
        total_amount_vested = mul_u128(n_periods_vested, unit_rewards_amount)?;
        staking_info.n_periods_vested = add_u32(staking_info.n_periods_vested, n_periods_vested as u32)?;
        staking_info.synced_at = block_time;
    }

    Ok((total_amount_vested, n_periods_vested as u32, staking_info))
}
