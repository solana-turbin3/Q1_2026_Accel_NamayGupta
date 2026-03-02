use {
    litesvm::LiteSVM,
    litesvm_token::{CreateAssociatedTokenAccount, CreateMint, MintTo, spl_token},
    solana_address::{Address, address},
    solana_keypair::Keypair,
    solana_native_token::LAMPORTS_PER_SOL,
    solana_signer::Signer,
    std::path::PathBuf,
};

pub static PROGRAM_ID: Address = address!("Ee7GRKhLxqXGPZ9YtR88us4Ni5sHYMFsejFYaresHQDM");
const TOKEN_PROGRAM_ID: Address = spl_token::ID;
/// Creates LiteSVM, airdrops to payer, loads the program. Run `cargo build-sbf` first.
pub fn setup() -> (LiteSVM, Keypair) {
    let mut svm = LiteSVM::new();
    let payer = Keypair::new();

    svm.airdrop(&payer.pubkey(), 10 * LAMPORTS_PER_SOL)
        .expect("Failed to airdrop SOL to payer");

    let so_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/deploy/pinocchio_fundraiser.so");
    let program_data = std::fs::read(&so_path).unwrap_or_else(|_| {
        panic!(
            "Failed to read program SO file. Run `cargo build-sbf` first. Expected: {}",
            so_path.display()
        )
    });

    svm.add_program(PROGRAM_ID, &program_data)
        .expect("Failed to add program");

    (svm, payer)
}

/// Derives the fundraiser PDA for a given maker.
pub fn fundraiser_pda(maker: &Address, program_id: &Address) -> (Address, u8) {
    Address::find_program_address(&[b"fundraiser".as_ref(), maker.as_ref()], program_id)
}

/// Derives the contributor PDA for a given fundraiser and contributor.
pub fn contributor_pda(
    fundraiser: &Address,
    contributor: &Address,
    program_id: &Address,
) -> (Address, u8) {
    Address::find_program_address(
        &[
            b"contributor".as_ref(),
            fundraiser.as_ref(),
            contributor.as_ref(),
        ],
        program_id,
    )
}

/// Gets the associated token address for a wallet and mint.
pub fn get_ata(wallet: &Address, mint: &Address) -> Address {
    spl_associated_token_account_interface::address::get_associated_token_address_with_program_id(
        wallet,
        mint,
        &TOKEN_PROGRAM_ID,
    )
}

/// Creates a mint with given decimals and authority. Returns the mint address.
pub fn create_mint(
    svm: &mut LiteSVM,
    payer: &Keypair,
    decimals: u8,
    authority: &Address,
) -> Address {
    CreateMint::new(svm, payer)
        .decimals(decimals)
        .authority(authority)
        .send()
        .expect("Failed to create mint")
}

/// Creates an ATA for owner and mint. Returns the ATA address.
pub fn create_ata(svm: &mut LiteSVM, payer: &Keypair, mint: &Address, owner: &Address) -> Address {
    CreateAssociatedTokenAccount::new(svm, payer, mint)
        .owner(owner)
        .send()
        .expect("Failed to create ATA")
}

/// Mints tokens to an ATA.
pub fn mint_tokens(
    svm: &mut LiteSVM,
    minter: &Keypair,
    mint: &Address,
    ata: &Address,
    amount: u64,
) {
    MintTo::new(svm, minter, mint, ata, amount)
        .owner(minter)
        .send()
        .expect("Failed to mint tokens");
}
