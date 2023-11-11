// Copyright 2020-2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0
use std::env;

// use identity_iota::crypto::KeyPair;
// use identity_iota::crypto::KeyType;
// use identity_iota::iota::IotaClientExt;
// use identity_iota::iota::IotaDocument;
// use identity_iota::iota::IotaIdentityClientExt;
// use identity_iota::iota::NetworkName;
// use identity_iota::verification::MethodScope;
// use identity_iota::verification::VerificationMethod;

// use iota_client::block::address::Address;
// use iota_client::block::output::AliasOutput;
// use iota_client::block::output::Output;
// use iota_client::crypto::keys::bip39;
// use iota_client::node_api::indexer::query_parameters::QueryParameter;
// use iota_client::secret::SecretManager;
// use iota_client::Client;
// use iota_sdk::types::block::address::Bech32Address;
// use rand::distributions::DistString;

// use iota_wallet::{
//     iota_client::constants::SHIMMER_COIN_TYPE,
//     secret::{stronghold::StrongholdSecretManager, },
//     ClientOptions, Result, account_manager::AccountManager,
// };

use std::path::PathBuf;
use anyhow::{Context};

use identity_iota::iota::block::output::AliasOutput;
use identity_iota::iota::IotaClientExt;
use identity_iota::iota::IotaDocument;
use identity_iota::iota::IotaIdentityClientExt;
use identity_iota::iota::NetworkName;
use identity_iota::storage::JwkDocumentExt;
use identity_iota::storage::JwkMemStore;
use identity_iota::storage::KeyIdMemstore;
use identity_iota::storage::Storage;
use identity_iota::verification::MethodScope;

use identity_iota::verification::jws::JwsAlgorithm;
use identity_stronghold::StrongholdStorage;
use iota_sdk::Wallet;
use iota_sdk::wallet::Result;
use iota_sdk::client::api::GetAddressesOptions;
use iota_sdk::client::node_api::indexer::query_parameters::QueryParameter;
use iota_sdk::client::secret::SecretManager;
use iota_sdk::client::Client;
use iota_sdk::crypto::keys::bip39;
use iota_sdk::types::block::address::Address;
use iota_sdk::types::block::address::Bech32Address;
use iota_sdk::types::block::address::Hrp;
use purity::utils::setup_secret_manager;
use purity::utils::setup_wallet;
use rand::distributions::DistString;

pub static API_ENDPOINT: &str = "http://localhost:14265";
pub static FAUCET_ENDPOINT: &str = "http://localhost:8091/api/enqueue";

pub type MemStorage = Storage<JwkMemStore, KeyIdMemstore>;


/// Creates a DID Document and publishes it in a new Alias Output.
///
/// Its functionality is equivalent to the "create DID" example
/// and exists for convenient calling from the other examples.
pub async fn create_did(
  client: &Client,
  secret_manager: &mut SecretManager,
  storage: &MemStorage,
  address: Address
) -> anyhow::Result<(Address, IotaDocument, String)> {
  // TODO: check and eventually remove this
  // let address: Address = get_address_with_funds(client, secret_manager, FAUCET_ENDPOINT)
  //   .await
  //   .context("failed to get address with funds")?;
  
  let network_name: NetworkName = client.network_name().await?;

  let (document, fragment): (IotaDocument, String) = create_did_document(&network_name, storage).await?;

  let alias_output: AliasOutput = client.new_did_output(address, document, None).await?;

  let document: IotaDocument = client.publish_did_output(secret_manager, alias_output).await?;

  Ok((address, document, fragment))
}

/// Creates an example DID document with the given `network_name`.
///
/// Its functionality is equivalent to the "create DID" example
/// and exists for convenient calling from the other examples.
pub async fn create_did_document(
  network_name: &NetworkName,
  storage: &MemStorage,
) -> anyhow::Result<(IotaDocument, String)> {
  let mut document: IotaDocument = IotaDocument::new(network_name);

  let fragment: String = document
    .generate_method(
      storage,
      JwkMemStore::ED25519_KEY_TYPE,
      JwsAlgorithm::EdDSA,
      None,
      MethodScope::VerificationMethod,
    )
    .await?;

  Ok((document, fragment))
}

// /// Generates an address from the given [`SecretManager`] and adds funds from the faucet.
// pub async fn get_address_with_funds(
//   client: &Client,
//   stronghold: &mut SecretManager,
//   faucet_endpoint: &str,
// ) -> anyhow::Result<Address> {
//   let address: Address = get_address(client, stronghold).await?;

//   request_faucet_funds(
//     client,
//     address,
//     client.get_bech32_hrp().await?.as_str(),
//     faucet_endpoint,
//   )
//   .await
//   .context("failed to request faucet funds")?;

//   Ok(address)
// }

// /// Initializes the [`SecretManager`] with a new mnemonic, if necessary,
// /// and generates an address from the given [`SecretManager`].
// pub async fn get_address(client: &Client, secret_manager: &mut SecretManager) -> anyhow::Result<Address> {
//   let keypair = KeyPair::new(KeyType::Ed25519)?;
//   let mnemonic =
//     iota_client::crypto::keys::bip39::wordlist::encode(keypair.private().as_ref(), &bip39::wordlist::ENGLISH)
//       .map_err(|err| anyhow::anyhow!(format!("{err:?}")))?;

//   if let SecretManager::Stronghold(ref mut stronghold) = secret_manager {
//     match stronghold.store_mnemonic(mnemonic).await {
//       Ok(()) => (),
//       Err(iota_client::Error::StrongholdMnemonicAlreadyStored) => (),
//       Err(err) => anyhow::bail!(err),
//     }
//   } else {
//     anyhow::bail!("expected a `StrongholdSecretManager`");
//   }

//   let address = client.get_addresses(secret_manager).with_range(0..1).get_raw().await?[0];

//   Ok(address)
// }

// /// Requests funds from the faucet for the given `address`.
// async fn request_faucet_funds(client: &Client, address: Bech32Address, faucet_endpoint: &str) -> anyhow::Result<()> {
//   iota_sdk::client::request_funds_from_faucet(faucet_endpoint, &address).await?;

//   tokio::time::timeout(std::time::Duration::from_secs(45), async {
//     loop {
//       tokio::time::sleep(std::time::Duration::from_secs(5)).await;

//       let balance = get_address_balance(client, &address)
//         .await
//         .context("failed to get address balance")?;
//       if balance > 0 {
//         break;
//       }
//     }
//     Ok::<(), anyhow::Error>(())
//   })
//   .await
//   .context("maximum timeout exceeded")??;

//   Ok(())
// }

// /// Returns the balance of the given Bech32-encoded `address`.
// async fn get_address_balance(client: &Client, address: &Bech32Address) -> anyhow::Result<u64> {
//   let output_ids = client
//     .basic_output_ids(vec![
//       QueryParameter::Address(address.to_owned()),
//       QueryParameter::HasExpiration(false),
//       QueryParameter::HasTimelock(false),
//       QueryParameter::HasStorageDepositReturn(false),
//     ])
//     .await?;

//   let outputs = client.get_outputs(&output_ids).await?;

//   let mut total_amount = 0;
//   for output_response in outputs {
//     total_amount += output_response.output().amount();
//   }

//   Ok(total_amount)
// }

// SPDX-FileCopyrightText: 2023 Fondazione LINKS
//
// SPDX-License-Identifier: APACHE-2.0


// pub async fn recover_storage() -> Result<Storage<StrongholdStorage, StrongholdStorage>> {

//   // Create a `StrongholdStorage`.
//   // `StrongholdStorage` creates internally a `SecretManager` that can be
//   // referenced to avoid creating multiple instances around the same stronghold snapshot.
//   let stronghold_storage = StrongholdStorage::new(secret_manager);
//   // Create storage for key-ids and JWKs.
//   //
//   // In this example, the same stronghold file that is used to store
//   // key-ids as well as the JWKs.
//   let storage: Storage<StrongholdStorage, StrongholdStorage> = Storage::new(stronghold_storage.clone(), stronghold_storage.clone());

//   Ok(storage)
// }

// pub async fn setup_secret_manager(password: &str, path: &str, mnemonic: &str) -> Result<SecretManager> {
//   // Setup Stronghold secret_manager
//   let mut secret_manager = StrongholdSecretManager::builder()
//   .password(password)
//   .build(PathBuf::from(path))?;

//   // Only required the first time, can also be generated with `manager.generate_mnemonic()?`
//   // The mnemonic only needs to be stored the first time
//   match secret_manager.store_mnemonic(mnemonic.to_string()).await {
//     Ok(()) => log::info!("Stronghold mnemonic stored"),
//     Err(iota_client::Error::StrongholdMnemonicAlreadyStored) => log::info!("Stronghold mnemonic already stored"),
//     Err(error) => panic!("Error: {:?}", error)
//   }
 
//   Ok(SecretManager::Stronghold(secret_manager))
// }

// pub async fn setup_account_manager(secret_manager: SecretManager) -> Result<AccountManager> {

//   // Create the account manager with the secret_manager and client options
//   let client_options = ClientOptions::new()
//   .with_node(&env::var("NODE_URL")
//   .unwrap())?;

//   let account_manager = AccountManager::builder()
//     .with_secret_manager(secret_manager)
//     .with_client_options(client_options)
//     .with_coin_type(SHIMMER_COIN_TYPE)
//     .finish()
//     .await?;

//     // // TODO: create or retrieve a main account
//     // log::info!("Creating new account into the wallet...");
//     // // let server_account = account_manager.create_account()
//     // //     .with_alias("main-account".to_string())
//     // //     .finish()
//     // //     .await.unwrap();
//     // let server_account = account_manager.get_account("main-account".to_string()).await.unwrap();
//     // let _ = server_account.sync(None).await.unwrap();

//     // log::info!("Generating an address for the account...");
//     // let addresses = server_account.generate_addresses(1, None).await.unwrap();
//     // let address =  addresses[0].address().as_ref().clone();
//     // log::info!("Address generate... {:?}", address);

//     // log::info!("Requesting funds...");
//     // request_faucet_funds(
//     //     &client,
//     //     address.clone(),
//     //     client.get_bech32_hrp().await?.as_str(),
//     //     &env::var("FAUCET_URL").unwrap(),
//     // ).await.context("Failed to request faucet funds")?;

//   Ok(account_manager)
// }
