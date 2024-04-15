use std::{sync::Arc};

use clap::Parser;
use ore::{state::Proof, utils::AccountDeserialize};
use solana_sdk::{signature::Signer};
use tracing::{info};

use crate::{constant, utils, Miner};

#[derive(Parser, Debug, Clone)]
pub struct BalanceArgs {
   
    #[arg(long, help = "The folder that contains all the keys used to claim $ORE")]
    pub key_folder: String,
   
}

impl BalanceArgs {
}

impl Miner {
    pub async fn balance(&self, args: &BalanceArgs) {
        let client = Miner::get_client_confirmed(&self.rpc);
        let accounts = Self::read_keys(&args.key_folder);

        let owner_proof_pdas = accounts
            .iter()
            .map(|key| (utils::get_proof_pda(key.pubkey())))
            .collect::<Vec<_>>();

        if owner_proof_pdas.is_empty() {
            info!("No claimable accounts found");
            return;
        }

    
        let mut claimable = Vec::with_capacity(owner_proof_pdas.len());

        // Fetch claimable amount of each account
        for (batch_pda, batch_account) in owner_proof_pdas
            .chunks(constant::FETCH_ACCOUNT_LIMIT)
            .zip(accounts.chunks(constant::FETCH_ACCOUNT_LIMIT))
        {
            let batch_accounts = client
                .get_multiple_accounts(batch_pda)
                .await
                .expect("Failed to get Proof accounts")
                .into_iter()
                .zip(batch_account.iter())
                .filter_map(|(account, key)| {
                    let account_data = account?.data;
                    let proof = Proof::try_from_bytes(&account_data).ok()?;

                    if proof.claimable_rewards == 0 {
                        return None;
                    }

                    Some((
                        key.pubkey(),
                        Arc::new(key.insecure_clone()) as Arc<dyn Signer>,
                        proof.claimable_rewards,
                    ))
                });

            claimable.extend(batch_accounts);
        }

        claimable.sort_by_key(|(_, _, amount)| *amount);
        claimable.reverse();

        let remaining = claimable.iter().map(|(_, _, amount)| amount).sum::<u64>();

        info!("total rewards: {}", utils::ore_ui_amount(remaining));
        info!("total claimable accounts: {}", claimable.len());

    
    }
}
