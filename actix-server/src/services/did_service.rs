// SPDX-FileCopyrightText: 2023 Fondazione LINKS
//
// SPDX-License-Identifier: APACHE-2.0

use std::env;
use anyhow::{Result, Context}
;
use identity_iota::did::DID;
use identity_iota::iota::{IotaDocument};
use identity_iota::prelude::{KeyPair, IotaIdentityClientExt, IotaDID};

use iota_client::block::address::Address;
use iota_client::Client;
use iota_wallet::account_manager::{AccountManager};

use mongodb::bson::Bson;
use mongodb::{Database, bson::doc};

use crate::models::user::User;
use crate::utils::create_did as create_did_identity;
use crate::utils::request_faucet_funds;
use crate::{USER_COLL_NAME, MAIN_ACCOUNT};


use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Nonce, Key // Or `Aes128Gcm`
};
use base64::{Engine as _, engine::general_purpose};

// TODO: handle failures and rollback
pub async fn create_did(account_manager: &mut AccountManager, mongo_db: Database) -> Result<String>  {
    let account = account_manager.get_account(MAIN_ACCOUNT).await?;
    let _ = account.sync(None).await?;
    let governor_address = account.addresses().await?[0].address().clone();
    log::info!("Main account address: {}", governor_address.bech32_hrp());

    let secret_manager = account_manager.get_secret_manager();
    let client = Client::builder().with_primary_node(&env::var("NODE_URL").unwrap(), None).unwrap().finish().unwrap();

    log::info!("Creating DID...");

    let (_, iota_document, key_pair_connector): (Address, IotaDocument, KeyPair) =
        match create_did_identity(&client, &mut *secret_manager.write().await, governor_address.as_ref().clone()).await {
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
    
    log::info!("Storing information in db..."); // TODO: this will become a keycloak request   
    let collection = mongo_db.collection::<User>(USER_COLL_NAME); 
    
    //TODO: understand if this should be in global state
    let aes_key_vec = general_purpose::STANDARD.decode(&env::var("ENC_KEY").expect("$ENC_KEY must be set."))?;
    let aes_key = Key::<Aes256Gcm>::from_slice(aes_key_vec.as_slice());
    let cipher = Aes256Gcm::new(aes_key);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

    // let user = doc! { "did": iota_document.id().as_str() , "private_key": hex::encode(key_pair_connector.private().as_ref()) };
    // let user = User { did: iota_document.id().to_string(), private_key: hex::encode(key_pair_connector.private().as_ref()), proofs: vec![] };
    log::info!("sk: {:?}", key_pair_connector.private().as_ref());
    let user = User { did: iota_document.id().to_string(), nonce: nonce.to_vec(),  private_key: cipher.encrypt(&nonce, key_pair_connector.private().as_ref()).unwrap(), proofs: vec![] };

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
    let account = account_manager
        .create_account()
        .with_alias(iota_document.id().to_string())
        .finish()
        .await?;

    log::info!("Generating an address for the account...");
    let addresses = account.generate_addresses(1, None).await?;
    let address =  addresses[0].address();
    // TODO: check and eventually remove this 
    log::info!("Requesting funds...");
    request_faucet_funds(
        &client,
        address.as_ref().clone(),
        client.get_bech32_hrp().await?.as_str(),
        &env::var("FAUCET_URL").unwrap(),
    ).await.context("Failed to request faucet funds")?;

    // let filter = doc! {"did": iota_document.id().as_str()};

    // let result = collection.find_one(Some(filter), None).await.unwrap();
    
    // match result {
    //     Some(user) => {
    //         println!("{:?}", user)
    //     },
    //     None => println!("No document found"),
    // }

    

    Ok(iota_document.id().to_string())
}

pub async fn get_did_doc(did: String) -> Result<String> {
    let client = Client::builder().with_primary_node(&env::var("NODE_URL").unwrap(), None).unwrap().finish().unwrap();
    log::info!("Resolving did...");
    let iota_document: IotaDocument = client.resolve_did(&IotaDID::try_from(did.as_str()).unwrap()).await?;
    Ok(iota_document.to_string())
}