// SPDX-FileCopyrightText: 2023 Fondazione LINKS
//
// SPDX-License-Identifier: APACHE-2.0

use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm// Or `Aes128Gcm`
};
use base64::{Engine as _, engine::general_purpose};

fn main() {
    
    // The encryption key can be generated randomly:
    let key = Aes256Gcm::generate_key(OsRng);
    let base64_key = general_purpose::STANDARD.encode(key);
    println!("{:?}", base64_key);
    
    let cipher = Aes256Gcm::new(&key);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng); // 96-bits; unique per message
    let ciphertext = cipher.encrypt(&nonce, b"plaintext message".as_ref()).unwrap();
    let plaintext = cipher.decrypt(&nonce, ciphertext.as_ref()).unwrap();
    assert_eq!(&plaintext, b"plaintext message");

}