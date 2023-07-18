use iota_client::{secret::SecretManager, Client};
use iota_wallet::account_manager::AccountManager;
use std::sync::{Mutex, RwLock, Arc};

pub mod controllers;
pub mod dtos;
pub mod services;
pub mod models;
pub mod utils;
pub mod errors;

const PROOF_TAG: &str = "proofs-2";
const DB_NAME: &str = "MODERATE";
const USER_COLL_NAME: &str = "Users"; // TODO: define this somewhere else


// This struct represents state
pub struct AppIotaState {
    pub account_manager: Arc<RwLock<AccountManager>>
}