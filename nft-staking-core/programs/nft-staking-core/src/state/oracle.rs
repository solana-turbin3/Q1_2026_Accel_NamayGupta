use anchor_lang::prelude::*;

#[account]
pub struct Oracle {
    pub validation: OracleValidation,
    pub bump: u8,
    pub vault_bump: u8,
}

impl Space for Oracle {
    const INIT_SPACE: usize = 8 + 5 + 1; // discriminator + enum + bumps (approx)
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum OracleValidation {
    Uninitialized,
    V1 {
        create: ExternalValidationResult,
        transfer: ExternalValidationResult,
        burn: ExternalValidationResult,
        update: ExternalValidationResult,
    },
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum ExternalValidationResult {
    Approved,
    Rejected,
    Pass,
}
