use serde::{Serialize, Deserialize};
use mongodb::bson::{Bson, Document};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Proof{
    pub proof_id: String,
    pub asset_id: String
}

impl From<Proof> for Bson {
    fn from(proof: Proof) -> Self {
        let mut document = Document::new();
        document.insert("proof_id", proof.proof_id);
        document.insert("asset_id", proof.asset_id);
        Bson::Document(document)
    }
}