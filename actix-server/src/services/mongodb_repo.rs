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

pub struct MongoRepo {
    user_collection: Collection<User>,
}

pub const USER_COLL_NAME: &str = "Users";

impl MongoRepo {
    pub async fn init() -> Self {
        log::info!("Init mongo");
        
        let mongo_usr = std::env::var("MONGO_INITDB_ROOT_USERNAME")
        .expect("$MONGO_INITDB_ROOT_USERNAME must be set.");
        let mongo_pass = std::env::var("MONGO_INITDB_ROOT_PASSWORD")
        .expect("$MONGO_INITDB_ROOT_PASSWORD must be set.");
        let mongo_endpoint = std::env::var("MONGO_ENDPOINT")
        .expect("$MONGO_ENDPOINT must be set.");

        let mongo_uri = format!("mongodb://{mongo_usr}:{mongo_pass}@{mongo_endpoint}");
        let mongo_client = MongoClient::with_uri_str(mongo_uri)
        .await
        .expect("failed to connect to database");

        let mongo_database = std::env::var("MONGO_DATABASE")
        .expect("$MONGO_DATABASE must be set.");

        let db = mongo_client.database(mongo_database.as_str());
        let user_collection: Collection<User> = db.collection(USER_COLL_NAME);

        MongoRepo { user_collection }
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

}