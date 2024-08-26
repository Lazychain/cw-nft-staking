# NFT Staking Smart Contract

Stake NFT's from one or more CW721 collections to earn tokens at the end of each
vesting period, which you must claim at your convenience. Each NFT vests on its
own schedule, relative to the block time in which it is staked. It is possible
for the contract's admin to set a minimum number of vesting periods during which
an NFT cannot be unstaked. For example, if the vesting period is 12 hours and
the minimum required vesting periods is 2, then the NFT will remain locked for a
total duration of 24 hours.
