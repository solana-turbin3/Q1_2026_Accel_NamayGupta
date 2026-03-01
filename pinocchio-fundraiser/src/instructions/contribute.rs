use pinocchio::{
    AccountView, ProgramResult,
    cpi::{Seed, Signer},
    error::ProgramError,
    sysvars::{Sysvar, clock::Clock, rent::Rent},
};
use pinocchio_pubkey::derive_address;
use pinocchio_system::instructions::CreateAccount;
use wincode::SchemaRead;

use crate::{
    constants::{MAX_CONTRIBUTION_PERCENTAGE, PERCENTAGE_SCALER, SECONDS_TO_DAYS},
    state::{contributor::Contributor, fundraiser::Fundraiser},
};

#[derive(SchemaRead)]
struct ContributeData {
    pub amount: [u8; 8],
    pub contributor_bump: u8,
}

pub fn process_contribute_instruction(accounts: &[AccountView], data: &[u8]) -> ProgramResult {
    let [
        contributor,
        fundraiser,
        contributor_account,
        contributor_ata,
        vault,
        mint,
        _remaining @ ..,
    ] = accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    assert!(contributor.is_signer());

    unsafe {
        assert!(fundraiser.owner() == &crate::ID, "invalid fundraiser owner");
    }

    let ix_data = wincode::deserialize::<ContributeData>(data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    let mint_state = pinocchio_token::state::Mint::from_account_view(mint)?;
    let amount = u64::from_le_bytes(ix_data.amount);
    assert!(amount >= 1_u8.pow(mint_state.decimals() as u32) as u64);

    let fundraiser_state = Fundraiser::from_account_info(fundraiser)?;
    let amount_to_raise = u64::from_le_bytes(fundraiser_state.amount_to_raise);
    let max_per_contributor = (amount_to_raise * MAX_CONTRIBUTION_PERCENTAGE) / PERCENTAGE_SCALER;

    // Validate vault: mint matches fundraiser, authority is fundraiser PDA
    {
        let vault_data = vault.try_borrow()?;
        assert!(vault_data.len() >= 64);
        assert!(&vault_data[0..32] == &fundraiser_state.mint_to_raise);
        assert!(&vault_data[32..64] == fundraiser.address().as_array());
    }

    // Validate contributor_ata: mint matches fundraiser, authority is contributor
    {
        let ata_data = contributor_ata.try_borrow()?;
        assert!(ata_data.len() >= 64);
        assert!(&ata_data[0..32] == &fundraiser_state.mint_to_raise);
        assert!(&ata_data[32..64] == contributor.address().as_array());
    }

    assert!(amount <= max_per_contributor);

    // Fundraiser must still be active
    let current_time = Clock::get()?.unix_timestamp;
    let time_started = i64::from_le_bytes(fundraiser_state.time_started);
    let elapsed_days = ((current_time - time_started) / SECONDS_TO_DAYS) as u8;
    assert!(elapsed_days < fundraiser_state.duration, "fundraiser ended");

    // Validate contributor_account PDA
    let contributor_bump = ix_data.contributor_bump;
    let pda_seeds = [
        b"contributor".as_ref(),
        fundraiser.address().as_ref(),
        contributor.address().as_ref(),
        &[contributor_bump],
    ];
    let expected_pda = derive_address(&pda_seeds, None, &crate::ID.to_bytes());
    assert_eq!(expected_pda, *contributor_account.address().as_array());

    let bump = [contributor_bump];
    let seed = [
        Seed::from(b"contributor"),
        Seed::from(fundraiser.address().as_array()),
        Seed::from(contributor.address().as_array()),
        Seed::from(&bump),
    ];
    let signer = Signer::from(&seed);
    // Create contributor_account if it doesn't exist
    let is_new = unsafe { contributor_account.owner() != &crate::ID };
    if is_new {
        CreateAccount {
            from: contributor,
            to: contributor_account,
            lamports: Rent::get()?.try_minimum_balance(Contributor::LEN)?,
            space: Contributor::LEN as u64,
            owner: &crate::ID,
        }
        .invoke_signed(&[signer])?;
    }
    let contributor_state = Contributor::from_account_info(contributor_account)?;
    let already_contributed = u64::from_le_bytes(contributor_state.amount);

    //First time contrbutor_account is created we populate the bump
    if is_new {
        contributor_state.bump = contributor_bump;
    }

    assert!(already_contributed + amount <= max_per_contributor);

    pinocchio_token::instructions::Transfer {
        from: contributor_ata,
        to: vault,
        authority: contributor,
        amount,
    }
    .invoke()?;

    let current = u64::from_le_bytes(fundraiser_state.current_amount);
    fundraiser_state.current_amount = (current + amount).to_le_bytes();

    contributor_state.amount = (already_contributed + amount).to_le_bytes();

    Ok(())
}
