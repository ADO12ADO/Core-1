// Tambahkan import yang hilang
use astroport::token::InstantiateMarketingInfo;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::Item;

/// This structure describes the parameters used for creating a contract.
#[cw_serde]
pub struct InstantiateMsg {
    /// The contract owner address
    pub owner: String,
    /// CW20 token code identifier
    pub token_code_id: u64,
    /// The ASTRO token contract address
    pub deposit_token_addr: String,
    /// The ADO token contract address
    pub astro_token_addr: String, // <-- Tambahkan ini
    /// the marketing info of type [`InstantiateMarketingInfo`]
    pub marketing: Option<InstantiateMarketingInfo>,
}
/// Stores the contract config at the given key
pub const CONFIG: Item<Config> = Item::new("config");
