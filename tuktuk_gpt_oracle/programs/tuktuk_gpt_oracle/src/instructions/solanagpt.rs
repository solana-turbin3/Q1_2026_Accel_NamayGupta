use crate::{state::Agent, ID};
use anchor_lang::prelude::*;
use anchor_lang::Discriminator;
use solana_gpt_oracle::{ContextAccount, Counter, Identity};

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        init,
        payer = payer,
        space = 8 + 32,
        seeds = [b"agent"],
        bump
    )]
    pub agent: Account<'info, Agent>,
    /// CHECK: Checked in oracle program
    #[account(mut)]
    pub llm_context: AccountInfo<'info>,
    #[account(mut)]
    pub counter: Account<'info, Counter>,
    pub system_program: Program<'info, System>,
    /// CHECK: Checked oracle id
    #[account(address = solana_gpt_oracle::ID)]
    pub oracle_program: AccountInfo<'info>,
}
impl<'info> Initialize<'info> {
    pub fn initialize(&mut self, agent_desc: String) -> Result<()> {
        self.agent.context = self.llm_context.key();

        // Create the context for the AI agent
        let cpi_program = self.oracle_program.to_account_info();
        let cpi_accounts = solana_gpt_oracle::cpi::accounts::CreateLlmContext {
            payer: self.payer.to_account_info(),
            counter: self.counter.to_account_info(),
            context_account: self.llm_context.to_account_info(),
            system_program: self.system_program.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        solana_gpt_oracle::cpi::create_llm_context(cpi_ctx, agent_desc)?;

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(text: String)]
pub struct InteractAgent<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    /// CHECK: Checked in oracle program
    #[account(mut)]
    pub interaction: AccountInfo<'info>,
    #[account(seeds = [b"agent"], bump)]
    pub agent: Account<'info, Agent>,
    #[account(address = agent.context)]
    pub context_account: Account<'info, ContextAccount>,
    /// CHECK: Checked oracle id
    #[account(address = solana_gpt_oracle::ID)]
    pub oracle_program: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CallbackFromAgent<'info> {
    /// CHECK: Checked in oracle program
    pub identity: Account<'info, Identity>,
}
impl<'info> InteractAgent<'info> {
    pub fn interact_agent(&mut self, text: String) -> Result<()> {
        let cpi_program = self.oracle_program.to_account_info();
        let cpi_accounts = solana_gpt_oracle::cpi::accounts::InteractWithLlm {
            payer: self.payer.to_account_info(),
            interaction: self.interaction.to_account_info(),
            context_account: self.context_account.to_account_info(),
            system_program: self.system_program.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        let disc: [u8; 8] = crate::instruction::CallbackFromAgent::DISCRIMINATOR
            .try_into()
            .expect("Discriminator must be 8 bytes");
        solana_gpt_oracle::cpi::interact_with_llm(cpi_ctx, text, ID, disc, None)?;

        Ok(())
    }
}

impl<'info> CallbackFromAgent<'info> {
    pub fn callback_from_agent(&mut self, response: String) -> Result<()> {
        // Check if the callback is from the LLM program
        if !self.identity.to_account_info().is_signer {
            return Err(ProgramError::InvalidAccountData.into());
        }
        // Do something with the response
        msg!("Agent Response: {:?}", response);
        Ok(())
    }
}
