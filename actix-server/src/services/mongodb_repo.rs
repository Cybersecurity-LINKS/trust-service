use std::env;
use anyhow::Result;

use mongodb::options::UpdateOptions;
use mongodb::Collection;
use mongodb::Client as MongoClient;
use mongodb::bson::doc;
use mongodb::options::FindOneOptions;
use mongodb::results::InsertOneResult;
use serde_json::Value;

use crate::errors::TrustServiceError;
use crate::models::asset::Asset;
use crate::models::user::User;
use crate::models::log_model::Log;

pub struct MongoRepo {
    user_collection: Collection<User>,
    log_collection: Collection<Log>
}

pub const USER_COLL_NAME: &str = "Users";
pub const LOG_COLL_NAME: &str = "Log_IPFS";

impl MongoRepo {
    pub async fn init() -> Self {
        log::info!("Init mongo");
        
        let mongo_usr = std::env::var("MONGO_INITDB_ROOT_USERNAME")
        .expect("$MONGO_INITDB_ROOT_USERNAME must be set.");
        let mongo_pass = std::env::var("MONGO_INITDB_ROOT_PASSWORD")
        .expect("$MONGO_INITDB_ROOT_PASSWORD must be set.");

        let mut mongo_endpoint="".to_string();
        if env::var("RUNNING_IN_DOCKER").is_ok(){
            mongo_endpoint = std::env::var("MONGO_ENDPOINT_D")
                .expect("$MONGO_ENDPOINT_D must be set.");
        } else {
            mongo_endpoint = std::env::var("MONGO_ENDPOINT_L")
                .expect("$MONGO_ENDPOINT_L must be set.");
        };



        let mongo_uri = format!("mongodb://{mongo_usr}:{mongo_pass}@{mongo_endpoint}");
        let mongo_client = MongoClient::with_uri_str(mongo_uri)
        .await
        .expect("failed to connect to database");

        let mongo_database = std::env::var("MONGO_DATABASE")
        .expect("$MONGO_DATABASE must be set.");

        let db = mongo_client.database(mongo_database.as_str());
        let user_collection: Collection<User> = db.collection(USER_COLL_NAME);
        let log_collection: Collection<Log> = db.collection(LOG_COLL_NAME);

        MongoRepo { user_collection, log_collection }
    }

    pub async fn store_user(&self, user: User) -> Result<InsertOneResult, TrustServiceError> {
        log::info!("Storing information in db...");
        let new_user = User { 
            did: user.did, 
            fragment: user.fragment, 
            assets: vec![] 
        };
       
        match self.user_collection.insert_one(new_user, None).await {
            Ok(id) => Ok(id),
            Err(err) => {
                log::info!("{}", err.to_string());
                return Err(TrustServiceError::InsertError)
            }
        }
        
    }

    pub async fn get_user(&self, did: &str) -> Result<User, TrustServiceError> {
        log::info!("Getting User information from db...");
        
        let filter = doc! {"did": did};

        match self.user_collection.find_one(Some(filter.clone()), None).await? {
            Some(user) => {
                Ok(user)
            },
            None => Err(TrustServiceError::UserDidNotFound),
        }
    
    }

    pub async fn get_asset(&self, asset_id: String) -> Result<Asset, TrustServiceError> {
    
        log::info!("Getting Asset information from db...");
        let projected_collection = self.user_collection.clone_with_type::<Value>();
        log::info!("Searching for asset: {:#?}", asset_id);

        // Define the filter query
        let filter = doc! {
            "assets": {
                "$elemMatch": {
                    "assetId": asset_id.clone()
                }
            }
        };

        // Define the projection query, use FindOptions::builder() to set the projection
        let find_options = FindOneOptions::builder().projection(doc! {
            "assets.$": 1,
            "_id": 0
        }).build();

        match projected_collection.find_one(Some(filter), find_options).await {
            Ok(Some(user)) => {
                Ok(serde_json::from_value(user["assets"][0].clone())?)
                   
            },
            Ok(None) => Err(TrustServiceError::AssetIdNotFound(asset_id)),
            Err(err) => Err(TrustServiceError::MongoDbError(err))
        }
    }

    pub async fn store_nft_addr(&self, asset_id: String, nft_addr: String) -> Result<Asset, TrustServiceError> {
    
        log::info!("Updating Asset {:#?} information...", asset_id);
        let projected_collection = self.user_collection.clone_with_type::<Value>();

        // Define the filter query
        let filter = doc! {
            "assets": {
                "$elemMatch": {
                    "assetId": asset_id.clone()
                }
            }
        };

        // Define update options
        let options = UpdateOptions::builder().upsert(false).build();

        // Define the update operation
        let update = doc! { "$set": { 
                "assets.$.nftAddr": nft_addr
            }
        };

        let res =  projected_collection.update_one(filter, update, options).await.unwrap();
        println!("Updated documents: {}", res.modified_count);
        let ass = Asset{ asset_id, proof_id: "todo()!".to_string(), nft_addr:  None };
        Ok(ass)
        //  {
        //     Ok(user) => {
        //         Ok(serde_json::from_value(user["assets"][0].clone())?)
                   
        //     },
        //     Ok(None) => Err(TrustServiceError::AssetIdNotFound(asset_id)),
        //     Err(err) => Err(TrustServiceError::MongoDbError(err))
        // }
    }

    pub async fn store_proof_relationship(
        &self, 
        did: &str,
        proof_id: String, 
        asset_id: String,
    ) -> Result<(), TrustServiceError> {

        log::info!("Storing proof-asset relationship...");
        let filter = doc! {"did": did};

        let asset = Asset { proof_id, asset_id, nft_addr: None };
        let update = doc! {
            "$push": {
                "assets": asset
            }
        };

        self.user_collection.update_one(filter, update, None).await?;
        Ok(())
    }

    /// This function takes a log object and stores it in the Mongo DB.
    /// The purpose of the function is to update the CID of the log file
    /// stored in the DB. The DB matches a name to a CID.
    /// There should be only one document in the DB where the field, name,
    /// does not change, while the CID field is updated on each request.
    ///
    /// If the document does not exist, this function will create it.
    ///
    /// # Parameters
    /// * log - the document to save
    pub async fn store_log_cid(&self, log: Log) -> Result<(), TrustServiceError> {
        log::info!("Storing information in db...");
        log::info!("File name: {}, CID: {}", log.name, log.cid);

        // definition of a filter for the find_one_and_replace
        let filter = doc! { "name": &log.name };

        // look for the document to update or add
        // find_one_... because there must be only one document in the DB
        match self.log_collection.find_one_and_replace(filter, &log, None).await {
            Ok(Some(_)) => { //Document found and updated
                log::info!("Log Updated");
                Ok(())
            }
            Ok(None) => { // Document not found and inserted
                match self.log_collection.insert_one(&log, None).await {
                    Ok(_) => {
                        log::info!("Log Inserted");
                        Ok(())
                    }
                    Err(err) => Err(TrustServiceError::MongoDbError(err))
                }
            }
            Err(err) => Err(TrustServiceError::MongoDbError(err))
        }

    }

    /// This function returns the CID of the log file.
    /// There must be only one document in the DB inside the Log_IPFS collection.
    /// The name of this document is fixed in the .mongo.env file.
    /// This function reads the name of the file, looks for it in the DB and then
    /// returns the CID field present in that document.
    /// The CID is the identifier of the file within IPFS.
    pub async fn get_log_cid(&self) -> Result<String, TrustServiceError>{

        // read the filename of the log file from .mongo.env
        let log_filename = std::env::var("LOG_FILE_NAME")
            .expect("$LOG_FILE_NAME must be set.");

        // define the filter
        let filter = doc! { "name": log_filename };

        log::info!("Looking in the DB");

        // look for the document
        match self.log_collection.find_one(filter, None).await {
            Ok(Some(log)) => {// document found
                log::info!("Log file from the DB {:?}", log);
                Ok(log.cid)
            }
            Ok(None) => {// document not found
                Err(TrustServiceError::MongoFileNotFound)
            }
            Err(err) => {
                Err(TrustServiceError::MongoDbError(err))
            }
        }
    }
}