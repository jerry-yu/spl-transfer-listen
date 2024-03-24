use solana_client::{
    rpc_client::{GetConfirmedSignaturesForAddress2Config,RpcClient},
    rpc_filter::{Memcmp, MemcmpEncodedBytes, RpcFilterType},
    rpc_config::RpcProgramAccountsConfig,
};
use solana_transaction_status::{EncodedTransaction,UiTransactionEncoding,UiMessage,UiInstruction,UiParsedInstruction};

use solana_sdk::{
    signature::{Keypair, Signer,Signature},
    transaction::Transaction,
    commitment_config::CommitmentConfig,
};
use spl_token::{
    instruction as token_instruction,
    state::Account as TokenAccount,
};
use std::str::FromStr;
use spl_memo::build_memo;
use spl_memo::id as memo_id;

fn main() {
    // 连接到 Solana 节点
    let rpc_url = String::from("https://api.testnet.solana.com");
    let client = RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed());

    let x = vec![99u8,145,152,156,220,165,60,157,4,136,243,62,204,38,190,94,82,201,134,153,100,230,34,243,31,21,168,253,209,59,204,192,205,65,234,255,47,147,140,84,234,176,144
        ,169,157,219,170,161,56,150,167,230,253,232,139,25,60,121,62,74,230,112,206,32];
    println!("{:02x?}",x);
    let payer = Keypair::from_bytes(&x).unwrap();

    let sk = hex::hex!(x);
    
    // 接收者的 SPL Token 地址
    let recipient_token_account_pubkey = solana_sdk::pubkey::Pubkey::from_str("ykVbD8L8zPdYM9cCP4EVzKCznoRx1T1VwoWj3CrRehG").unwrap();
    
    // 发送者的 SPL Token 地址
    let sender_token_account_pubkey = solana_sdk::pubkey::Pubkey::from_str("EoeUawi9uyajomTk4DPvThcBtFnpnNNcC4N8dRSqcZhH").unwrap();
    
    // SPL Token 的 Mint 地址
    let token_mint_pubkey = solana_sdk::pubkey::Pubkey::from_str("7yc4MSZ5wmqng8X4u2xrDgYKkLQdqyKLjpUFRXxvFqd9").unwrap();
    
    let amount = 1_000_000_000; 

    let transfer_instruction = token_instruction::transfer(
        &spl_token::id(),
        &sender_token_account_pubkey,
        &recipient_token_account_pubkey,
        &payer.pubkey(),
        &[&payer.pubkey()],
        amount,
    ).unwrap();

    let memo = "deeper-chain dst address".as_bytes();
    let memo_instruction = build_memo(memo, &[&payer.pubkey()]);

    let mut transaction = Transaction::new_with_payer(
        &[transfer_instruction, memo_instruction],
        Some(&payer.pubkey()),
    );

    let recent_blockhash = client.get_latest_blockhash().unwrap();
    transaction.sign(&[&payer], recent_blockhash);

    // let result = client.send_and_confirm_transaction(&transaction);

    // match result {
    //     Ok(signature) => println!("tx success: {}", signature),
    //     Err(err) => println!("tx fail: {:?}", err),
    // }


     let config = GetConfirmedSignaturesForAddress2Config {
         before: None,
         until: None,
         limit: None,
         commitment: Some(CommitmentConfig::confirmed()),
     };
     
     let signatures = client
     .get_signatures_for_address_with_config(&sender_token_account_pubkey,config)
     .unwrap();

 for signature in signatures {
      
      if let Some(memo)  = signature.memo {
        if memo.contains("deeper-chain dst address") {
            println!("sig {} mem {:?}",signature.signature,memo);


            let s: Signature = Signature::from_str(&signature.signature).unwrap();
            let tx = client.get_transaction(&s, UiTransactionEncoding::JsonParsed).unwrap();
            //println!("EncodedConfirmedTransactionWithStatusMeta-- {:?}",tx);

            //let tx  = tx.transaction.transaction;

            // let tx = tx.decode();
            // println!("{:?}",tx);
            // if let Some(vtx) = tx {
            //     let is  = vtx.message.instructions();

            //     for i in is {
            //        let accounts = vtx.message.static_account_keys();
            //        let pid = i.program_id(accounts);
            //        println!("{:?}",pid);

            //        let ti = token_instruction::TokenInstruction::unpack(&i.data);
            //        println!("{:?}",ti);
            //         if let Ok(ti) = ti {
                       
            //         }

                   
            //     }
            // }

            if let EncodedTransaction::Json(uitx) = tx.transaction.transaction {
                if let UiMessage::Parsed(uiParsed) = uitx.message {
                     if let UiInstruction::Parsed(UiParsedInstruction::Parsed(uiIns)) = uiParsed.instructions[0].clone() {
                        println!("UiParsedInstruction-- {:?}",uiIns);
                     }
                }
            }

        }
      }
     
 }


}
