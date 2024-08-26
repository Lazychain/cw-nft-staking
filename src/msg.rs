use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128};
use cw721::Cw721ReceiveMsg;

use crate::{
    responses::{AccountResponse, ConfigResponse, MetadataResponse},
    token::Token,
};

#[cw_serde]
pub struct InstantiateMsg {
    pub manager_addr: Addr,
    pub allowed_cw721_addrs: Vec<Addr>,
    pub vesting_period_seconds: u32,
    pub vesting_period_rewards: Uint128,
    pub min_vesting_periods: u32,
    pub rewards_token: Token,
}

#[cw_serde]
#[derive(cw_orch::ExecuteFns)]
pub enum ExecuteMsg {
    ReceiveNft(Cw721ReceiveMsg),
    Unstake { cw721_addr: Addr, token_id: String },
    Claim {},
}

#[cw_serde]
#[derive(cw_orch::QueryFns, QueryResponses)]
pub enum QueryMsg {
    #[returns(AccountResponse)]
    Account { address: Addr },

    #[returns(ConfigResponse)]
    Config {},

    #[returns(MetadataResponse)]
    Metadata {},
}

#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
pub enum Cw721ReceiveInnerMsg {
    Stake {},
}
