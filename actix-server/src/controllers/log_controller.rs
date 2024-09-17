use std::env;
use std::io::{Cursor};
use actix_web::{get, post};
use actix_web::{web, HttpResponse, Error};
use actix_multipart::Multipart;
use actix_web::http::uri::Scheme;
use futures_util::{StreamExt as _, TryStreamExt};
use ipfs_api_backend_actix::{IpfsApi, IpfsClient, TryFromUri};
use log::log;
use crate::errors::TrustServiceError;
use crate::models::log_model::Log;
use crate::services::mongodb_repo::MongoRepo;

/// This API takes a file, checks that the filename is correct,
/// then it publishes that file to IPFS.
/// It gets the CID from IPFS, then stores the CID in the DB.
/// Update the document in the DB with the same name or create it.
#[post("")]
async fn publish_log(mut payload: Multipart, mongodb_repo: web::Data<MongoRepo>) -> Result<HttpResponse, TrustServiceError> {

    let mut file_count = 0;
    let mut file_data = Vec::new();
    let mut filename = String::new();

    while let Some(item) = payload.next().await {
        // Read a file for each loop
        file_count += 1;// increment the number of files
        if file_count > 1 { // if more that a loop is executed, error just one file allowed
            return Ok(HttpResponse::BadRequest().body("Only one file allowed"));
        }

        let mut field = item?;
        if let Some(content_disposition) = field.content_disposition() {
            if let Some(fname) = content_disposition.get_filename() {
                filename = fname.to_string();
                if filename == "" { file_count -= 1}//if the name is empy no file was sent decrement file_count

                // this 'while' stores the file in the file_data variable
                while let Some(chunk) = field.next().await {
                    let data = chunk?;
                    file_data.extend_from_slice(&data);
                }
            } else {
                return Ok(HttpResponse::BadRequest().body("Cannot read filename"));
            }
        } else {
            return Ok(HttpResponse::BadRequest().body("Content-Disposition missing"));
        }
    }

    // No file check
    if file_count == 0 {
        return Ok(HttpResponse::BadRequest().body("Missing file"));
    }

    // Verification of the file name
    if filename !=  std::env::var("LOG_FILE_NAME")
        .expect("$LOG_FILE_NAME must be set.") {
        return Ok(HttpResponse::BadRequest().body("Wrong file"));
    }

    // Upload the file on IPFS
    let ipfs_client =
        if env::var("RUNNING_IN_DOCKER").is_ok(){
            IpfsClient::from_host_and_port(Scheme::HTTP, "ipfs", 5001).unwrap()
        } else {
            IpfsClient::default()
        };

    let data = Cursor::new(file_data);
    let add_result = ipfs_client.add(data).await;

    let cid = match add_result {
        Ok(res) => {
            //log::info!("{:?}", res);
            log::info!("File uploaded to IPFS with cid: {}", res.hash);
            Ok((res.hash))
        },
        Err(_e) => Err(TrustServiceError::IpfsUploadError),
    };

    let log_doc = Log {
        name: filename,
        cid: cid?,
    };

    mongodb_repo.store_log_cid(log_doc).await?;


    Ok(HttpResponse::Ok().body("File uploaded successfully"))
}

/// This API retrieves the log file from IPFS
/// It reads the CID from the DB
/// The document that contains the CID has a fixed field called name.
/// So having that name allows to find the CID
#[get("")]
async fn get_log(mongodb_repo: web::Data<MongoRepo>) -> Result<HttpResponse, Error> {

    // get the CID from the DB
    let cid = mongodb_repo.get_log_cid().await?;

    let ipfs_client =
        if env::var("RUNNING_IN_DOCKER").is_ok(){
            IpfsClient::from_host_and_port(Scheme::HTTP, "ipfs", 5001).unwrap()
        } else {
            IpfsClient::default()
        };

    log::info!("Retrieving from IPFS");

    // retrieve from IPFS
    let file = ipfs_client
        .cat(&cid)
        .map_ok(|chunk| chunk.to_vec())
        .try_concat()
        .await
        .map_err(|_| "error reading full file");

    match file {
        Ok(data) => Ok(HttpResponse::Ok().body(data)),
        Err(e) => Ok(HttpResponse::InternalServerError().body(format!("Error: {}", e))),
    }
}

pub fn scoped_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        // prefixes all resources and routes attached to it...
        web::scope("/log")
            .service(publish_log)
            .service(get_log)
    );
}