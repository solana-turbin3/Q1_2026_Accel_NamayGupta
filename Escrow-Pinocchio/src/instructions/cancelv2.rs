use pinocchio::{
    cpi::{Seed, Signer},
    error::ProgramError,
    AccountView, ProgramResult,
};
use pinocchio_pubkey::derive_address;
use wincode::{SchemaRead, SchemaWrite};

use crate::state::Escrow;

#[derive(SchemaRead, SchemaWrite)]
pub(crate) struct CancelV2Data {
    pub bump: u8,
}

pub fn process_cancel_v2_instruction(accounts: &[AccountView], data: &[u8]) -> ProgramResult {
    let [maker, mint_a, maker_ata, escrow_account, escrow_ata, _system_program, _token_program, _associated_token_program @ ..] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    let ix_data = wincode::deserialize::<CancelV2Data>(data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    let escrow_account_state = Escrow::from_account_info(&escrow_account)?;

    if escrow_account_state.maker() != *maker.address() {
        return Err(ProgramError::IllegalOwner);
    }
    if escrow_account_state.mint_a() != *mint_a.address() {
        return Err(ProgramError::InvalidAccountData);
    }

    let escrow_pda = derive_address(
        &[
            b"escrow".as_ref(),
            maker.address().as_ref(),
            &[ix_data.bump],
        ],
        None,
        &crate::ID.to_bytes(),
    );
    assert_eq!(escrow_pda, *escrow_account.address().as_array());
    let bump = escrow_account_state.bump;

    let amount_to_give = escrow_account_state.amount_to_give();
    let bump = [bump];
    let seed = [
        Seed::from(b"escrow"),
        Seed::from(maker.address().as_array()),
        Seed::from(&bump),
    ];
    let seeds = [Signer::from(&seed)];

    pinocchio_token::instructions::Transfer {
        from: escrow_ata,
        to: maker_ata,
        authority: escrow_account,
        amount: amount_to_give,
    }
    .invoke_signed(&seeds)?;
    pinocchio_token::instructions::CloseAccount {
        account: escrow_ata,
        destination: maker,
        authority: escrow_account,
    }
    .invoke_signed(&seeds)?;
    maker.set_lamports(escrow_account.lamports() + maker.lamports());
    escrow_account.set_lamports(0);
    Ok(())
}
