use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::Item;

/// This structure stores the main parameters for the staking contract.
#[cw_serde]
pub struct Config {
    /// The ASTRO token contract address
    pub astro_token_addr: Addr,
    /// The ADO token contract address
    pub xastro_token_addr: Addr,
    /// The owner of the staking contract
    pub owner: Addr, // Add this line for the 'owner' field
}

/// Stores the contract config at the given key
pub const CONFIG: Item<Config> = Item::new("config");
