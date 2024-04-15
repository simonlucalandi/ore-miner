
use clap::Parser;
use solana_sdk::{
    pubkey::Pubkey, signature::{Keypair, Signer},
    signer::EncodableKey, system_instruction, transaction::Transaction,
    message::Message
};

use crate::Miner;

#[derive(Parser, Debug, Clone)]
pub struct CollectArgs {
    #[arg(long)]
    pub keypair: String,

    #[arg(long)]
    pub beneficiary: Pubkey,

    #[arg(long)]
    pub fee_payer: String,
}

impl Miner {
    pub async fn collect(&self, args: &CollectArgs) {
        let client = Miner::get_client_confirmed(&self.rpc);
        let accounts = Self::read_keys(&args.keypair);
        let fee_payer_account =  Keypair::read_from_file(&args.fee_payer).unwrap();
        
        let mut instructions = Vec::new();
        let mut signers = Vec::new();

        let balance_fee_payer = client
            .get_balance(&fee_payer_account.pubkey())
            .await
            .expect("Failed to get balance");

        println!("Fee payer balance: {}", balance_fee_payer);

        for keypair in accounts.iter() {
            let pubkey = keypair.pubkey();
            let balance = client
                .get_balance(&pubkey)
                .await
                .expect("Failed to get balance");
            let rent_exemption = client
                .get_minimum_balance_for_rent_exemption(0)
                .await
                .expect("Failed to get minimum balance for rent exemption");

            if balance - rent_exemption > 0 {
                let instruction = system_instruction::transfer(
                    &pubkey,
                    &args.beneficiary,
                    balance - rent_exemption,
                );
                instructions.push(instruction);
                signers.push(keypair);
                println!("Bundling transfer of {} from {} to {}", balance, pubkey, args.beneficiary)
            }

            if instructions.len() >= 8 {
                signers.push(&fee_payer_account);
                
                
                
                let recent_blockhash = client
                .get_latest_blockhash()
                .await
                .expect("Failed to get recent blockhash");

                let transaction = Transaction::new_signed_with_payer(
                    &instructions,
                    Some(&fee_payer_account.pubkey()),
                    &signers,
                    recent_blockhash,
                );
                
                let message = Message::new(&instructions, Some(&fee_payer_account.pubkey()));
                let estimate_transfer_fee = client.get_fee_for_message(&message).await.expect("Failed to get fee for message");

                if estimate_transfer_fee > balance_fee_payer {
                    eprintln!("Insufficient funds to pay for transaction fee");
                    return;
                }

                println!("Estimate transfer fee: {}", estimate_transfer_fee);

                match client.send_and_confirm_transaction(&transaction).await {
                    Ok(signature) => {
                        println!("Bundled transfer succeeded. Signature: {}", signature);
                    }
                    Err(err) => {
                        eprintln!("Bundled transfer failed: err {}", err);
                    }
                }

                instructions.clear();
                signers.clear();
            }
        }
        
     
    }

}