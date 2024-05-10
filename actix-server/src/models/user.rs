// SPDX-FileCopyrightText: 2023 Fondazione LINKS
//
// SPDX-License-Identifier: APACHE-2.0

use serde::{Serialize, Deserialize};

use super::asset::Asset;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User{
    pub did: String,
    pub fragment: String,
    pub assets: Vec<Asset>
}