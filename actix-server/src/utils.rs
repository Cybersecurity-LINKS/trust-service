// Copyright 2020-2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use anyhow::Context;
use iota_sdk::{client::{node_api::indexer::query_parameters::QueryParameter, Client}, types::block::address::Bech32Address, wallet::Account, Wallet, wallet::Result};

/// Requests funds from the faucet for the given `address`.
pub async fn request_faucet_funds(client: &Client, address: &Bech32Address, faucet_endpoint: &str) -> anyhow::Result<()> {
    iota_sdk::client::request_funds_from_faucet(faucet_endpoint, &address).await?;

    tokio::time::timeout(std::time::Duration::from_secs(45), async {
        loop {
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;

        let balance = get_address_balance(client, &address)
            .await
            .context("failed to get address balance")?;
        if balance > 0 {
            break;
        }
        }
        Ok::<(), anyhow::Error>(())
    })
    .await
    .context("maximum timeout exceeded")??;

    Ok(())
}
  
/// Returns the balance of the given Bech32-encoded `address`.
pub async fn get_address_balance(client: &Client, address: &Bech32Address) -> anyhow::Result<u64> {
    let output_ids = client
        .basic_output_ids(vec![
        QueryParameter::Address(address.to_owned()),
        QueryParameter::HasExpiration(false),
        QueryParameter::HasTimelock(false),
        QueryParameter::HasStorageDepositReturn(false),
        ])
        .await?;

    let outputs = client.get_outputs(&output_ids).await?;

    let mut total_amount = 0;
    for output_response in outputs {
        total_amount += output_response.output().amount();
    }

    Ok(total_amount)
}

pub async fn print_accounts(wallet: &Wallet) -> Result<()> {
    let accounts = wallet.get_accounts().await?;
    println!("Accounts:");
    for account in accounts {
        let details = account.details().await;
        println!("- {}", details.alias());
    }
    Ok(())
}

pub async fn print_addresses(account: &Account) -> Result<()> {
    let addresses = account.addresses().await?;
    println!("{}'s addresses:", account.alias().await);
    for address in addresses {
        println!("- {}", address.address());
    }
    Ok(())
}

pub async fn sync_print_balance(account: &Account, full_report: bool) -> Result<()> {
    let alias = account.alias().await;
    let now = tokio::time::Instant::now();
    let balance = account.sync(None).await?;
    println!("{alias}'s account synced in: {:.2?}", now.elapsed());
    if full_report {
        println!("{alias}'s balance:\n{balance:#?}");
    } else {
        println!("{alias}'s coin balance:\n{:#?}", balance.base_coin());
    }
    Ok(())
}

pub async fn print_addresses_with_funds(account: &Account) -> Result<()> {
    let addresses_with_unspent_outputs = account.addresses_with_unspent_outputs().await?;
    println!(
        "{}'s addresses with funds/assets: {}",
        account.alias().await,
        addresses_with_unspent_outputs.len()
    );
    for address_with_unspent_outputs in addresses_with_unspent_outputs {
        println!("- {}", address_with_unspent_outputs.address());
        println!("  Output Ids:");
        for output_id in address_with_unspent_outputs.output_ids() {
            println!("  {}", output_id);
        }
    }
    Ok(())
}