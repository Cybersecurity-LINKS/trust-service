// SPDX-FileCopyrightText: 2023 Fondazione LINKS
//
// SPDX-License-Identifier: APACHE-2.0

use std::env;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use actix_web::http::uri::Scheme;
use actix_web::HttpResponse;
use futures_util::TryStreamExt;
use ipfs_api_backend_actix::{IpfsApi, IpfsClient, TryFromUri};
use crate::errors::TrustServiceError;

pub struct IpfsService {
    client: IpfsClient,
}

impl IpfsService {
    pub fn new() -> Self {

        /// Create a new IPFS client
        let ipfs_client =
            if env::var("RUNNING_IN_DOCKER").is_ok(){
                IpfsClient::from_host_and_port(Scheme::HTTP, "ipfs", 5001).unwrap()
            } else {
                IpfsClient::default()
            };
        
        IpfsService {
            client: ipfs_client,
        }
    }

    /// Push a file to IPFS, given its path
    pub async fn add_file(&self, file_path: &str) -> Result<String, TrustServiceError> {
        let path = Path::new(file_path);
        let file = File::open(&path).map_err(|e| TrustServiceError::FileOpenError)?;

        let res = self.client.add(file).await.map_err(|e| TrustServiceError::IpfsUploadError)?;
        Ok(res.hash)
    }

    /// Retrieve a file from IPFS given its CID
    pub async fn get_file(&self, cid: &str) -> Result<Vec<u8>, TrustServiceError> {
        // retrieve from IPFS
        let file = self.client
            .cat(&cid)
            .map_ok(|chunk| chunk.to_vec())
            .try_concat()
            .await
            .map_err(|_| TrustServiceError::IpfsReadError);

        match file {
            Ok(data) => Ok(data),
            Err(e) => Err(e.into()),
        }
    }

    /// Delete a file from the local IPFS node
    pub async fn delete_file(&self, cid: &str) -> Result<(), TrustServiceError> {
        //unpin a file in IPFS = tell the garbage collector that the file can be deleted if space is needed,
        // if the garbage collector has not yet deleted the file it is still possible to retrieve it, because it is not actually deleted.
        let res = self.client.pin_rm(&cid, true).await;
        match res { 
            Ok(_) => {
                log::info!("File unpin ok {}", cid);
                log::info!("PinRm Response: {:?}", res.unwrap());
                log::info!("IPFS unpinned file: {:?}", cid);
            },
            Err(e) => {
                // Log as warning instead of error - this is not a fatal condition
                // The file might already be unpinned, not pinned directly, or garbage collected
                log::warn!("File unpin failed for CID {}: {}. This is not critical - continuing with cleanup.", cid, e);
            }
        }
        
        //block_rm delete a file in IPFS before the garbage collector does it, after this, the file is deleted and cannot be restored
        let res = self.client.block_rm(&cid).await;
        match res {
            Ok(response) => {
                log::info!("BlockRm Response: {:?}", response);
                log::info!("IPFS removed file block: {:?}", cid);
            },
            Err(e) => {
                // Log as warning - the block might already be removed or not exist
                log::warn!("Block removal failed for CID {}: {}. Block may already be removed.", cid, e);
            }
        }

        Ok(())
    }
}