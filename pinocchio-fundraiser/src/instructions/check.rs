use pinocchio::{
    AccountView, ProgramResult,
    cpi::{Seed, Signer},
    error::ProgramError,
};
use pinocchio_pubkey::derive_address;

use crate::state::fundraiser::Fundraiser;

pub fn process_check_instruction(accounts: &[AccountView], _data: &[u8]) -> ProgramResult {
    let [maker, maker_ata, fundraiser, vault, _remaining @ ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    assert!(maker.is_signer(), "maker must be signer");
    unsafe {
        assert!(fundraiser.owner() == &crate::ID, "invalid fundraiser owner");
    }

    let fundraiser_state = Fundraiser::from_account_info(fundraiser)?;

    let bump = fundraiser_state.bump;
    let seed = [b"fundraiser".as_ref(), maker.address().as_ref(), &[bump]];

    let fundraiser_account_pda = derive_address(&seed, None, &crate::ID.to_bytes());
    assert_eq!(fundraiser_account_pda, *fundraiser.address().as_array());

    // Maker must match the one stored in fundraiser
    assert!(fundraiser_state.maker == *maker.address().as_array());

    // Validate maker_ata: mint matches fundraiser, authority is maker
    {
        let ata_data = maker_ata.try_borrow()?;
        assert!(ata_data.len() >= 64);
        assert!(&ata_data[0..32] == &fundraiser_state.mint_to_raise);
        assert!(&ata_data[32..64] == maker.address().as_array());
    }
    let vault_amount = {
        let vault_data = vault.try_borrow()?;
        assert!(vault_data.len() >= 72);
        assert!(&vault_data[0..32] == &fundraiser_state.mint_to_raise);
        assert!(&vault_data[32..64] == fundraiser.address().as_array());
        u64::from_le_bytes(vault_data[64..72].try_into().unwrap())
    };
    let amount_to_raise = u64::from_le_bytes(fundraiser_state.amount_to_raise);
    assert!(vault_amount >= amount_to_raise, "target not met");

    let bump = fundraiser_state.bump;
    let bump_bytes = [bump];
    let seed = [
        Seed::from(b"fundraiser"),
        Seed::from(&fundraiser_state.maker),
        Seed::from(&bump_bytes),
    ];
    let signer = Signer::from(&seed);

    pinocchio_token::instructions::Transfer {
        from: vault,
        to: maker_ata,
        authority: fundraiser,
        amount: vault_amount,
    }
    .invoke_signed(&[signer])?;

    maker.set_lamports(fundraiser.lamports() + maker.lamports());
    fundraiser.set_lamports(0);
    Ok(())
}
