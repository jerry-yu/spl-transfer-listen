use solana_client::{
    rpc_client::{GetConfirmedSignaturesForAddress2Config, RpcClient},
    rpc_config::RpcProgramAccountsConfig,
    rpc_filter::{Memcmp, MemcmpEncodedBytes, RpcFilterType},
};
use solana_program::{instruction::Instruction, program_pack::Pack, pubkey::Pubkey};
use solana_transaction_status::{
    EncodedTransaction, UiInstruction, UiMessage, UiParsedInstruction, UiTransactionEncoding,
};

use mpl_token_metadata::accounts::Metadata;
use mpl_token_metadata::instructions::{CreateMetadataAccountV3, CreateMetadataAccountV3Builder};
use mpl_token_metadata::types::DataV2;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    derivation_path::DerivationPath,
    rent::Rent,
    signature::{Keypair, Signature, Signer},
    signer::{keypair::generate_seed_from_seed_phrase_and_passphrase, SeedDerivable},
    system_instruction,
    transaction::Transaction,
};
use spl_associated_token_account::{
    get_associated_token_address, instruction::create_associated_token_account,
};
use spl_memo::build_memo;
use spl_memo::id as memo_id;
use spl_token::{
    instruction as token_instruction, state::Account as TokenAccount, state::Mint as TokenMint,
};
use std::{env, str::FromStr};

fn keypair(s: String) -> Keypair {
    let seed = generate_seed_from_seed_phrase_and_passphrase(&s, "");
    let derivation_path = DerivationPath::from_key_str("0/0").unwrap();
    // let kp = Keypair::from_base58_string(
    //     "",
    // );
    let kp = Keypair::from_seed_and_derivation_path(&seed, Some(derivation_path)).unwrap();
    println!("address {}", kp.pubkey());
    kp
}

// 1. seed phrase 2.option, default 1_000_000 2. option: defualt: "https://api.mainnet-beta.solana.com"
fn create(args: Vec<String>) {
    let len = args.len();

    let rpc_url = if len == 3 {
        args[2].clone()
    } else {
        "https://api.mainnet-beta.solana.com".to_string()
    };

    let amount: u64 = if len >= 2 {
        args[1].parse().unwrap()
    } else {
        1_000_000
    };

    let client = RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed());

    let payer = keypair(args[0].clone());

    let mint_keypair = Keypair::new();

    let rent = Rent::default();
    let create_mint_account_ix = system_instruction::create_account(
        &payer.pubkey(),
        &mint_keypair.pubkey(),
        rent.minimum_balance(TokenMint::LEN),
        TokenMint::LEN as u64,
        &spl_token::ID,
    );

    let initialize_mint_instruction = token_instruction::initialize_mint(
        &spl_token::ID,
        &mint_keypair.pubkey(),
        &payer.pubkey(),
        None,
        9,
    )
    .unwrap();

    let recipient_pubkey = payer.pubkey();
    // solana_sdk::pubkey::Pubkey::from_str("WQRVRTbdSeBdPgocRg26oV5oqPxXhnzPD6KfBztGTtK")
    //     .unwrap();

    let recipient_token_pubkey =
        get_associated_token_address(&recipient_pubkey, &mint_keypair.pubkey());
    // let x =client.get_account(&recipient_token_pubkey);
    // println!("{:?}",x);

    // if x.is_err()  {
    let ata_ix = create_associated_token_account(
        &payer.pubkey(),
        &recipient_pubkey,
        &mint_keypair.pubkey(),
        &spl_token::ID,
    );

    // }
    let mint_to_instruction = token_instruction::mint_to(
        &spl_token::ID,
        &mint_keypair.pubkey(),
        &recipient_token_pubkey,
        &payer.pubkey(),
        &[],
        amount * 10u64.pow(9), // amount
    )
    .unwrap();

    let (metadata_pubkey, _bump) = Metadata::find_pda(&mint_keypair.pubkey());

    // let create_metadata_account_ix = system_instruction::create_account(
    //     &mint_keypair.pubkey(),
    //     &metadata_pubkey,
    //     rent.minimum_balance(800),
    //     800,
    //     &mpl_token_metadata::ID,
    // );

    let data = DataV2 {
        name: "Deeper Network".to_string(),
        symbol: "DPR".to_string(),
        uri: "https://44gphqzcxus5c2xbrleetiamwycy7pgfzjsjfkijacj6h5me6qvq.arweave.net/5wzzwyK9JdFq4YrISaAMtgWPvMXKZJKpCQCT4_WE9Cs".to_string(),
        seller_fee_basis_points: 0,
        creators: None,
        collection: None,
        uses: None,
    };

    let create_metadata_ix = CreateMetadataAccountV3Builder::new()
        .metadata(metadata_pubkey)
        .data(data)
        .mint_authority(payer.pubkey())
        .mint(mint_keypair.pubkey())
        .payer(payer.pubkey())
        .update_authority(payer.pubkey(), true)
        .is_mutable(true)
        .instruction();

    let mut transaction = Transaction::new_with_payer(
        &[
            create_mint_account_ix,
            initialize_mint_instruction,
            ata_ix,
            mint_to_instruction,
            //create_metadata_account_ix,
            create_metadata_ix,
        ],
        Some(&payer.pubkey()),
    );
    transaction.sign(
        &[&payer, &mint_keypair],
        client.get_latest_blockhash().unwrap(),
    );

    client.send_and_confirm_transaction(&transaction).unwrap();

    println!("Mint Account: {}", mint_keypair.pubkey());
    println!("Recipient Token Account: {}", recipient_token_pubkey);

    println!("Token {}", mint_keypair.pubkey())
    // Metadata::find_pda(mint)
    // let mut meta = CreateMetadataAccountV3Builder::new();
    // meta.metadata(mint)
}

//1. seed phrase 2. token address 3. amount 4. rpc
fn mint(args: Vec<String>) {
    let len = args.len();

    let rpc_url = if len == 4 {
        args[3].clone()
    } else {
        "https://api.mainnet-beta.solana.com".to_string()
    };

    let amount: u64 = if len >= 3 {
        args[2].parse().unwrap()
    } else {
        1_000_000
    };

    let client = RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed());

    let payer = keypair(args[0].clone());

    let mint_pubkey = Pubkey::from_str(&args[1]).unwrap();

    let recipient_token_pubkey = get_associated_token_address(&payer.pubkey(), &mint_pubkey);
    let mut ixs = vec![];
    if client.get_account(&recipient_token_pubkey).is_err() {
        let ata_ix = create_associated_token_account(
            &payer.pubkey(),
            &payer.pubkey(),
            &mint_pubkey,
            &spl_token::ID,
        );

        ixs.push(ata_ix);
    }

    let mint_to_instruction = token_instruction::mint_to(
        &spl_token::ID,
        &mint_pubkey,
        &recipient_token_pubkey,
        &payer.pubkey(),
        &[],
        amount * 10u64.pow(9), // amount
    )
    .unwrap();

    ixs.push(mint_to_instruction);

    let mut transaction = Transaction::new_with_payer(&ixs, Some(&payer.pubkey()));
    transaction.sign(&[&payer], client.get_latest_blockhash().unwrap());

    client.send_and_confirm_transaction(&transaction).unwrap();

    println!(
        "Token {} mint to {}  {} DPR success",
        mint_pubkey,
        payer.pubkey(),
        amount
    )
    // Metadata::find_pda(mint)
    // let mut meta = CreateMetadataAccountV3Builder::new();
    // meta.metadata(mint)
}

//1. seed phrase 2. token account 3. new authority 4. rpc
fn set_authority(args: Vec<String>) {
    let len = args.len();

    let rpc_url = if len == 4 {
        args[3].clone()
    } else {
        "https://api.mainnet-beta.solana.com".to_string()
    };

    let client = RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed());
    let payer = keypair(args[0].clone());
    let token_pubkey = Pubkey::from_str(&args[1]).unwrap();
    let new_author_pubkey = Pubkey::from_str(&args[2]).unwrap();

    let set_authority_instruction = token_instruction::set_authority(
        &spl_token::ID,
        &token_pubkey,
        Some(&new_author_pubkey),
        token_instruction::AuthorityType::MintTokens,
        &payer.pubkey(),
        &[&payer.pubkey()],
    )
    .unwrap();

    let mut transaction =
        Transaction::new_with_payer(&[set_authority_instruction], Some(&payer.pubkey()));
    transaction.sign(&[&payer], client.get_latest_blockhash().unwrap());

    client.send_and_confirm_transaction(&transaction).unwrap();

    println!(
        "Token {} mint authority from {} to {}  success",
        token_pubkey,
        payer.pubkey(),
        new_author_pubkey
    )
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let len = args.len();
    if len < 2 {
        println!("params too short");
    }
    let sub_cmd: &str = &args[1];
    match sub_cmd {
        "address" => {
            keypair(args[2].clone());
        }
        "create" => create(args[2..].to_vec()),
        "mint" => mint(args[2..].to_vec()),
        "set_authority" => set_authority(args[2..].to_vec()),
        _ => {}
    }
}

// fn set_metadata() {
//     let x = vec![
//
//     ];
//     let payer = Keypair::from_bytes(&x).unwrap();
//     let token =
//         solana_sdk::pubkey::Pubkey::from_str("5WQBdXjMHF7ExBUFyXwdkRdQiASddFMh4GHrY9RL6NKk")
//             .unwrap();
//     let rpc_url = String::from("https://api.devnet.solana.com");
//     let client = RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed());

//     let (metadata_pubkey, _bump) = Metadata::find_pda(&token);

//     let rent = Rent::default();
//     let create_metadata_account_ix = system_instruction::create_account(
//         &payer.pubkey(),
//         &metadata_pubkey,
//         rent.minimum_balance(84),
//         84,
//         &mpl_token_metadata::ID,
//     );

//     let data = DataV2 {
//         name: "Deeper Network".to_string(),
//         symbol: "DPR".to_string(),
//         uri: "https://pmq5p5pe4cws2yjjo634nrmcceara4peepikv3vmdqwip7gjumxa.arweave.net/eyHX9eTgrS1hKXe3xsWCEQEQceQj0KrurBwsh_zJoy4".to_string(),
//         seller_fee_basis_points: 0,
//         creators: None,
//         collection: None,
//         uses: None,
//     };

//     let create_metadata_ix = CreateMetadataAccountV3Builder::new()
//         .metadata(metadata_pubkey)
//         .data(data)
//         .mint_authority(payer.pubkey())
//         .mint(token)
//         .payer(payer.pubkey())
//         .update_authority(payer.pubkey(), false)
//         .is_mutable(true)
//         .instruction();

//     let mut transaction = Transaction::new_with_payer(
//         &[create_metadata_account_ix, create_metadata_ix],
//         Some(&payer.pubkey()),
//     );

//     // 签名并发送交易
//     transaction.sign(&[&payer], client.get_latest_blockhash().unwrap());
//     client.send_and_confirm_transaction(&transaction).unwrap();

//     println!("Metadata 账户: {}", metadata_pubkey);
// }

// fn transfer() {
//     // 连接到 Solana 节点
//     let rpc_url = String::from("https://api.testnet.solana.com");
//     let client = RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed());

//     let x = vec![
//     ];
//     println!("{:02x?}", x);
//     let payer = Keypair::from_bytes(&x).unwrap();

//     // 接收者的 SPL Token 地址
//     let recipient_token_account_pubkey =
//         solana_sdk::pubkey::Pubkey::from_str("ykVbD8L8zPdYM9cCP4EVzKCznoRx1T1VwoWj3CrRehG")
//             .unwrap();

//     // 发送者的 SPL Token 地址
//     let sender_token_account_pubkey =
//         solana_sdk::pubkey::Pubkey::from_str("EoeUawi9uyajomTk4DPvThcBtFnpnNNcC4N8dRSqcZhH")
//             .unwrap();

//     // SPL Token 的 Mint 地址
//     let token_mint_pubkey =
//         solana_sdk::pubkey::Pubkey::from_str("7yc4MSZ5wmqng8X4u2xrDgYKkLQdqyKLjpUFRXxvFqd9")
//             .unwrap();

//     let amount = 1_000_000_000;

//     let transfer_instruction = token_instruction::transfer(
//         &spl_token::id(),
//         &sender_token_account_pubkey,
//         &recipient_token_account_pubkey,
//         &payer.pubkey(),
//         &[&payer.pubkey()],
//         amount,
//     )
//     .unwrap();

//     let memo = "deeper-chain dst address".as_bytes();
//     let memo_instruction = build_memo(memo, &[&payer.pubkey()]);

//     let mut transaction = Transaction::new_with_payer(
//         &[transfer_instruction, memo_instruction],
//         Some(&payer.pubkey()),
//     );

//     let recent_blockhash = client.get_latest_blockhash().unwrap();
//     transaction.sign(&[&payer], recent_blockhash);

//     // let result = client.send_and_confirm_transaction(&transaction);

//     // match result {
//     //     Ok(signature) => println!("tx success: {}", signature),
//     //     Err(err) => println!("tx fail: {:?}", err),
//     // }

//     let config = GetConfirmedSignaturesForAddress2Config {
//         before: None,
//         until: None,
//         limit: None,
//         commitment: Some(CommitmentConfig::confirmed()),
//     };

//     let signatures = client
//         .get_signatures_for_address_with_config(&sender_token_account_pubkey, config)
//         .unwrap();

//     for signature in signatures {
//         if let Some(memo) = signature.memo {
//             if memo.contains("deeper-chain dst address") {
//                 println!("sig {} mem {:?}", signature.signature, memo);

//                 let s: Signature = Signature::from_str(&signature.signature).unwrap();
//                 let tx = client
//                     .get_transaction(&s, UiTransactionEncoding::JsonParsed)
//                     .unwrap();
//                 //println!("EncodedConfirmedTransactionWithStatusMeta-- {:?}",tx);

//                 //let tx  = tx.transaction.transaction;

//                 // let tx = tx.decode();
//                 // println!("{:?}",tx);
//                 // if let Some(vtx) = tx {
//                 //     let is  = vtx.message.instructions();

//                 //     for i in is {
//                 //        let accounts = vtx.message.static_account_keys();
//                 //        let pid = i.program_id(accounts);
//                 //        println!("{:?}",pid);

//                 //        let ti = token_instruction::TokenInstruction::unpack(&i.data);
//                 //        println!("{:?}",ti);
//                 //         if let Ok(ti) = ti {

//                 //         }

//                 //     }
//                 // }

//                 if let EncodedTransaction::Json(uitx) = tx.transaction.transaction {
//                     if let UiMessage::Parsed(uiParsed) = uitx.message {
//                         if let UiInstruction::Parsed(UiParsedInstruction::Parsed(uiIns)) =
//                             uiParsed.instructions[0].clone()
//                         {
//                             println!("UiParsedInstruction-- {:?}", uiIns);
//                         }
//                     }
//                 }
//             }
//         }
//     }
// }
