#![allow(warnings)]

use {
    borsh::BorshSerialize,
    solana_asset_splitter::{
        error::SplitterError,
        instruction::{
            SplitLamports,
            SplitSplTokensFromMultipleMints,
            SplitSplTokensFromSingleMint,
            SplitterInstruction
        },
        processor::Processor
    },
    solana_program_test::{
        processor,
        tokio,
        BanksClient,
        ProgramTest
    },
    solana_sdk::{
        hash::Hash,
        instruction::{
            AccountMeta,
            Instruction
        },
        native_token::sol_to_lamports,
        pubkey::Pubkey,
        signature::Signer,
        signer::keypair::Keypair,
        transaction::Transaction,
        system_program::ID as SYSTEM_PROGRAM_ID,
        program_error,
        program_pack::Pack,
        system_instruction::{
            transfer as transfer_lamports,
            create_account as create_solana_account
        },
        rent::Rent
    },
    spl_token::{
        state::{
            Account as TokenAccount,
            Mint as MintAccount
        },
        ID as TOKEN_STANDARD_PROGRAM,
        instruction::{
            transfer as transfer_spl_token,
            initialize_mint as initialize_mint_account,
            initialize_account as initialize_token_account,
            mint_to
        }
    }
};

async fn setup(program_id: &Pubkey) -> ProgramTest {
    let program_test = ProgramTest::new(
        "solana_asset_splitter",
        *program_id,
        processor!(Processor::process)
    );

    program_test
}

async fn setup_single_mint(
    banks_client: &mut BanksClient,
    mint_account: &Keypair,
    src_token_account: &Keypair,
    dst_token_account: &Keypair,
    dst_sc_token_account: &Keypair,
    owner: &Keypair,
    recent_blockhash: &Hash
) {
    // 1. create and initialize mint_account
    let ix_1 = create_solana_account(
        &owner.pubkey(),
        &mint_account.pubkey(),
        Rent::default().minimum_balance(MintAccount::LEN),
        MintAccount::LEN as u64,
        &TOKEN_STANDARD_PROGRAM
    );
    let ix_2 = initialize_mint_account(
        &TOKEN_STANDARD_PROGRAM,
        &mint_account.pubkey(),
        &owner.pubkey(),
        None,
        2u8
    ).unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[
            ix_1,
            ix_2
        ],
        Some(&owner.pubkey()),
        &[
            &owner,
            &mint_account
        ],
        *recent_blockhash
    );

    banks_client
        .process_transaction(tx)
        .await
        .unwrap();

    // 2. create and initialize source_token_account
    let ix_1 = create_solana_account(
        &owner.pubkey(),
        &src_token_account.pubkey(),
        Rent::default().minimum_balance(TokenAccount::LEN),
        TokenAccount::LEN as u64,
        &TOKEN_STANDARD_PROGRAM
    );
    let ix_2 = initialize_token_account(
        &TOKEN_STANDARD_PROGRAM,
        &src_token_account.pubkey(),
        &mint_account.pubkey(),
        &owner.pubkey()
    ).unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[
            ix_1,
            ix_2
        ],
        Some(&owner.pubkey()),
        &[
            &owner,
            &src_token_account
        ],
        *recent_blockhash
    );

    banks_client
        .process_transaction(tx)
        .await
        .unwrap();

    // 3. create and initialize destination_token_account
    let ix_1 = create_solana_account(
        &owner.pubkey(),
        &dst_token_account.pubkey(),
        Rent::default().minimum_balance(TokenAccount::LEN),
        TokenAccount::LEN as u64,
        &TOKEN_STANDARD_PROGRAM
    );
    let ix_2 = initialize_token_account(
        &TOKEN_STANDARD_PROGRAM,
        &dst_token_account.pubkey(),
        &mint_account.pubkey(),
        &owner.pubkey()
    ).unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[
            ix_1,
            ix_2
        ],
        Some(&owner.pubkey()),
        &[
            &owner,
            &dst_token_account
        ],
        *recent_blockhash
    );

    banks_client
        .process_transaction(tx)
        .await
        .unwrap();

    // 4. create and initialize destination_second_token_account
    let ix_1 = create_solana_account(
        &owner.pubkey(),
        &dst_sc_token_account.pubkey(),
        Rent::default().minimum_balance(TokenAccount::LEN),
        TokenAccount::LEN as u64,
        &TOKEN_STANDARD_PROGRAM
    );
    let ix_2 = initialize_token_account(
        &TOKEN_STANDARD_PROGRAM,
        &dst_sc_token_account.pubkey(),
        &mint_account.pubkey(),
        &owner.pubkey()
    ).unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[
            ix_1,
            ix_2
        ],
        Some(&owner.pubkey()),
        &[
            &owner,
            &dst_sc_token_account
        ],
        *recent_blockhash
    );

    banks_client
        .process_transaction(tx)
        .await
        .unwrap();

    // 5. mint tokens to source_token_account
    let ix = mint_to(
        &TOKEN_STANDARD_PROGRAM,
        &mint_account.pubkey(),
        &src_token_account.pubkey(),
        &owner.pubkey(),
        &[],
        100_00u64
    ).unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&owner.pubkey()),
        &[&owner],
        *recent_blockhash
    );

    banks_client
        .process_transaction(tx)
        .await
        .unwrap();
}

async fn setup_multiple_mints(
    banks_client: &mut BanksClient,
    mint_one_account: &Keypair,
    mint_two_account: &Keypair,
    source_mint_one_token_account: &Keypair,
    source_mint_two_token_account: &Keypair,
    destination_mint_one_token_account: &Keypair,
    destination_mint_two_token_account: &Keypair,
    owner: &Keypair,
    recent_blockhash: &Hash
) {
    // 1. Create mint-accounts
    // mint-account 1
    let ix_1 = create_solana_account(
        &owner.pubkey(),
        &mint_one_account.pubkey(),
        Rent::default().minimum_balance(MintAccount::LEN),
        MintAccount::LEN as u64,
        &TOKEN_STANDARD_PROGRAM
    );
    let ix_2 = initialize_mint_account(
        &TOKEN_STANDARD_PROGRAM,
        &mint_one_account.pubkey(),
        &owner.pubkey(),
        None,
        3u8
    ).unwrap();

    // mint-account 2
    let ix_3 = create_solana_account(
        &owner.pubkey(),
        &mint_two_account.pubkey(),
        Rent::default().minimum_balance(MintAccount::LEN),
        MintAccount::LEN as u64,
        &TOKEN_STANDARD_PROGRAM
    );
    let ix_4 = initialize_mint_account(
        &TOKEN_STANDARD_PROGRAM,
        &mint_two_account.pubkey(),
        &owner.pubkey(),
        None,
        2u8
    ).unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[
            ix_1,
            ix_2,
            ix_3,
            ix_4
        ],
        Some(&owner.pubkey()),
        &[
            &owner,
            &mint_one_account,
            &mint_two_account
        ],
        *recent_blockhash
    );

    banks_client
        .process_transaction(tx)
        .await
        .unwrap();

    // 2. Create source-token-accounts
    // source-token-account one
    let ix_1 = create_solana_account(
        &owner.pubkey(),
        &source_mint_one_token_account.pubkey(),
        Rent::default().minimum_balance(TokenAccount::LEN),
        TokenAccount::LEN as u64,
        &TOKEN_STANDARD_PROGRAM
    );
    let ix_2 = initialize_token_account(
        &TOKEN_STANDARD_PROGRAM,
        &source_mint_one_token_account.pubkey(),
        &mint_one_account.pubkey(),
        &owner.pubkey()
    ).unwrap();

    // source-token-account two
    let ix_3 = create_solana_account(
        &owner.pubkey(),
        &source_mint_two_token_account.pubkey(),
        Rent::default().minimum_balance(TokenAccount::LEN),
        TokenAccount::LEN as u64,
        &TOKEN_STANDARD_PROGRAM
    );
    let ix_4 = initialize_token_account(
        &TOKEN_STANDARD_PROGRAM,
        &source_mint_two_token_account.pubkey(),
        &mint_two_account.pubkey(),
        &owner.pubkey()
    ).unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[
            ix_1,
            ix_2,
            ix_3,
            ix_4
        ],
        Some(&owner.pubkey()),
        &[
            &owner,
            &source_mint_one_token_account,
            &source_mint_two_token_account
        ],
        *recent_blockhash
    );

    banks_client
        .process_transaction(tx)
        .await
        .unwrap();

    // 3. Create destination-token-accounts
    // destination-token-account one
    let ix_1 = create_solana_account(
        &owner.pubkey(),
        &destination_mint_one_token_account.pubkey(),
        Rent::default().minimum_balance(TokenAccount::LEN),
        TokenAccount::LEN as u64,
        &TOKEN_STANDARD_PROGRAM
    );
    let ix_2 = initialize_token_account(
        &TOKEN_STANDARD_PROGRAM,
        &destination_mint_one_token_account.pubkey(),
        &mint_one_account.pubkey(),
        &owner.pubkey()
    ).unwrap();

    // destination-token-account two
    let ix_3 = create_solana_account(
        &owner.pubkey(),
        &destination_mint_two_token_account.pubkey(),
        Rent::default().minimum_balance(TokenAccount::LEN),
        TokenAccount::LEN as u64,
        &TOKEN_STANDARD_PROGRAM
    );
    let ix_4 = initialize_token_account(
        &TOKEN_STANDARD_PROGRAM,
        &destination_mint_two_token_account.pubkey(),
        &mint_two_account.pubkey(),
        &owner.pubkey()
    ).unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[
            ix_1,
            ix_2,
            ix_3,
            ix_4
        ],
        Some(&owner.pubkey()),
        &[
            &owner,
            &destination_mint_one_token_account,
            &destination_mint_two_token_account
        ],
        *recent_blockhash
    );

    banks_client
        .process_transaction(tx)
        .await
        .unwrap();

    // 4. Mint tokens to sourec-token-accounts
    // mint to source-token-account one
    let ix = mint_to(
        &TOKEN_STANDARD_PROGRAM,
        &mint_one_account.pubkey(),
        &source_mint_one_token_account.pubkey(),
        &owner.pubkey(),
        &[&owner.pubkey()],
        1000_000u64
    ).unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&owner.pubkey()),
        &[&owner],
        *recent_blockhash
    );

    banks_client
        .process_transaction(tx)
        .await
        .unwrap();

    // mint to source-token-account two
    let ix = mint_to(
        &TOKEN_STANDARD_PROGRAM,
        &mint_two_account.pubkey(),
        &source_mint_two_token_account.pubkey(),
        &owner.pubkey(),
        &[&owner.pubkey()],
        500_00u64
    ).unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&owner.pubkey()),
        &[&owner],
        *recent_blockhash
    );

    banks_client
        .process_transaction(tx)
        .await
        .unwrap();
}

#[tokio::test]
async fn success_splitlamports() {
    let program_id = Pubkey::new_unique();
    let pt = setup(&program_id).await;
    let (
        mut banks_client,
        operator,
        recent_blockhash
    ) = pt.start().await;

    // 1. data
    let amounts: Vec<u64> = vec![
        sol_to_lamports(1.1),
        sol_to_lamports(1.2),
        sol_to_lamports(1.3)
    ];
    let mut data: Vec<u8> = vec![];
    data.extend_from_slice(
        SplitterInstruction::get_discriminator("instruction:splitlamports").as_slice()
    );
    data.append(
        &mut SplitLamports { amounts }
        .try_to_vec()
        .unwrap()
    );

    // 2. provide accounts
    let accounts: Vec<Pubkey> = vec![
        Keypair::new().pubkey(),
        Keypair::new().pubkey(),
        Keypair::new().pubkey()
    ]; 
    let keys: Vec<AccountMeta> = vec![
        AccountMeta::new(operator.pubkey(), true),
        AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
        AccountMeta::new(*accounts.get(0_usize).unwrap(), false),
        AccountMeta::new(*accounts.get(1_usize).unwrap(), false),
        AccountMeta::new(*accounts.get(2_usize).unwrap(), false)
    ];

    let tx = Transaction::new_signed_with_payer(
        &[
            Instruction {
                program_id,
                data,
                accounts: keys
            }
        ],
        Some(&operator.pubkey()),
        &[&operator],
        recent_blockhash
    );

    let operator_balance_before = banks_client.get_balance(operator.pubkey())
        .await
        .unwrap();

    banks_client
        .process_transaction(tx)
        .await
        .unwrap();

    let account_1 = banks_client.get_account(*accounts.get(0usize).unwrap())
        .await
        .unwrap()
        .unwrap();
    let account_2 = banks_client.get_account(*accounts.get(1usize).unwrap())
        .await
        .unwrap()
        .unwrap();
    let account_3 = banks_client.get_account(*accounts.get(2usize).unwrap())
        .await
        .unwrap()
        .unwrap();

    assert_eq!(
        account_1.lamports,
        sol_to_lamports(1.1),
        "Mismacth account_1 balance"
    );
    assert_eq!(
        account_2.lamports,
        sol_to_lamports(1.2),
        "Mismacth account_2 balance"
    );
    assert_eq!(
        account_3.lamports,
        sol_to_lamports(1.3),
        "Mismacth account_3 balance"
    );

    let operator_balance_after = banks_client.get_balance(operator.pubkey())
        .await
        .unwrap();
    assert_eq!(
        operator_balance_before - (sol_to_lamports(3.6) + 5000),
        operator_balance_after,
        "Mismatch operator balance"
    );
}

#[tokio::test]
async fn fail_splitlamports() {
    let program_id = Pubkey::new_unique();
    let pt = setup(&program_id).await;
    let (
        mut banks_client,
        operator,
        recent_blockhash
    ) = pt.start().await;

    // 1. data
    let amounts: Vec<u64> = vec![
        sol_to_lamports(1.1),
        sol_to_lamports(1.2),
        sol_to_lamports(1.3)
    ];
    let mut data: Vec<u8> = vec![];
    data.extend_from_slice(
        SplitterInstruction::get_discriminator("instruction:splitlamports").as_slice()
    );
    data.append(
        &mut SplitLamports { amounts }
        .try_to_vec()
        .unwrap()
    );

    // 2. provide accounts
    let accounts: Vec<Pubkey> = vec![
        Keypair::new().pubkey(),
        Keypair::new().pubkey()
    ]; 
    let keys: Vec<AccountMeta> = vec![
        AccountMeta::new(operator.pubkey(), true),
        AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
        AccountMeta::new(*accounts.get(0_usize).unwrap(), false),
        AccountMeta::new(*accounts.get(1_usize).unwrap(), false)
    ];

    let tx = Transaction::new_signed_with_payer(
        &[
            Instruction {
                program_id,
                data,
                accounts: keys
            }
        ],
        Some(&operator.pubkey()),
        &[&operator],
        recent_blockhash
    );

    banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();
}

#[tokio::test]
async fn success_splitfromsinglemint() {
    let program_id = Pubkey::new_unique();
    let pt = setup(&program_id).await;
    let (
        mut banks_client,
        payer,
        recent_blockhash
    ) = pt.start().await;

    // 1. setup all mint and token-accounts
    let owner = payer;
    let mint_account = Keypair::new();
    let src_token_account = Keypair::new();
    let dst_token_account = Keypair::new();
    let dst_sc_token_account = Keypair::new();

    setup_single_mint(
        &mut banks_client,
        &mint_account,
        &src_token_account,
        &dst_token_account,
        &dst_sc_token_account,
        &owner,
        &recent_blockhash
    ).await;

    // test owner instruction
    // 2. provide data
    let amounts: Vec<u64> = vec![
        10_00u64,
        20_00u64
    ];

    let mut data: Vec<u8> = vec![];
    data.extend_from_slice(
        SplitterInstruction::get_discriminator("instruction:splitspltokensfromsinglemint").as_slice()
    );
    data.append(
        &mut SplitSplTokensFromSingleMint { amounts }
            .try_to_vec()
            .unwrap()
    );

    // 3. provide accounts
    let accounts: Vec<AccountMeta> = vec![
        AccountMeta::new_readonly(owner.pubkey(), true),
        AccountMeta::new_readonly(TOKEN_STANDARD_PROGRAM, false),
        AccountMeta::new(src_token_account.pubkey(), false),
        AccountMeta::new(dst_token_account.pubkey(), false),
        AccountMeta::new(dst_sc_token_account.pubkey(), false)
    ];

    // 4. send transaction
    let tx = Transaction::new_signed_with_payer(
        &[
            Instruction {
                program_id,
                data,
                accounts
            }
        ],
        Some(&owner.pubkey()),
        &[&owner],
        recent_blockhash
    );

    banks_client
        .process_transaction(tx)
        .await
        .unwrap();

    // Checks...
    let mint_account_info = banks_client.get_account(mint_account.pubkey())
        .await
        .unwrap()
        .unwrap();
    let source_token_account_info = banks_client.get_account(src_token_account.pubkey())
        .await
        .unwrap()
        .unwrap();
    let destination_token_account_info = banks_client.get_account(dst_token_account.pubkey())
        .await
        .unwrap()
        .unwrap();
    let destination_second_token_account_info = banks_client.get_account(dst_sc_token_account.pubkey())
        .await
        .unwrap()
        .unwrap();

    let mint_account = MintAccount::unpack(
        mint_account_info.data.as_slice()
    ).unwrap();
    let source_token_account = TokenAccount::unpack(
        source_token_account_info.data.as_slice()
    ).unwrap();
    let destination_token_account = TokenAccount::unpack(
        destination_token_account_info.data.as_slice()
    ).unwrap();
    let destination_second_token_account = TokenAccount::unpack(
        destination_second_token_account_info.data.as_slice()
    ).unwrap();

    assert_eq!(
        mint_account.supply,
        100_00u64,
        "Mint's supply mismatch."
    );
    assert_eq!(
        source_token_account.amount,
        70_00u64,
        "Source token-account balance mismatch."
    );
    assert_eq!(
        destination_token_account.amount,
        10_00u64,
        "Destination token account balance mismatch."
    );
    assert_eq!(
        destination_second_token_account.amount,
        20_00u64,
        "Destination second token account balance mismatch."
    );
}

#[tokio::test]
async fn success_splitfromsinglemint_2() {
    let program_id = Pubkey::new_unique();
    let pt = setup(&program_id).await;
    let (
        mut banks_client,
        payer,
        recent_blockhash
    ) = pt.start().await;

    // 1. setup all mint and token-accounts
    let owner = payer;
    let mint_account = Keypair::new();
    let src_token_account = Keypair::new();
    let dst_token_account = Keypair::new();
    let dst_sc_token_account = Keypair::new();

    setup_single_mint(
        &mut banks_client,
        &mint_account,
        &src_token_account,
        &dst_token_account,
        &dst_sc_token_account,
        &owner,
        &recent_blockhash
    ).await;

    // test owner instruction
    // 2. provide data
    let amounts: Vec<u64> = vec![
        50_00u64
    ];

    let mut data: Vec<u8> = vec![];
    data.extend_from_slice(
        SplitterInstruction::get_discriminator("instruction:splitspltokensfromsinglemint").as_slice()
    );
    data.append(
        &mut SplitSplTokensFromSingleMint { amounts }
            .try_to_vec()
            .unwrap()
    );

    // 3. provide accounts
    let accounts: Vec<AccountMeta> = vec![
        AccountMeta::new_readonly(owner.pubkey(), true),
        AccountMeta::new_readonly(TOKEN_STANDARD_PROGRAM, false),
        AccountMeta::new(src_token_account.pubkey(), false),
        AccountMeta::new(dst_token_account.pubkey(), false),
        AccountMeta::new(dst_sc_token_account.pubkey(), false)
    ];

    // 4. send transaction
    let tx = Transaction::new_signed_with_payer(
        &[
            Instruction {
                program_id,
                data,
                accounts
            }
        ],
        Some(&owner.pubkey()),
        &[&owner],
        recent_blockhash
    );

    banks_client
        .process_transaction(tx)
        .await
        .unwrap();

    // Checks...
    let mint_account_info = banks_client.get_account(mint_account.pubkey())
        .await
        .unwrap()
        .unwrap();
    let source_token_account_info = banks_client.get_account(src_token_account.pubkey())
        .await
        .unwrap()
        .unwrap();
    let destination_token_account_info = banks_client.get_account(dst_token_account.pubkey())
        .await
        .unwrap()
        .unwrap();
    let destination_second_token_account_info = banks_client.get_account(dst_sc_token_account.pubkey())
        .await
        .unwrap()
        .unwrap();

    let mint_account = MintAccount::unpack(
        mint_account_info.data.as_slice()
    ).unwrap();
    let source_token_account = TokenAccount::unpack(
        source_token_account_info.data.as_slice()
    ).unwrap();
    let destination_token_account = TokenAccount::unpack(
        destination_token_account_info.data.as_slice()
    ).unwrap();
    let destination_second_token_account = TokenAccount::unpack(
        destination_second_token_account_info.data.as_slice()
    ).unwrap();

    assert_eq!(
        mint_account.supply,
        100_00u64,
        "Mint's supply mismatch."
    );
    assert_eq!(
        source_token_account.amount,
        50_00u64,
        "Source token-account balance mismatch."
    );
    assert_eq!(
        destination_token_account.amount,
        50_00u64,
        "Destination token account balance mismatch."
    );
    assert_eq!(
        destination_second_token_account.amount,
        0_00u64,
        "Destination second token account balance mismatch."
    );
}

#[tokio::test]
async fn fail_splitfromsinglemint() {
    let program_id = Pubkey::new_unique();
    let pt = setup(&program_id).await;
    let (
        mut banks_client,
        payer,
        recent_blockhash
    ) = pt.start().await;

    // 1. setup all mint and token-accounts
    let owner = payer;
    let mint_account = Keypair::new();
    let src_token_account = Keypair::new();
    let dst_token_account = Keypair::new();
    let dst_sc_token_account = Keypair::new();

    setup_single_mint(
        &mut banks_client,
        &mint_account,
        &src_token_account,
        &dst_token_account,
        &dst_sc_token_account,
        &owner,
        &recent_blockhash
    ).await;

    // test owner instruction
    // 2. provide data
    let amounts: Vec<u64> = vec![
        10_00u64,
        20_00u64
    ];

    let mut data: Vec<u8> = vec![];
    data.extend_from_slice(
        SplitterInstruction::get_discriminator("instruction:splitspltokensfromsinglemint").as_slice()
    );
    data.append(
        &mut SplitSplTokensFromSingleMint { amounts }
            .try_to_vec()
            .unwrap()
    );

    // 3. provide accounts
    let accounts: Vec<AccountMeta> = vec![
        AccountMeta::new_readonly(owner.pubkey(), true),
        AccountMeta::new_readonly(TOKEN_STANDARD_PROGRAM, false),
        AccountMeta::new(src_token_account.pubkey(), false),
        AccountMeta::new(dst_token_account.pubkey(), false)
    ];

    // 4. send transaction
    let tx = Transaction::new_signed_with_payer(
        &[
            Instruction {
                program_id,
                data,
                accounts
            }
        ],
        Some(&owner.pubkey()),
        &[&owner],
        recent_blockhash
    );

    banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    // Checks...
    let mint_account_info = banks_client.get_account(mint_account.pubkey())
        .await
        .unwrap()
        .unwrap();
    let source_token_account_info = banks_client.get_account(src_token_account.pubkey())
        .await
        .unwrap()
        .unwrap();
    let destination_token_account_info = banks_client.get_account(dst_token_account.pubkey())
        .await
        .unwrap()
        .unwrap();
    let destination_second_token_account_info = banks_client.get_account(dst_sc_token_account.pubkey())
        .await
        .unwrap()
        .unwrap();

    let mint_account = MintAccount::unpack(
        mint_account_info.data.as_slice()
    ).unwrap();
    let source_token_account = TokenAccount::unpack(
        source_token_account_info.data.as_slice()
    ).unwrap();
    let destination_token_account = TokenAccount::unpack(
        destination_token_account_info.data.as_slice()
    ).unwrap();
    let destination_second_token_account = TokenAccount::unpack(
        destination_second_token_account_info.data.as_slice()
    ).unwrap();

    assert_eq!(
        mint_account.supply,
        100_00u64,
        "Mint's supply mismatch."
    );
    assert_eq!(
        source_token_account.amount,
        100_00u64,
        "Source token-account balance mismatch."
    );
    assert_eq!(
        destination_token_account.amount,
        0_00u64,
        "Destination token account balance mismatch."
    );
    assert_eq!(
        destination_second_token_account.amount,
        0_00u64,
        "Destination second token account balance mismatch."
    );
}

#[tokio::test]
async fn success_splitefrommultiplemintaccounts() {
    let program_id = Pubkey::new_from_array([5u8; 32]);
    let pt = setup(&program_id).await;
    let (
        mut banks_client,
        payer,
        recent_blockhash
    ) = pt.start().await;

    // 1. setup all mint and token-accounts
    let owner = payer;
    let mint_account_1 = Keypair::new();
    let mint_account_2 = Keypair::new();
    let src_token_account_mint_1 = Keypair::new();
    let src_token_account_mint_2 = Keypair::new();
    let dst_token_account_mint_1 = Keypair::new();
    let dst_token_account_mint_2 = Keypair::new();
    
    setup_multiple_mints(
        &mut banks_client,
        &mint_account_1,
        &mint_account_2,
        &src_token_account_mint_1,
        &src_token_account_mint_2,
        &dst_token_account_mint_1,
        &dst_token_account_mint_2,
        &owner,
        &recent_blockhash
    ).await;

    // 2. Create instruction
    // provide data
    let mut data: Vec<u8> = Vec::new();
    data.extend_from_slice(
        SplitterInstruction::get_discriminator("instruction:splitspltokensfrommultiplemints").as_slice()
    );

    let amounts: Vec<u64> = vec![
        100_000u64,
        50_00u64
    ];
    let m = 2u16;

    data.append(
        &mut SplitSplTokensFromMultipleMints {
            amounts,
            m
        }.try_to_vec().unwrap()
    );

    // provide accounts
    let accounts: Vec<AccountMeta> = vec![
        AccountMeta::new_readonly(owner.pubkey(), true),
        AccountMeta::new_readonly(TOKEN_STANDARD_PROGRAM, false),
        AccountMeta::new(src_token_account_mint_1.pubkey(), false),
        AccountMeta::new(src_token_account_mint_2.pubkey(), false),
        AccountMeta::new(dst_token_account_mint_1.pubkey(), false),
        AccountMeta::new(dst_token_account_mint_2.pubkey(), false)
    ];

    let ix = Instruction {
        program_id,
        data,
        accounts
    };
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&owner.pubkey()),
        &[&owner],
        recent_blockhash
    );

    banks_client
        .process_transaction(tx)
        .await
        .unwrap();

    let source_token_account_mint_1 = TokenAccount::unpack(
        banks_client
            .get_account(src_token_account_mint_1.pubkey())
            .await
            .unwrap()
            .unwrap()
            .data
            .as_slice()
    ).unwrap();
    let source_token_account_mint_2 = TokenAccount::unpack(
        banks_client
            .get_account(src_token_account_mint_2.pubkey())
            .await
            .unwrap()
            .unwrap()
            .data
            .as_slice()
    ).unwrap();
    let destination_token_account_mint_1 = TokenAccount::unpack(
        banks_client
            .get_account(dst_token_account_mint_1.pubkey())
            .await
            .unwrap()
            .unwrap()
            .data
            .as_slice()
    ).unwrap();
    let destination_token_account_mint_2 = TokenAccount::unpack(
        banks_client
            .get_account(dst_token_account_mint_2.pubkey())
            .await
            .unwrap()
            .unwrap()
            .data
            .as_slice()
    ).unwrap();

    assert_eq!(
        source_token_account_mint_1.amount,
        (1000_000 - 100_000) as u64,
        "Source token account mint-1 token balance mismatch."
    );
    assert_eq!(
        source_token_account_mint_2.amount,
        (500_00 - 50_00) as u64,
        "Source token account mint-2 token balance mismatch."
    );
    assert_eq!(
        destination_token_account_mint_1.amount,
        100_000u64,
        "Destination token account mint-1 token balance mismatch."
    );
    assert_eq!(
        destination_token_account_mint_2.amount,
        50_00u64,
        "Destination token account mint-2 token balance mismatch."
    );
}

#[tokio::test]
async fn fail_splitefrommultiplemintaccounts() {
    let program_id = Pubkey::new_from_array([5u8; 32]);
    let pt = setup(&program_id).await;
    let (
        mut banks_client,
        payer,
        recent_blockhash
    ) = pt.start().await;

    // 1. setup all mint and token-accounts
    let owner = payer;
    let mint_account_1 = Keypair::new();
    let mint_account_2 = Keypair::new();
    let src_token_account_mint_1 = Keypair::new();
    let src_token_account_mint_2 = Keypair::new();
    let dst_token_account_mint_1 = Keypair::new();
    let dst_token_account_mint_2 = Keypair::new();
    
    setup_multiple_mints(
        &mut banks_client,
        &mint_account_1,
        &mint_account_2,
        &src_token_account_mint_1,
        &src_token_account_mint_2,
        &dst_token_account_mint_1,
        &dst_token_account_mint_2,
        &owner,
        &recent_blockhash
    ).await;

    // 2. Create instruction
    // provide data
    let mut data: Vec<u8> = Vec::new();
    data.extend_from_slice(
        SplitterInstruction::get_discriminator("instruction:splitspltokensfrommultiplemints").as_slice()
    );

    let amounts: Vec<u64> = vec![
        100_000u64,
        50_00u64
    ];
    let m = 1u16;

    data.append(
        &mut SplitSplTokensFromMultipleMints {
            amounts,
            m
        }.try_to_vec().unwrap()
    );

    // provide accounts
    let accounts: Vec<AccountMeta> = vec![
        AccountMeta::new_readonly(owner.pubkey(), true),
        AccountMeta::new_readonly(TOKEN_STANDARD_PROGRAM, false),
        AccountMeta::new(src_token_account_mint_1.pubkey(), false),
        AccountMeta::new(src_token_account_mint_2.pubkey(), false),
        AccountMeta::new(dst_token_account_mint_1.pubkey(), false),
        AccountMeta::new(dst_token_account_mint_2.pubkey(), false)
    ];

    let ix = Instruction {
        program_id,
        data,
        accounts
    };
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&owner.pubkey()),
        &[&owner],
        recent_blockhash
    );

    banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();
}

#[tokio::test]
async fn fail_splitefrommultiplemintaccounts_2() {
    let program_id = Pubkey::new_from_array([5u8; 32]);
    let pt = setup(&program_id).await;
    let (
        mut banks_client,
        payer,
        recent_blockhash
    ) = pt.start().await;

    // 1. setup all mint and token-accounts
    let owner = payer;
    let mint_account_1 = Keypair::new();
    let mint_account_2 = Keypair::new();
    let src_token_account_mint_1 = Keypair::new();
    let src_token_account_mint_2 = Keypair::new();
    let dst_token_account_mint_1 = Keypair::new();
    let dst_token_account_mint_2 = Keypair::new();
    
    setup_multiple_mints(
        &mut banks_client,
        &mint_account_1,
        &mint_account_2,
        &src_token_account_mint_1,
        &src_token_account_mint_2,
        &dst_token_account_mint_1,
        &dst_token_account_mint_2,
        &owner,
        &recent_blockhash
    ).await;

    // 2. Create instruction
    // provide data
    let mut data: Vec<u8> = Vec::new();
    data.extend_from_slice(
        SplitterInstruction::get_discriminator("instruction:splitspltokensfrommultiplemints").as_slice()
    );

    let amounts: Vec<u64> = vec![
        100_000u64,
        50_00u64,
        100_00u64
    ];
    let m = 2u16;

    data.append(
        &mut SplitSplTokensFromMultipleMints {
            amounts,
            m
        }.try_to_vec().unwrap()
    );

    // provide accounts
    let accounts: Vec<AccountMeta> = vec![
        AccountMeta::new_readonly(owner.pubkey(), true),
        AccountMeta::new_readonly(TOKEN_STANDARD_PROGRAM, false),
        AccountMeta::new(src_token_account_mint_1.pubkey(), false),
        AccountMeta::new(src_token_account_mint_2.pubkey(), false),
        AccountMeta::new(dst_token_account_mint_1.pubkey(), false),
        AccountMeta::new(dst_token_account_mint_2.pubkey(), false)
    ];

    let ix = Instruction {
        program_id,
        data,
        accounts
    };
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&owner.pubkey()),
        &[&owner],
        recent_blockhash
    );

    banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();
}