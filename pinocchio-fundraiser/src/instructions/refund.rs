use crate::constants::SECONDS_TO_DAYS;
use crate::state::{contributor::Contributor, fundraiser::Fundraiser};
use pinocchio::{
    AccountView, ProgramResult,
    cpi::{Seed, Signer},
    error::ProgramError,
    sysvars::{Sysvar, clock::Clock},
};
use pinocchio_pubkey::derive_address;

pub fn process_refund_instruction(accounts: &[AccountView], _data: &[u8]) -> ProgramResult {
    let [
        contributor,
        maker,
        fundraiser,
        contributor_account,
        contributor_ata,
        vault,
        _remaining @ ..,
    ] = accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    assert!(contributor.is_signer());
    let fundraiser_state = Fundraiser::from_account_info(fundraiser)?;

    let bump = fundraiser_state.bump;
    let seed = [b"fundraiser".as_ref(), maker.address().as_ref(), &[bump]];

    let fundraiser_account_pda = derive_address(&seed, None, &crate::ID.to_bytes());
    assert_eq!(fundraiser_account_pda, *fundraiser.address().as_array());

    // Maker must match the one stored in fundraiser
    assert!(fundraiser_state.maker == *maker.address().as_array());

    // Validate contributor_ata: mint matches fundraiser, authority is contributor
    {
        let ata_data = contributor_ata.try_borrow()?;
        assert!(ata_data.len() >= 64);
        assert!(&ata_data[0..32] == &fundraiser_state.mint_to_raise);
        assert!(&ata_data[32..64] == contributor.address().as_array());
    }
    let vault_amount = {
        let vault_data = vault.try_borrow()?;
        assert!(vault_data.len() >= 72);
        assert!(&vault_data[0..32] == &fundraiser_state.mint_to_raise);
        assert!(&vault_data[32..64] == fundraiser.address().as_array());
        u64::from_le_bytes(vault_data[64..72].try_into().unwrap())
    };

    let current_time = Clock::get()?.unix_timestamp;
    let time_started = i64::from_le_bytes(fundraiser_state.time_started);
    let elapsed_days = ((current_time - time_started) / SECONDS_TO_DAYS) as u8;
    assert!(
        fundraiser_state.duration >= elapsed_days,
        "fundraiser not ended"
    );
    let amount_to_raise = u64::from_le_bytes(fundraiser_state.amount_to_raise);
    assert!(vault_amount < amount_to_raise, "target met");

    unsafe {
        assert!(
            contributor_account.owner() == &crate::ID,
            "invalid contributor account"
        );
    }

    let contributor_state = Contributor::from_account_info(contributor_account)?;
    let contributor_bump = contributor_state.bump;
    let contributor_pda_seeds = [
        b"contributor".as_ref(),
        fundraiser.address().as_ref(),
        contributor.address().as_ref(),
        &[contributor_bump],
    ];
    let expected_contributor = derive_address(&contributor_pda_seeds, None, &crate::ID.to_bytes());
    assert_eq!(
        expected_contributor,
        *contributor_account.address().as_array()
    );
    let contributed_amount = u64::from_le_bytes(contributor_state.amount);

    let bump = [bump];
    let seed = [
        Seed::from(b"fundraiser"),
        Seed::from(maker.address().as_array()),
        Seed::from(&bump),
    ];
    let seeds = Signer::from(&seed);
    pinocchio_token::instructions::Transfer {
        from: vault,
        to: contributor_ata,
        authority: fundraiser,
        amount: contributed_amount,
    }
    .invoke_signed(&[seeds])?;

    Ok(())
}
