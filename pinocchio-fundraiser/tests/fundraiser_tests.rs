mod common;

use common::*;
use litesvm::LiteSVM;
use litesvm::types::TransactionMetadata;
use solana_address::Address;
use solana_keypair::Keypair;
use solana_message::Message;
use solana_native_token::LAMPORTS_PER_SOL;
use solana_signer::Signer;
use solana_transaction::Transaction;

fn run_initialize(
    svm: &mut LiteSVM,
    maker: &Keypair,
    amount_to_raise: u64,
    duration: u8,
) -> (TransactionMetadata, Address, Address, Address, Address) {
    let mint = create_mint(svm, maker, 6, &maker.pubkey());
    let (fundraiser_pda, _) = fundraiser_pda(&maker.pubkey(), &PROGRAM_ID);
    let vault = get_ata(&fundraiser_pda, &mint);
    let maker_ata = get_ata(&maker.pubkey(), &mint);

    create_ata(svm, maker, &mint, &maker.pubkey());
    create_ata(svm, maker, &mint, &fundraiser_pda);

    let ix = ix_initialize(maker, mint, amount_to_raise, duration);
    let msg = Message::new(&[ix], Some(&maker.pubkey()));
    let tx = Transaction::new(&[maker], msg, svm.latest_blockhash());
    let meta = svm.send_transaction(tx).expect("Initialize failed");
    (meta, mint, fundraiser_pda, vault, maker_ata)
}

fn run_contribute(
    svm: &mut LiteSVM,
    maker: &Keypair,
    contributor: &Keypair,
) -> TransactionMetadata {
    let amount_to_raise = 100_000u64;
    let duration = 30u8;

    let (_, mint, fundraiser_pda, vault, _) = run_initialize(svm, maker, amount_to_raise, duration);

    svm.airdrop(&contributor.pubkey(), LAMPORTS_PER_SOL)
        .expect("airdrop contributor");

    let contributor_ata = create_ata(svm, contributor, &mint, &contributor.pubkey());
    let amount = 10_000u64;

    mint_tokens(svm, maker, &mint, &contributor_ata, amount);

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

/// Init + enough contributes to meet target + check (maker claims).
fn run_check(svm: &mut LiteSVM, maker: &Keypair) -> TransactionMetadata {
    let amount_to_raise = 100_000u64;
    let max_per_contributor = 10_000u64; // 10%
    let duration = 30u8;

    let (_, mint, fundraiser_pda, vault, maker_ata) =
        run_initialize(svm, maker, amount_to_raise, duration);

    let num_contributors = 10;
    for i in 0..num_contributors {
        let contributor = Keypair::new();
        svm.airdrop(&contributor.pubkey(), LAMPORTS_PER_SOL)
            .expect("airdrop contributor");
        let contributor_ata = create_ata(svm, &contributor, &mint, &contributor.pubkey());
        mint_tokens(svm, maker, &mint, &contributor_ata, max_per_contributor);

        let ix = ix_contribute(
            &contributor,
            fundraiser_pda,
            contributor_ata,
            vault,
            mint,
            max_per_contributor,
        );
        let msg = Message::new(&[ix], Some(&contributor.pubkey()));
        let tx = Transaction::new(&[&contributor], msg, svm.latest_blockhash());
        svm.send_transaction(tx)
            .expect(&format!("Contribute {} failed", i));
    }

    let ix = ix_check(maker, maker_ata, fundraiser_pda, vault);
    let msg = Message::new(&[ix], Some(&maker.pubkey()));
    let tx = Transaction::new(&[maker], msg, svm.latest_blockhash());
    svm.send_transaction(tx).expect("Check failed")
}

/// Init + contribute (below target) + refund (contributor reclaims).
fn run_refund(svm: &mut LiteSVM, maker: &Keypair, contributor: &Keypair) -> TransactionMetadata {
    let amount_to_raise = 100_000u64;
    let duration = 30u8;

    let (_, mint, fundraiser_pda, vault, _) = run_initialize(svm, maker, amount_to_raise, duration);

    svm.airdrop(&contributor.pubkey(), LAMPORTS_PER_SOL)
        .expect("airdrop contributor");

    let contributor_ata = create_ata(svm, contributor, &mint, &contributor.pubkey());
    let amount = 10_000u64;

    mint_tokens(svm, maker, &mint, &contributor_ata, amount);

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
    svm.send_transaction(tx).expect("Contribute failed");

    let (contributor_account, _) =
        contributor_pda(&fundraiser_pda, &contributor.pubkey(), &PROGRAM_ID);

    let ix = ix_refund(
        contributor,
        maker.pubkey(),
        fundraiser_pda,
        contributor_account,
        contributor_ata,
        vault,
    );
    let msg = Message::new(&[ix], Some(&contributor.pubkey()));
    let tx = Transaction::new(&[contributor], msg, svm.latest_blockhash());
    svm.send_transaction(tx).expect("Refund failed")
}

#[test]
fn test_initialize() {
    let (mut svm, maker) = setup();
    let (meta, _, _, _, _) = run_initialize(&mut svm, &maker, 100_000, 30);
    println!("Initialize CU: {}", meta.compute_units_consumed);
}

#[test]
fn test_contribute() {
    let (mut svm, maker) = setup();
    let contributor = Keypair::new();
    let meta = run_contribute(&mut svm, &maker, &contributor);
    println!("Contribute CU: {}", meta.compute_units_consumed);
}

#[test]
fn test_check() {
    let (mut svm, maker) = setup();
    let meta = run_check(&mut svm, &maker);
    println!("Check CU: {}", meta.compute_units_consumed);
}

#[test]
fn test_refund() {
    let (mut svm, maker) = setup();
    let contributor = Keypair::new();
    let meta = run_refund(&mut svm, &maker, &contributor);
    println!("Refund CU: {}", meta.compute_units_consumed);
}
