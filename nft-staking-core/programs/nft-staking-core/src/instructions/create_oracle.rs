use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};


use crate::state::{Oracle, OracleValidation, ExternalValidationResult};
use crate::utils::{is_transfer_allowed, REWARD_IN_LAMPORTS};

#[derive(Accounts)]
pub struct CreateOracle<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        init_if_needed,
        payer = payer,
        space = 8 + Oracle::INIT_SPACE, 
        seeds = [b"oracle"],
        bump
    )]
    pub oracle: Account<'info, Oracle>,
    #[account(
        mut,
        seeds = [b"vault", oracle.key().as_ref()],
        bump
    )]
    pub vault: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}

impl<'info> CreateOracle<'info> {
    pub fn create_oracle(&mut self, bumps: &CreateOracleBumps) -> Result<()> {
        let current_time = Clock::get()?.unix_timestamp;
        let validation = if is_transfer_allowed(current_time) {
            OracleValidation::V1 {
                transfer: ExternalValidationResult::Approved,
                create: ExternalValidationResult::Pass,
                update: ExternalValidationResult::Pass,
                burn: ExternalValidationResult::Pass,
            }
        } else {
            OracleValidation::V1 {
                transfer: ExternalValidationResult::Rejected,
                create: ExternalValidationResult::Pass,
                update: ExternalValidationResult::Pass,
                burn: ExternalValidationResult::Pass,
            }
        };

        self.oracle.set_inner(Oracle {
            validation,
            bump: bumps.oracle,
            vault_bump: bumps.vault,
        });

        // Fund reward vault for crank rewards 
        let rent = Rent::get()?;
        let vault_lamports = rent.minimum_balance(0) + REWARD_IN_LAMPORTS * 10; // 10 rewards
        transfer(
            CpiContext::new(
                self.system_program.to_account_info(),
                Transfer {
                    from: self.payer.to_account_info(),
                    to: self.vault.to_account_info(),
                },
            ),
            vault_lamports,
        )?;

        Ok(())
    }
}
