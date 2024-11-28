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

    pub async fn add_file(&self, file_path: &str) -> Result<String, TrustServiceError> {
        let path = Path::new(file_path);
        let file = File::open(&path).map_err(|e| TrustServiceError::FileOpenError)?;

        let res = self.client.add(file).await.map_err(|e| TrustServiceError::IpfsUploadError)?;
        Ok(res.hash)
    }

    pub async fn get_file(&self, cid: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
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
}