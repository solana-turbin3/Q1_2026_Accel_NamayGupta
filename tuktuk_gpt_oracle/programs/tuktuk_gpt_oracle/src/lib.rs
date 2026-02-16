use anchor_lang::prelude::*;
mod instructions;
use instructions::*;

mod state;

declare_id!("DvdmHN2XpPSzSNE8y3ieb13Emr6irm6gmHjEDe3mbe4g");

#[program]
pub mod tuktuk_gpt_oracle {
    use super::*;

    const AGENT_DESC: &str = "You maybe a helpful assistant.";

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        ctx.accounts.initialize(AGENT_DESC.to_string())
    }

    pub fn interact_agent(ctx: Context<InteractAgent>, text: String) -> Result<()> {
        ctx.accounts.interact_agent(text)
    }

    pub fn callback_from_agent(ctx: Context<CallbackFromAgent>, response: String) -> Result<()> {
        ctx.accounts.callback_from_agent(response)
    }
    pub fn schedule(ctx: Context<Schedule>, text: String, task_id: u16) -> Result<()> {
        ctx.accounts.schedule(text, task_id, &ctx.bumps)
    }
}
