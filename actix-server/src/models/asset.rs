// SPDX-FileCopyrightText: 2023 Fondazione LINKS
//
// SPDX-License-Identifier: APACHE-2.0

use serde::{Serialize, Deserialize};
use mongodb::bson::{Bson, Document};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Asset{
    pub asset_id: String,
    pub proof_id: String,
    pub nft_addr: Option<String>,
}

impl From<Asset> for Bson {
    fn from(asset: Asset) -> Self {
        let mut document = Document::new();
        document.insert("assetId", asset.asset_id);
        document.insert("proofId", asset.proof_id);
        document.insert("nftAddr", asset.nft_addr);
        Bson::Document(document)
    }
}