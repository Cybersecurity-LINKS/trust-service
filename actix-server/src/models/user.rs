use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct User{
    pub did: String,
    pub private_key: String
}