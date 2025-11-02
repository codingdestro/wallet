use aes_gcm::{
    Aes256Gcm, Nonce, aead::{Aead, KeyInit, OsRng, rand_core::RngCore}
};
use pbkdf2::pbkdf2_hmac;
// use rand::RngCore;
use sha2::Sha256;
use std::{fs, io, path::Path};

const PBKDF2_ITERATIONS: u32 = 100_000;
const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 12;
const KEY_LEN: usize = 32;

/// Derives a 256-bit key from a password and salt using PBKDF2 (HMAC-SHA256).
fn derive_key(password: &str, salt: &[u8]) -> [u8; KEY_LEN] {
    let mut key = [0u8; KEY_LEN];
    pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, PBKDF2_ITERATIONS, &mut key);
    key
}

/// Encrypts the given input file using AES-256-GCM.
/// Output file contains [salt | nonce | ciphertext].
pub fn encrypt_file<P: AsRef<Path>>(input: P, password: &str, output: P) -> io::Result<()> {
    let plaintext = fs::read(&input)?;

    let mut salt = [0u8; SALT_LEN];
    OsRng.fill_bytes(&mut salt);

    let key = derive_key(password, &salt);
    let cipher = Aes256Gcm::new_from_slice(&key).unwrap();

    let mut nonce_bytes = [0u8; NONCE_LEN];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .expect("encryption failed");

    let mut encrypted_data = Vec::new();
    encrypted_data.extend_from_slice(&salt);
    encrypted_data.extend_from_slice(&nonce_bytes);
    encrypted_data.extend_from_slice(&ciphertext);

    fs::write(&output, encrypted_data)?;
    Ok(())
}

/// Decrypts the given file using the provided password.
pub fn decrypt_file<P: AsRef<Path>>(input: P, password: &str, output: P) -> io::Result<()> {
    let data = fs::read(&input)?;

    if data.len() < SALT_LEN + NONCE_LEN {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "File too short or corrupted",
        ));
    }

    let salt = &data[..SALT_LEN];
    let nonce_bytes = &data[SALT_LEN..SALT_LEN + NONCE_LEN];
    let ciphertext = &data[SALT_LEN + NONCE_LEN..];

    let key = derive_key(password, salt);
    let cipher = Aes256Gcm::new_from_slice(&key).unwrap();
    let nonce = Nonce::from_slice(nonce_bytes);

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "Invalid password or data"))?;

    fs::write(&output, plaintext)?;
    Ok(())
}
