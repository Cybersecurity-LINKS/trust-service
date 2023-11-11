// SPDX-FileCopyrightText: 2023 Fondazione LINKS
//
// SPDX-License-Identifier: APACHE-2.0

use std::env;
use anyhow::{Result, Context};
use identity_iota::did::DID;
use identity_iota::iota::IotaDocument;
use identity_iota::prelude::{IotaIdentityClientExt, IotaDID};
use identity_iota::storage::Storage;
use identity_stronghold::StrongholdStorage;
use iota_sdk::types::block::address::{Address, Bech32Address};
use iota_sdk::client::Client;
use iota_sdk::Wallet;
use mongodb::Database;
use mongodb::bson::doc;
use purity::utils::request_faucet_funds;
// use mongodb::bson::Bson;
// use mongodb::bson::doc;

use crate::models::user::User;
// use crate::models::user::User;
use crate::utils::{create_did as create_did_identity, MemStorage};
// use purity::utils::request_faucet_funds;
use crate::{MAIN_ACCOUNT, USER_COLL_NAME};

// use aes_gcm::{
//     aead::{Aead, AeadCore, KeyInit, OsRng},
//     Aes256Gcm, Nonce, Key // Or `Aes128Gcm`
// };
// use base64::{Engine as _, engine::general_purpose};

// TODO: handle failures and rollback
pub async fn create_did(wallet: &mut Wallet, key_storage: &mut MemStorage, mongo_db: Database) -> Result<String>  {

    let secret_manager = wallet.get_secret_manager();
    let account = wallet.get_account(MAIN_ACCOUNT).await?;
    let _ = account.sync(None).await?;
    let governor_address = account.addresses_with_unspent_outputs().await?[0].address().clone();
    log::info!("Main account address: {}", governor_address);

    let client = Client::builder().with_primary_node(&env::var("NODE_URL").unwrap(), None).unwrap().finish().await?;

    log::info!("Creating DID...");
    

    let (_, iota_document, fragment): (Address, IotaDocument, String) =
        match create_did_identity(&client, &mut *secret_manager.write().await, key_storage, governor_address.inner().clone()).await {
            Ok(result) => {
                let _ = account.sync(None).await?;
                result
            },
            Err(error) => {
                log::info!("{:?}", error);
                return Err(error)
            }
        };
    log::info!("{:#}", iota_document);
    
    log::info!("Storing information in db... TODO:!"); // TODO: this will become a keycloak request   
    let collection = mongo_db.collection::<User>(USER_COLL_NAME); 
    
    // //TODO: understand if this should be in global state
    // let aes_key_vec = general_purpose::STANDARD.decode(&env::var("ENC_KEY").expect("$ENC_KEY must be set."))?;
    // let aes_key = Key::<Aes256Gcm>::from_slice(aes_key_vec.as_slice());
    // let cipher = Aes256Gcm::new(aes_key);
    // let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

    // let user = doc! { "did": iota_document.id().as_str() , "private_key": hex::encode(key_pair_connector.private().as_ref()) };
    // let user = User { did: iota_document.id().to_string(), private_key: hex::encode(key_pair_connector.private().as_ref()), proofs: vec![] };
    // log::info!("sk: {:?}", key_pair_connector.private().as_ref());
    // let user = User { did: iota_document.id().to_string(), nonce: nonce.to_vec(),  private_key: cipher.encrypt(&nonce, key_pair_connector.private().as_ref()).unwrap(), proofs: vec![] };
    let user = User { did: iota_document.id().to_string(), fragment: fragment, proofs: vec![] };

    let result = collection.insert_one(user, None).await;
    let _ = match result {
        Ok(result) => result,
        Err(error) => {
            log::info!("{}", error);
            return Err(error.into())
        }
    };

    // Create a new account for that user
    log::info!("Creating new account into the wallet...");
    let account = wallet
    .create_account()
    .with_alias(iota_document.id().to_string())
    .finish()
    .await?;

    log::info!("Generating an address for the account...");
    let address = &account.generate_ed25519_addresses(1, None).await?[0];
    // TODO: check and eventually remove this 
    log::info!("Requesting funds...");
    request_faucet_funds(&client, address.address(), &env::var("FAUCET_URL").unwrap()).await.context("Failed to request faucet funds")?;
    let _ = account.sync(None).await?;

    let filter = doc! {"did": iota_document.id().as_str()};
    let result = collection.find_one(Some(filter), None).await.unwrap();
    match result {
        Some(user) => {
            println!("{:?}", user)
        },
        None => println!("No document found"),
    }

    Ok(iota_document.id().to_string())
}

pub async fn get_did_doc(did: String) -> Result<String> {
    let client = Client::builder().with_primary_node(&env::var("NODE_URL").unwrap(), None).unwrap().finish().await?;
    log::info!("Resolving did...");
    let iota_document: IotaDocument = client.resolve_did(&IotaDID::try_from(did.as_str()).unwrap()).await?;
    Ok(iota_document.to_string())
}