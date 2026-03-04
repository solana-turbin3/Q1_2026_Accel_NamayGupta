use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};
use mpl_core::types::{ExternalValidationResult, OracleValidation};

use crate::errors::StakingError;
use crate::state::Oracle;
use crate::utils::{is_close_to_open_close, is_transfer_allowed, REWARD_IN_LAMPORTS};

#[derive(Accounts)]
pub struct UpdateValidationOracle<'info> {
    pub signer: Signer<'info>,
    #[account(
        mut,
        seeds = [b"oracle"],
        bump = oracle.bump,
    )]
    pub oracle: Account<'info, Oracle>,
    #[account(
        mut,
        seeds = [b"vault", oracle.key().as_ref()],
        bump = oracle.vault_bump,
    )]
    pub vault: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}

impl<'info> UpdateValidationOracle<'info> {
    pub fn update_validation_oracle(&mut self) -> Result<()> {
        let current_time = Clock::get()?.unix_timestamp;
        let approved = OracleValidation::V1 {
            transfer: ExternalValidationResult::Approved,
            create: ExternalValidationResult::Pass,
            update: ExternalValidationResult::Pass,
            burn: ExternalValidationResult::Pass,
        };
        let rejected = OracleValidation::V1 {
            transfer: ExternalValidationResult::Rejected,
            create: ExternalValidationResult::Pass,
            update: ExternalValidationResult::Pass,
            burn: ExternalValidationResult::Pass,
        };

        if is_transfer_allowed(current_time) {
            require!(
                self.oracle.validation == rejected,
                StakingError::OracleAlreadyUpdated
            );
            self.oracle.validation = approved;
        } else {
            require!(
                self.oracle.validation == approved,
                StakingError::OracleAlreadyUpdated
            );
            self.oracle.validation = rejected;
        }
        let vault_lamports = self.vault.lamports();
        let oracle_key = self.oracle.key();
        let signer_seeds: &[&[&[u8]]] =
            &[&[b"vault", oracle_key.as_ref(), &[self.oracle.vault_bump]]];

        if is_close_to_open_close(current_time) && vault_lamports > REWARD_IN_LAMPORTS {
            transfer(
                CpiContext::new_with_signer(
                    self.system_program.to_account_info(),
                    Transfer {
                        from: self.vault.to_account_info(),
                        to: self.signer.to_account_info(),
                    },
                    signer_seeds,
                ),
                REWARD_IN_LAMPORTS,
            )?;
        }

        Ok(())
    }
}
