use pinocchio::{
    cpi::{Seed, Signer},
    error::ProgramError,
    AccountView, ProgramResult,
};
use pinocchio_pubkey::derive_address;

use crate::state::Escrow;

pub fn process_take_instruction(accounts: &[AccountView], data: &[u8]) -> ProgramResult {
    let [maker, taker, mint_a, mint_b, escrow_account, maker_ata_b, taker_ata_b, taker_ata_a, escrow_ata, system_program, token_program, _associated_token_program @ ..] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    let (bump, amount_to_give, amount_to_receive) = {
        let escrow_account_state = Escrow::from_account_info(&escrow_account)?;
        if escrow_account_state.maker() != *maker.address() {
            return Err(ProgramError::IllegalOwner);
        }
        if escrow_account_state.mint_a() != *mint_a.address() {
            return Err(ProgramError::InvalidAccountData);
        }
        if escrow_account_state.mint_b() != *mint_b.address() {
            return Err(ProgramError::InvalidAccountData);
        }
        let escrow_pda = derive_address(
            &[b"escrow".as_ref(), maker.address().as_ref(), &[data[0]]],
            None,
            &crate::ID.to_bytes(),
        );
        assert_eq!(escrow_pda, *escrow_account.address().as_array());
        (
            escrow_account_state.bump,
            escrow_account_state.amount_to_give(),
            escrow_account_state.amount_to_receive(),
        )
    };

    {
        let maker_state_b = pinocchio_token::state::TokenAccount::from_account_view(&maker_ata_b)?;
        if maker_state_b.owner() != maker.address() {
            return Err(ProgramError::IllegalOwner);
        }
        if maker_state_b.mint() != mint_b.address() {
            return Err(ProgramError::InvalidAccountData);
        }
    }
    {
        let taker_state_b = pinocchio_token::state::TokenAccount::from_account_view(&taker_ata_b)?;
        if taker_state_b.owner() != taker.address() {
            return Err(ProgramError::IllegalOwner);
        }
        if taker_state_b.mint() != mint_b.address() {
            return Err(ProgramError::InvalidAccountData);
        }
    }
    {
        let taker_state_a = pinocchio_token::state::TokenAccount::from_account_view(&taker_ata_a)?;
        if taker_state_a.owner() != taker.address() {
            return Err(ProgramError::IllegalOwner);
        }
        if taker_state_a.mint() != mint_a.address() {
            return Err(ProgramError::InvalidAccountData);
        }
    }

    let bump = [bump.to_le()];
    let seed = [
        Seed::from(b"escrow"),
        Seed::from(maker.address().as_array()),
        Seed::from(&bump),
    ];
    let seeds = [Signer::from(&seed)];

    pinocchio_token::instructions::Transfer {
        from: escrow_ata,
        to: taker_ata_a,
        authority: escrow_account,
        amount: amount_to_give,
    }
    .invoke_signed(&seeds)?;
    pinocchio_token::instructions::Transfer {
        from: taker_ata_b,
        to: maker_ata_b,
        authority: taker,
        amount: amount_to_receive,
    }
    .invoke()?;

    pinocchio_token::instructions::CloseAccount {
        account: escrow_ata,
        destination: maker,
        authority: escrow_account,
    }
    .invoke_signed(&seeds)?;

    maker.set_lamports(escrow_account.lamports() + maker.lamports());
    escrow_account.set_lamports(0);
    // escrow_account.close()?;
    Ok(())
}
