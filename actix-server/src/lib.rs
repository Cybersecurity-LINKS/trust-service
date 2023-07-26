// SPDX-FileCopyrightText: 2023 Fondazione LINKS
//
// SPDX-License-Identifier: APACHE-2.0

use iota_client::{secret::SecretManager, Client};
use iota_wallet::account_manager::AccountManager;
use std::sync::{Mutex, RwLock, Arc};

pub mod controllers;
pub mod dtos;
pub mod services;
pub mod models;
pub mod utils;
pub mod errors;

const PROOF_TAG: &str = "proofs-2"; // TODO: define this somewhere else
const DB_NAME: &str = "MODERATE"; // TODO: define this somewhere else
const USER_COLL_NAME: &str = "Users"; // TODO: define this somewhere else
const MAIN_ACCOUNT: &str = "main-account";

// This struct represents state
pub struct AppIotaState {
    pub account_manager: Arc<RwLock<AccountManager>>
}