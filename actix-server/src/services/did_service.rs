use std::env;
use actix_web::web;
use anyhow::Result;
use identity_iota::did::DID;
use identity_iota::iota::{IotaDocument};
use identity_iota::prelude::KeyPair;
use iota_client::block::address::Address;
use iota_client::secret::SecretManager;
use iota_wallet::account_manager::{AccountManager, self};
use crate::models::user::User;
use crate::utils::create_did as create_did_identity;
use iota_client::Client;
use mongodb::{Client as MongoClient, Database, bson};
use mongodb::bson::{doc, Bson};

const COLL_NAME: &str = "Users"; // TODO: define this somewhere else


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
    let collection = mongo_db.collection::<User>(COLL_NAME); 
    
    // let user = doc! { "did": iota_document.id().as_str() , "private_key": hex::encode(key_pair_connector.private().as_ref()) };
    let user = User { did: iota_document.id().to_string() , private_key: hex::encode(key_pair_connector.private().as_ref()) };

    let result = collection.insert_one(user, None).await;
    let _ = match result {
        Ok(result) => result,
        Err(error) => {
            log::info!("{}", error);
            return Err(error.into())
        }
    };

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
