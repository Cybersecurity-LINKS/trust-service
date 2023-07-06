use std::env;
use anyhow::{Result, Context}
;
use identity_iota::did::DID;
use identity_iota::iota::{IotaDocument};
use identity_iota::prelude::KeyPair;

use iota_client::block::address::Address;
use iota_client::Client;
use iota_wallet::account_manager::{AccountManager};

use mongodb::{Database, bson::doc};

use crate::models::user::User;
use crate::utils::create_did as create_did_identity;
use crate::utils::request_faucet_funds;
use crate::USER_COLL_NAME;

// TODO: handle failures and rollback
pub async fn create_did(account_manager: &mut AccountManager, mongo_db: Database) -> Result<()>  {

    let secret_manager = account_manager.get_secret_manager();
    let client = Client::builder().with_primary_node(&env::var("NODE_URL").unwrap(), None).unwrap().finish().unwrap();

    log::info!("Creating DID...");

    let (_, iota_document, key_pair_connector): (Address, IotaDocument, KeyPair) =
        match create_did_identity(&client, &mut *secret_manager.write().await).await {
            Ok(result) => result,
            Err(error) => return Err(error)
        };
    log::info!("{:#}", iota_document);
    
    log::info!("Storing information in db..."); // TODO: this will become a keycloak request   
    let collection = mongo_db.collection::<User>(USER_COLL_NAME); 
    
    // let user = doc! { "did": iota_document.id().as_str() , "private_key": hex::encode(key_pair_connector.private().as_ref()) };
    let user = User { did: iota_document.id().to_string() , private_key: hex::encode(key_pair_connector.private().as_ref()), proofs: None };

    let result = collection.insert_one(user, None).await;
    let _ = match result {
        Ok(result) => result,
        Err(error) => {
            log::info!("{}", error);
            return Err(error.into())
        }
    };

    // Create a new account for that user
    let account = account_manager
        .create_account()
        .with_alias(iota_document.id().to_string())
        .finish()
        .await?;

    let addresses = account.generate_addresses(1, None).await?;
    let address =  addresses[0].address();
    request_faucet_funds(
        &client,
        address.as_ref().clone(),
        client.get_bech32_hrp().await?.as_str(),
        &env::var("FAUCET_URL").unwrap(),
    ).await.context("Failed to request faucet funds")?;

    let filter = doc! {"did": iota_document.id().as_str()};

    let result = collection.find_one(Some(filter), None).await.unwrap();
    
    match result {
        Some(user) => {
            println!("{:?}", user)
        },
        None => println!("No document found"),
    }

    

    Ok(())
}
