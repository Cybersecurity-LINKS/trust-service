use serde::{Serialize, Deserialize};

use super::proof::Proof;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User{
    pub did: String,
    pub private_key: String,
    pub proofs: Vec<Proof>
}