use mongodb::Client as MongoClient;
use async_trait::async_trait;
use anyhow::{Result, Context};
use anyhow::anyhow;
use mongodb::bson::doc;
use mongodb::options::FindOneOptions;
use mongodb::results::{InsertOneResult, UpdateResult};
use serde_json::Value;
use crate::dtos::proof_dto;
use crate::models::proof::Proof;
use crate::{USER_COLL_NAME, PROOF_TAG, DB_NAME};
use crate::models::user::User;
use crate::keycloak::{KeycloakSession, create_user, update_user_attrs};

#[derive(Clone)]
pub enum StorageType {
    MongoDB(MongoClient),
    Keycloak(KeycloakSession)
}

impl StorageType {

    pub async fn store_user_key(&self, user: User) -> Result<User> {
        match self {
            StorageType::MongoDB(mongo_client) => {
                let mongo_db = mongo_client.database(DB_NAME);
                let collection = mongo_db.collection::<User>(USER_COLL_NAME); 
                // collection.insert_one(user.clone(), None).await.map_err(anyhow::Error::from);
                let _ = match collection.insert_one(user.clone(), None).await.map_err(anyhow::Error::from) {
                    Ok(result) => result,
                    Err(error) => {
                        log::info!("{}", error);
                        return Err(error.into())
                    }
                };
                Ok(user)
            },
            StorageType::Keycloak(keycloak_session) => {
                // Create a new user.
                // Users in Keycloak are entities that represent individuals who can
                // authenticate and access applications or services within a realm.

                let user_data = create_user(keycloak_session).await.unwrap();
                log::info!("User created:\n{:#}", user_data);
                let username = user_data["username"].as_str().unwrap().to_string();
                log::info!("{:?}", user);
                Ok(user)
            },
            _ => {
                todo!("Hello!");
            }
            }
    }
    

    pub async fn get_user_key(&self, did: &str) -> User {
        match self {
            StorageType::MongoDB(mongo_client) => {
                let mongo_db = mongo_client.database(DB_NAME);
                let collection = mongo_db.collection::<User>(USER_COLL_NAME); 
                let filter = doc! {"did": did};

                collection.find_one(Some(filter.clone()), None).await.unwrap().unwrap()
            },
            StorageType::Keycloak(keycloak_session) => todo!(),
        }
    }
    
    pub async fn store_proof_info(&self, did: &str, proof: Proof) -> Result<UpdateResult>{
        match self {
            StorageType::MongoDB(mongo_client) => {
                let mongo_db = mongo_client.database(DB_NAME);
                let collection = mongo_db.collection::<User>(USER_COLL_NAME); 
                let filter = doc! {"did": did};

                let update = doc! {
                    "$push": {
                        "proofs": proof
                    }
                };
                collection.update_one(filter, update, None).await.map_err(anyhow::Error::from)

            },
            StorageType::Keycloak(keycloak_session) => {
                
                // These are an example of arbitrary attributes that can be added to a user.
                //     let additional_attrs = json!({
                //         "private_key": general_purpose::STANDARD.encode(rand::thread_rng().gen::<[u8; 32]>()),
                //         "public_key": general_purpose::STANDARD.encode(rand::thread_rng().gen::<[u8; 32]>()),
                //         "did": "did:iota:LjA7K_x_08vmsiVkc4wD5n4_i0RmPm3gxCQoHe9IQIY",
                //     });

                // pdate the user with the arbitrary attributes.
                //     let updated_user_data = update_user_attrs(
                //         &http_client,
                //         &realm_name,
                //         &username,
                //         &admin_token,
                //         &additional_attrs,
                //     ).await.unwrap();

                //     log::info!("User updated with arbitrary attributes:\n{:#}", updated_user_data);
                todo!()
            }
    }
    }

    pub async fn get_proof_id(&self, asset_id: String) -> Result<String> {
        match self {
            StorageType::MongoDB(mongo_client) => {
                let mongo_db = mongo_client.database(DB_NAME);
                let collection = mongo_db.collection::<User>(USER_COLL_NAME); 
                let projected_collection = collection.clone_with_type::<Value>();
                log::info!("Searching for asset: {:#?}", asset_id);

                // Define the filter query
                let filter = doc! {
                    "proofs": {
                    "$elemMatch": {
                            "assetId": asset_id.clone()
                        }
                    }
                };

                // Define the projection query
                // Use FindOptions::builder() to set the projection
                let find_options = FindOneOptions::builder().projection(doc! {
                    "proofs.$": 1,
                    "_id": 0
                }).build();

                let proof_id = match projected_collection.find_one(Some(filter), find_options).await {
                    Ok(Some(user)) => {
                        if let Some(proof_id) = user["proofs"][0]["proofId"].as_str() {
                            log::info!("proofId: {}", proof_id);
                            Ok(proof_id.to_owned())
                        } else {
                            Err(anyhow!("ProofId not found in the document."))
                        }      
                    },
                    Ok(None) => Err(anyhow!("No asset found with id: {}", asset_id)),
                    Err(err) => Err(anyhow!("Error: {}", err))
                };
                proof_id
            },
            StorageType::Keycloak(keycloak_session) => todo!()
        }
    }

}

