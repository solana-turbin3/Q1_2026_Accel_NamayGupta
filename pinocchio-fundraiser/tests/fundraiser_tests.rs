mod common;

use common::*;
use litesvm::LiteSVM;
use litesvm::types::TransactionMetadata;
use solana_keypair::Keypair;
use solana_message::Message;
use solana_native_token::LAMPORTS_PER_SOL;
use solana_signer::Signer;
use solana_transaction::Transaction;

fn run_initialize(svm: &mut LiteSVM, maker: &Keypair) -> TransactionMetadata {
    let mint = create_mint(svm, maker, 6, &maker.pubkey());
    let (fundraiser_pda, _bump) = fundraiser_pda(&maker.pubkey(), &PROGRAM_ID);
    let vault = get_ata(&fundraiser_pda, &mint);

    create_ata(svm, maker, &mint, &maker.pubkey()); //maker_ata
    create_ata(svm, maker, &mint, &fundraiser_pda); //vault

    let amount_to_raise = 1_000_000u64; // 1 token with 6 decimals
    let duration = 30u8;

    let ix = ix_initialize(maker, mint, amount_to_raise, duration);
    let msg = Message::new(&[ix], Some(&maker.pubkey()));
    let tx = Transaction::new(&[maker], msg, svm.latest_blockhash());
    svm.send_transaction(tx).expect("Initialize failed")
}
fn run_contribute(
    svm: &mut LiteSVM,
    maker: &Keypair,
    contributor: &Keypair,
) -> TransactionMetadata {
    let mint = create_mint(svm, maker, 6, &maker.pubkey());
    let (fundraiser_pda, _bump) = fundraiser_pda(&maker.pubkey(), &PROGRAM_ID);
    let vault = get_ata(&fundraiser_pda, &mint);

    let _maker_ata = create_ata(svm, maker, &mint, &maker.pubkey());
    create_ata(svm, maker, &mint, &fundraiser_pda); // vault ATA

    let amount_to_raise = 1_000_000u64;
    let duration = 30u8;

    let init_ix = ix_initialize(maker, mint, amount_to_raise, duration);
    let init_msg = Message::new(&[init_ix], Some(&maker.pubkey()));
    let init_tx = Transaction::new(&[maker], init_msg, svm.latest_blockhash());
    svm.send_transaction(init_tx).expect("initialize failed");

    // 2) Prepare contributor: airdrop SOL + create ATA + mint tokens
    svm.airdrop(&contributor.pubkey(), LAMPORTS_PER_SOL)
        .expect("airdrop contributor");

    let contributor_ata = create_ata(svm, contributor, &mint, &contributor.pubkey());
    let amount = 40_000u64; // contribution amount

    mint_tokens(svm, maker, &mint, &contributor_ata, amount);

    // 3) Call contribute
    let ix = ix_contribute(
        contributor,
        fundraiser_pda,
        contributor_ata,
        vault,
        mint,
        amount,
    );
    let msg = Message::new(&[ix], Some(&contributor.pubkey()));
    let tx = Transaction::new(&[contributor], msg, svm.latest_blockhash());
    svm.send_transaction(tx).expect("Contribute failed")
}

// #[test]
// fn test_initialize() {
//     let (mut svm, maker) = setup();
//     let signature = run_initialize(&mut svm, &maker);
//     println!(
//         "Compute units consumed: {}",
//         signature.compute_units_consumed
//     );
// }

#[test]
fn test_contribute() {
    let (mut svm, maker) = setup();
    let contributor = Keypair::new();
    let signature = run_contribute(&mut svm, &maker, &contributor);
    println!(
        "Compute units consumed: {}",
        signature.compute_units_consumed
    );
}
