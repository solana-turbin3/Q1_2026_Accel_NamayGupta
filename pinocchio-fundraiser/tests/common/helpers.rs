use super::setup::{PROGRAM_ID, contributor_pda, fundraiser_pda};

use solana_address::Address;
use solana_instruction::{AccountMeta, Instruction};
use solana_keypair::Keypair;
use solana_sdk_ids::system_program;
use solana_signer::Signer;

pub const TOKEN_PROGRAM_ID: Address = litesvm_token::spl_token::ID;
pub const ASSOCIATED_TOKEN_PROGRAM_ID: Address =
    spl_associated_token_account_interface::program::ID;

/// Instruction discriminators (first byte of instruction data).
pub mod disc {
    pub const INITIALIZE: u8 = 0;
    pub const CONTRIBUTE: u8 = 1;
    pub const CHECK: u8 = 2;
    pub const REFUND: u8 = 3;
}

/// Builds the Initialize instruction.
///
/// Accounts: maker (signer), mint, fundraiser
pub fn ix_initialize(
    maker: &Keypair,
    mint: Address,
    amount_to_raise: u64,
    duration: u8,
) -> Instruction {
    let (fundraiser_pda, bump) = fundraiser_pda(&maker.pubkey(), &PROGRAM_ID);

    let data = [
        vec![disc::INITIALIZE],
        vec![bump],
        amount_to_raise.to_le_bytes().to_vec(),
        vec![duration],
    ]
    .concat();

    Instruction {
        program_id: PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(maker.pubkey(), true),
            AccountMeta::new(mint, false),
            AccountMeta::new(fundraiser_pda, false),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
        data,
    }
}

/// Builds the Contribute instruction.
///
/// Accounts: contributor (signer), fundraiser, contributor_account, contributor_ata, vault, mint
pub fn ix_contribute(
    contributor: &Keypair,
    fundraiser: Address,
    contributor_ata: Address,
    vault: Address,
    mint: Address,
    amount: u64,
) -> Instruction {
    let (contributor_account, contributor_bump) =
        contributor_pda(&fundraiser, &contributor.pubkey(), &PROGRAM_ID);

    let data = [
        vec![disc::CONTRIBUTE],
        amount.to_le_bytes().to_vec(),
        vec![contributor_bump],
    ]
    .concat();

    Instruction {
        program_id: PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(contributor.pubkey(), true),
            AccountMeta::new(fundraiser, false),
            AccountMeta::new(contributor_account, false),
            AccountMeta::new(contributor_ata, false),
            AccountMeta::new(vault, false),
            AccountMeta::new(mint, false),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
        ],
        data,
    }
}

/// Builds the Check instruction (claim by maker when target met).
///
/// Accounts: maker (signer), maker_ata, fundraiser, vault
pub fn ix_check(
    maker: &Keypair,
    maker_ata: Address,
    fundraiser: Address,
    vault: Address,
) -> Instruction {
    let data = vec![disc::CHECK];

    Instruction {
        program_id: PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(maker.pubkey(), true),
            AccountMeta::new(maker_ata, false),
            AccountMeta::new(fundraiser, false),
            AccountMeta::new(vault, false),
        ],
        data,
    }
}

/// Builds the Refund instruction (contributor reclaims when fundraiser ended and target not met).
///
/// Accounts: contributor (signer), maker, fundraiser, contributor_account, contributor_ata, vault
pub fn ix_refund(
    contributor: &Keypair,
    maker: Address,
    fundraiser: Address,
    contributor_account: Address,
    contributor_ata: Address,
    vault: Address,
) -> Instruction {
    let data = vec![disc::REFUND];

    Instruction {
        program_id: PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(contributor.pubkey(), true),
            AccountMeta::new(maker, false),
            AccountMeta::new(fundraiser, false),
            AccountMeta::new(contributor_account, false),
            AccountMeta::new(contributor_ata, false),
            AccountMeta::new(vault, false),
        ],
        data,
    }
}
