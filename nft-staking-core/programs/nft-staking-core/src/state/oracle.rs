use anchor_lang::prelude::*;
use mpl_core::types::OracleValidation;

#[account]
pub struct Oracle {
    pub validation: OracleValidation,
    pub bump: u8,
    pub vault_bump: u8,
}

impl Space for Oracle {
    const INIT_SPACE: usize = 5 + 1 + 1; // OracleValidation::V1 (4 results) + bump + vault_bump
}
