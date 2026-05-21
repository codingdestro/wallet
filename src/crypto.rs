use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit, OsRng, rand_core::RngCore},
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
pub fn decrypt_file<P: AsRef<Path>>(input: P, password: &str) -> io::Result<String> {
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

    String::from_utf8(plaintext).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Decrypted data is not valid UTF-8: {e}"),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let path = format!("/tmp/crypto-roundtrip-{}.enc", std::process::id());
        let password = "test-password";
        let original = "Hello, this is a secret message!";

        // Write plaintext to input file
        {
            let mut f = std::fs::File::create(&path).unwrap();
            f.write_all(original.as_bytes()).unwrap();
        }

        // Encrypt
        encrypt_file(&path, password, &path).unwrap();

        // The file should now be encrypted
        let data = std::fs::read(&path).unwrap();
        assert_ne!(data, original.as_bytes(), "File should be encrypted");

        // Decrypt
        let decrypted = decrypt_file(&path, password).unwrap();
        assert_eq!(decrypted, original);

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_decrypt_with_wrong_password_fails() {
        let path = format!("/tmp/crypto-badpw-{}.enc", std::process::id());
        let password = "correct-password";
        let original = "sensitive data";

        {
            let mut f = std::fs::File::create(&path).unwrap();
            f.write_all(original.as_bytes()).unwrap();
        }

        encrypt_file(&path, password, &path).unwrap();

        let result = decrypt_file(&path, "wrong-password");
        assert!(result.is_err(), "Decrypting with wrong password should fail");

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_decrypt_corrupted_file_fails() {
        let path = format!("/tmp/crypto-corrupt-{}.enc", std::process::id());

        // Write garbage data
        {
            let mut f = std::fs::File::create(&path).unwrap();
            f.write_all(b"this is not encrypted data").unwrap();
        }

        let result = decrypt_file(&path, "any-password");
        assert!(result.is_err(), "Decrypting corrupted data should fail");

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_decrypt_too_short_file_fails() {
        let path = format!("/tmp/crypto-short-{}.enc", std::process::id());

        // Write too-short data (less than salt + nonce)
        {
            let mut f = std::fs::File::create(&path).unwrap();
            f.write_all(b"short").unwrap();
        }

        let result = decrypt_file(&path, "password");
        assert!(result.is_err(), "Decrypting a too-short file should fail");

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_encrypt_produces_unique_output() {
        let path = format!("/tmp/crypto-unique-{}.enc", std::process::id());
        let password = "password";
        let original = "same content";

        let mut results = Vec::new();

        for _ in 0..3 {
            {
                let mut f = std::fs::File::create(&path).unwrap();
                f.write_all(original.as_bytes()).unwrap();
            }
            encrypt_file(&path, password, &path).unwrap();
            let data = std::fs::read(&path).unwrap();
            results.push(data);
        }

        // Each encryption should produce different output (different salt + nonce)
        assert!(results[0] != results[1] || results[1] != results[2],
                "Each encryption should produce unique output");

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_decrypt_utf8_content() {
        let path = format!("/tmp/crypto-utf8-{}.enc", std::process::id());
        let password = "pässwörd";
        let original = "Hello 你好 ñoño émoji🔥";

        {
            let mut f = std::fs::File::create(&path).unwrap();
            f.write_all(original.as_bytes()).unwrap();
        }

        encrypt_file(&path, password, &path).unwrap();
        let decrypted = decrypt_file(&path, password).unwrap();
        assert_eq!(decrypted, original);

        std::fs::remove_file(&path).ok();
    }
}
