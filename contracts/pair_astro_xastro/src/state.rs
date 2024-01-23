use astroport_pair_bonded::error::ContractError;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Api};

/// This structure stores a ASTRO-ITO pool's params.
#[cw_serde]
pub struct Params {
    /// ASTRO token contract address.
    pub astro_addr: Addr,
    /// ITO token contract address.
    pub xastro_addr: Addr,
    /// Ito Staking contract address.
    pub staking_addr: Addr,
}

/// This structure stores a ASTRO-ITO pool's init params.
#[cw_serde]
pub struct InitParams {
    /// ASTRO token contract address.
    pub astro_addr: String,
    /// ITO token contract address.
    pub xastro_addr: String,
    /// Ito Staking contract address.
    pub staking_addr: String,
}

impl InitParams {
    pub fn try_into_params(self, api: &dyn Api) -> Result<Params, ContractError> {
        Ok(Params {
            astro_addr: api.addr_validate(&self.astro_addr)?,
            xastro_addr: api.addr_validate(&self.xastro_addr)?,
            staking_addr: api.addr_validate(&self.staking_addr)?,
        })
    }
}

/// This structure describes a migration message.
/// We currently take no arguments for migrations.
#[cw_serde]
pub struct MigrateMsg {}
