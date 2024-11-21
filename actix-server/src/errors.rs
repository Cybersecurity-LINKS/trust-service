// SPDX-FileCopyrightText: 2023 Fondazione LINKS
//
// SPDX-License-Identifier: APACHE-2.0

use actix_multipart::MultipartError;
use actix_web::{HttpResponse, ResponseError, http::header::ContentType};
use reqwest::StatusCode;

#[derive(thiserror::Error, Debug)]
pub enum TrustServiceError {

    #[error("Error while writing a proof")]  
    WriteProofError,
    #[error("User did not found")]   
    UserDidNotFound,
    #[error("Proof not found")]   
    ProofNotFound,
    #[error("Proof id not found")]   
    ProofIdNotFound,
    #[error("Asset id: {0} not found")]   
    AssetIdNotFound(String),
    #[error("Iota Client Error")]
    IotaClientError(#[from] iota_sdk::client::Error),
    #[error("Resolve Error")]
    ResolveError(#[from] identity_iota::iota::Error),
    #[error("Identity Core Error")]
    IdentityCoreError(#[from] identity_iota::core::Error),
    #[error("Wallet Error")]
    WalletError(#[from] iota_sdk::wallet::Error),
    #[error("Did Error")]
    DidError(#[from] identity_iota::did::Error),
    #[error("Error during insert")]   
    InsertError,
    #[error("Jwk error")]
    JwkError(#[from]identity_iota::storage::JwkStorageDocumentError),
    #[error("Mongo db Error")]
    MongoDbError(#[from]mongodb::error::Error),
    #[error("Smart Contract address recovery Error")]
    ContractAddressRecoveryError,
    #[error("Contract error: {0}")]
    ContractError(String),
    #[error("Attempting to interact with a non-existent smart contract")]
    MissingNftAddress,
    #[error("Error converting to Address")]
    AddressError,
    
    #[error("Error converting OutputId")]
    IotaBlockError(#[from]identity_iota::iota::block::Error),
    #[error("Proof signature not valid")]
    ProofSignatureNotValid,
    #[error("Error serde_json")]
    SerdeJsonError(#[from]serde_json::Error),
    #[error("Error: {0}")]
    CustomError(String),
    #[error("Generic error")]
    GenericError(#[from] anyhow::Error),
    #[error("File not Found in Mongo")]
    MongoFileNotFound,
    #[error("IPFS upload error")]
    IpfsUploadError,
    #[error("IPFS connection error")]
    IpfsConnError,

    #[error("Multipart error: {0}")]
    MultipartError(String),
    
    #[error("Error opening file")]
    FileOpenError,
    #[error("Error while writing file")]
    FileWriteError,
}

impl From<MultipartError> for TrustServiceError {
    fn from(err: MultipartError) -> TrustServiceError {
        TrustServiceError::MultipartError(err.to_string())
    }
}

impl ResponseError for TrustServiceError {

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::html())
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            TrustServiceError::ProofNotFound => StatusCode::NOT_FOUND,
            TrustServiceError::IotaClientError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            TrustServiceError::ResolveError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            TrustServiceError::DidError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            TrustServiceError::InsertError => StatusCode::INTERNAL_SERVER_ERROR,
            TrustServiceError::JwkError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            TrustServiceError::UserDidNotFound => StatusCode::NOT_FOUND,
            TrustServiceError::MongoDbError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            TrustServiceError::ProofIdNotFound => StatusCode::NOT_FOUND,
            TrustServiceError::AssetIdNotFound(_) => StatusCode::NOT_FOUND,
            TrustServiceError::IotaBlockError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            TrustServiceError::SerdeJsonError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            TrustServiceError::ProofSignatureNotValid => StatusCode::NOT_ACCEPTABLE,
            TrustServiceError::IdentityCoreError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            TrustServiceError::WalletError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            TrustServiceError::WriteProofError => StatusCode::INTERNAL_SERVER_ERROR,
            TrustServiceError::GenericError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            TrustServiceError::ContractAddressRecoveryError => StatusCode::INTERNAL_SERVER_ERROR,
            TrustServiceError::ContractError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            TrustServiceError::MissingNftAddress => StatusCode::BAD_REQUEST,
            TrustServiceError::AddressError => StatusCode::INTERNAL_SERVER_ERROR,
            TrustServiceError::CustomError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            TrustServiceError::MongoFileNotFound => StatusCode::NOT_FOUND,
            TrustServiceError::IpfsUploadError => StatusCode::INTERNAL_SERVER_ERROR,
            TrustServiceError::IpfsConnError => StatusCode::INTERNAL_SERVER_ERROR,
            TrustServiceError::MultipartError(_) => StatusCode::BAD_REQUEST,
            TrustServiceError::FileOpenError => StatusCode::INTERNAL_SERVER_ERROR,
            TrustServiceError::FileWriteError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}