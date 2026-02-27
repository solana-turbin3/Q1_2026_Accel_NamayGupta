#[cfg(test)]
mod tests {

    use litesvm::LiteSVM;
    use litesvm_token::{spl_token, CreateAssociatedTokenAccount, CreateMint, MintTo};
    use solana_address::{address, Address};
    use solana_instruction::{AccountMeta, Instruction};
    use solana_keypair::Keypair;
    use solana_message::Message;
    use solana_native_token::LAMPORTS_PER_SOL;
    use solana_signer::Signer;
    use solana_transaction::Transaction;
    use std::path::PathBuf;

    pub static PROGRAM_ID: Address = address!("4ibrEMW5F6hKnkW4jVedswYv6H6VtwPN6ar6dvXDN1mT");
    const TOKEN_PROGRAM_ID: Address = spl_token::ID;
    const ASSOCIATED_TOKEN_PROGRAM_ID: Address =
        spl_associated_token_account_interface::program::ID;

    /// State returned after a successful Make instruction. Reused for Cancel and Take tests.
    pub struct MakeState {
        pub mint_a: Address,
        pub mint_b: Address,
        pub maker_ata_a: Address,
        pub escrow_pda: Address,
        pub vault: Address,
        pub bump: u8,
        pub amount_to_receive: u64,
        pub amount_to_give: u64,
    }

    fn setup() -> (LiteSVM, Keypair) {
        let mut svm = LiteSVM::new();
        let payer = Keypair::new();

        svm.airdrop(&payer.pubkey(), 10 * LAMPORTS_PER_SOL)
            .expect("Airdrop failed");

        // Load program SO file (run `cargo build-sbf` first)
        let so_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target/sbpf-solana-solana/release/escrow.so");

        let program_data = std::fs::read(so_path).expect("Failed to read program SO file");

        svm.add_program(PROGRAM_ID, &program_data)
            .expect("Failed to add program");

        (svm, payer)
    }

    /// Runs the Make instruction: creates mints, ATAs, mints tokens, and deposits into escrow.
    /// Returns state needed for Cancel and Take tests.
    fn run_make(svm: &mut LiteSVM, maker: &Keypair) -> MakeState {
        let mint_a = CreateMint::new(svm, maker)
            .decimals(6)
            .authority(&maker.pubkey())
            .send()
            .unwrap();
        let mint_b = CreateMint::new(svm, maker)
            .decimals(6)
            .authority(&maker.pubkey())
            .send()
            .unwrap();

        let maker_ata_a = CreateAssociatedTokenAccount::new(svm, maker, &mint_a)
            .owner(&maker.pubkey())
            .send()
            .unwrap();

        let (escrow_pda, bump) = Address::find_program_address(
            &[b"escrow".as_ref(), maker.pubkey().as_ref()],
            &PROGRAM_ID,
        );
        let vault = spl_associated_token_account_interface::address::get_associated_token_address_with_program_id(
            &escrow_pda,
            &mint_a,
            &TOKEN_PROGRAM_ID,
        );

        MintTo::new(svm, maker, &mint_a, &maker_ata_a, 1000000000)
            .owner(maker)
            .send()
            .unwrap();

        let amount_to_receive = 100000000u64;
        let amount_to_give = 500000000u64;

        let make_data = [
            vec![0u8],
            vec![bump],
            amount_to_receive.to_le_bytes().to_vec(),
            amount_to_give.to_le_bytes().to_vec(),
        ]
        .concat();

        let make_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(maker.pubkey(), true),
                AccountMeta::new(mint_a, false),
                AccountMeta::new(mint_b, false),
                AccountMeta::new(escrow_pda, false),
                AccountMeta::new(maker_ata_a, false),
                AccountMeta::new(vault, false),
                AccountMeta::new(solana_sdk_ids::system_program::ID, false),
                AccountMeta::new(TOKEN_PROGRAM_ID, false),
                AccountMeta::new(ASSOCIATED_TOKEN_PROGRAM_ID, false),
            ],
            data: make_data,
        };

        let message = Message::new(&[make_ix], Some(&maker.pubkey()));
        let tx = Transaction::new(&[maker], message, svm.latest_blockhash());
        svm.send_transaction(tx).expect("Make instruction failed");

        MakeState {
            mint_a,
            mint_b,
            maker_ata_a,
            escrow_pda,
            vault,
            bump,
            amount_to_receive,
            amount_to_give,
        }
    }

    #[test]
    fn test_make_instruction() {
        let (mut svm, maker) = setup();
        let _state = run_make(&mut svm, &maker);
        println!("Make OK");
    }

    #[test]
    fn test_cancel_instruction() {
        let (mut svm, maker) = setup();
        let state = run_make(&mut svm, &maker);

        let cancel_data = [vec![2u8], vec![state.bump]].concat();
        let cancel_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(maker.pubkey(), true),
                AccountMeta::new(state.mint_a, false),
                AccountMeta::new(state.maker_ata_a, false),
                AccountMeta::new(state.escrow_pda, false),
                AccountMeta::new(state.vault, false),
                AccountMeta::new(solana_sdk_ids::system_program::ID, false),
                AccountMeta::new(TOKEN_PROGRAM_ID, false),
                AccountMeta::new(ASSOCIATED_TOKEN_PROGRAM_ID, false),
            ],
            data: cancel_data,
        };

        let msg = Message::new(&[cancel_ix], Some(&maker.pubkey()));
        let tx = Transaction::new(&[maker], msg, svm.latest_blockhash());
        svm.send_transaction(tx).expect("Cancel instruction failed");
        println!("Cancel OK");
    }

    #[test]
    fn test_take_instruction() {
        let (mut svm, maker) = setup();
        let state = run_make(&mut svm, &maker);

        let taker = Keypair::new();
        svm.airdrop(&taker.pubkey(), LAMPORTS_PER_SOL)
            .expect("Airdrop taker");

        let maker_ata_b = CreateAssociatedTokenAccount::new(&mut svm, &maker, &state.mint_b)
            .owner(&maker.pubkey())
            .send()
            .unwrap();
        let taker_ata_b = CreateAssociatedTokenAccount::new(&mut svm, &taker, &state.mint_b)
            .owner(&taker.pubkey())
            .send()
            .unwrap();
        let taker_ata_a = CreateAssociatedTokenAccount::new(&mut svm, &taker, &state.mint_a)
            .owner(&taker.pubkey())
            .send()
            .unwrap();

        MintTo::new(
            &mut svm,
            &maker,
            &state.mint_b,
            &taker_ata_b,
            state.amount_to_receive,
        )
        .owner(&maker)
        .send()
        .unwrap();

        let take_data = [vec![1u8], vec![state.bump]].concat();
        let take_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(maker.pubkey(), false),
                AccountMeta::new(taker.pubkey(), true),
                AccountMeta::new(state.mint_a, false),
                AccountMeta::new(state.mint_b, false),
                AccountMeta::new(state.escrow_pda, false),
                AccountMeta::new(maker_ata_b, false),
                AccountMeta::new(taker_ata_b, false),
                AccountMeta::new(taker_ata_a, false),
                AccountMeta::new(state.vault, false),
                AccountMeta::new(solana_sdk_ids::system_program::ID, false),
                AccountMeta::new(TOKEN_PROGRAM_ID, false),
                AccountMeta::new(ASSOCIATED_TOKEN_PROGRAM_ID, false),
            ],
            data: take_data,
        };

        let msg = Message::new(&[take_ix], Some(&taker.pubkey()));
        let tx = Transaction::new(&[&taker], msg, svm.latest_blockhash());
        svm.send_transaction(tx).expect("Take instruction failed");
        println!("Take OK");
    }
}
