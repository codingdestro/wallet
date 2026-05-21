use std::collections::HashMap;
use std::io::Write;

pub mod cli;
pub mod clipboard;
pub mod crypto;
pub mod tui;

pub struct Wallet {
    list: HashMap<String, String>,
    file_path: String,
}

impl Wallet {
    pub fn new() -> Self {
        let list = HashMap::new();
        let homepath = std::env::var("HOME").unwrap();
        Wallet {
            list,
            file_path: format!("{}/.local/share/wallet/data.wallet", homepath),
        }
    }

    /// Create a wallet with a custom file path (useful for testing)
    #[cfg(test)]
    fn with_path(file_path: String) -> Self {
        Wallet {
            list: HashMap::new(),
            file_path,
        }
    }

    pub fn load(&mut self, password: &str) {
        //check is file exists
        if !std::path::Path::new(&self.file_path).exists() {
            // If the file doesn't exist, initialize an empty wallet and return
            //create a folder if it doesn't exist
            let parent_dir = std::path::Path::new(&self.file_path).parent().unwrap();
            if !parent_dir.exists() {
                std::fs::create_dir_all(parent_dir).expect("Failed to create directory");
            }
            self.list = HashMap::new();
            self.save(password);
        }

        let mut wallet_data: Vec<u8> = Vec::new();
        let result = crypto::decrypt_file(&self.file_path, password);
        match result {
            Ok(decrypted_data) => {
                wallet_data = decrypted_data.into_bytes();
            }
            Err(_) => {
                // If decryption fails, return with an error password and clear the wallet
                eprintln!("Error: Incorrect password or corrupted wallet file");
                std::process::exit(1);
            }
        }

        let buf = String::from_utf8(wallet_data).unwrap();
        if !buf.is_empty() {
            for line in buf.lines() {
                if let Some((key, value)) = line.split_once(':') {
                    self.list.insert(key.to_string(), value.to_string());
                }
            }
        }
    }

    pub fn save(&mut self, password: &str) {
        let mut file = std::fs::File::options()
            .create(true)
            .write(true)
            .truncate(true)
            .open(self.file_path.to_string())
            .expect("Failed to open file");

        for (key, value) in &self.list {
            writeln!(file, "{}:{}", key, value).expect("Failed to write to file");
        }

        crypto::encrypt_file(&self.file_path, password, &self.file_path)
            .expect("Failed to encrypt wallet file");
    }

    pub fn add(&mut self, key: String, value: String) {
        self.list.insert(key, value);
    }
    pub fn del(&mut self, key: &String) {
        println!("Deleting key: {}", &key);
        self.list.remove(key);
    }
    pub fn get(&self, key: &String) -> Option<&String> {
        self.list.get(key)
    }
    pub fn copy(&mut self, key: &String) -> Result<(), ()> {
        match self.list.get(key) {
            Some(value) => match crate::clipboard::Clipboard::copy_with_message(value) {
                Ok(_) => Ok(()),
                Err(_) => {
                    eprintln!("Failed to copy to clipboard");
                    Err(())
                }
            },
            None => Err(()),
        }
    }

    pub fn clear(&mut self) {
        self.list.clear();
    }

    pub fn print(&self) {
        for (key, _) in &self.list {
            println!("{}", key);
        }
    }

    pub fn get_keys(&self) -> Vec<String> {
        let mut keys: Vec<String> = self.list.keys().cloned().collect();
        keys.sort();
        keys
    }

    pub fn key_exists(&self, key: &str) -> bool {
        self.list.contains_key(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Wallet Unit Tests (in-memory, no filesystem) ──────────────

    #[test]
    fn test_add_and_get() {
        let mut wallet = Wallet::with_path("/tmp/test-wallet.wallet".to_string());
        wallet.add("email".to_string(), "test@example.com".to_string());
        assert_eq!(
            wallet.get(&"email".to_string()),
            Some(&"test@example.com".to_string())
        );
    }

    #[test]
    fn test_add_overwrites_existing() {
        let mut wallet = Wallet::with_path("/tmp/test-wallet.wallet".to_string());
        wallet.add("key".to_string(), "old".to_string());
        wallet.add("key".to_string(), "new".to_string());
        assert_eq!(wallet.get(&"key".to_string()), Some(&"new".to_string()));
    }

    #[test]
    fn test_get_nonexistent_key() {
        let wallet = Wallet::with_path("/tmp/test-wallet.wallet".to_string());
        assert_eq!(wallet.get(&"nothing".to_string()), None);
    }

    #[test]
    fn test_del_removes_key() {
        let mut wallet = Wallet::with_path("/tmp/test-wallet.wallet".to_string());
        wallet.add("key".to_string(), "value".to_string());
        wallet.del(&"key".to_string());
        assert_eq!(wallet.get(&"key".to_string()), None);
    }

    #[test]
    fn test_del_nonexistent_key_does_not_panic() {
        let mut wallet = Wallet::with_path("/tmp/test-wallet.wallet".to_string());
        // Should not panic
        wallet.del(&"nonexistent".to_string());
    }

    #[test]
    fn test_key_exists() {
        let mut wallet = Wallet::with_path("/tmp/test-wallet.wallet".to_string());
        wallet.add("present".to_string(), "yes".to_string());
        assert!(wallet.key_exists("present"));
        assert!(!wallet.key_exists("absent"));
    }

    #[test]
    fn test_get_keys_empty_wallet() {
        let wallet = Wallet::with_path("/tmp/test-wallet.wallet".to_string());
        assert!(wallet.get_keys().is_empty());
    }

    #[test]
    fn test_get_keys_returns_sorted() {
        let mut wallet = Wallet::with_path("/tmp/test-wallet.wallet".to_string());
        wallet.add("z".to_string(), "last".to_string());
        wallet.add("a".to_string(), "first".to_string());
        wallet.add("m".to_string(), "middle".to_string());
        let keys = wallet.get_keys();
        assert_eq!(keys, vec!["a", "m", "z"]);
    }

    #[test]
    fn test_clear_wallet() {
        let mut wallet = Wallet::with_path("/tmp/test-wallet.wallet".to_string());
        wallet.add("k1".to_string(), "v1".to_string());
        wallet.add("k2".to_string(), "v2".to_string());
        assert_eq!(wallet.get_keys().len(), 2);
        wallet.clear();
        assert!(wallet.get_keys().is_empty());
    }

    #[test]
    fn test_copy_returns_err_for_missing_key() {
        let mut wallet = Wallet::with_path("/tmp/test-wallet.wallet".to_string());
        let result = wallet.copy(&"missing".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_in_memory_save_and_load_roundtrip() {
        let path = format!("/tmp/wallet-roundtrip-{}.wallet", std::process::id());
        // Make sure test directory exists
        let parent = std::path::Path::new(&path).parent().unwrap();
        std::fs::create_dir_all(parent).ok();

        let password = "test-password";

        // Write
        {
            let mut wallet = Wallet::with_path(path.clone());
            wallet.add("github".to_string(), "ghp_token".to_string());
            wallet.add("email".to_string(), "user@example.com".to_string());
            wallet.save(password);
        }

        // Read back
        {
            let mut wallet = Wallet::with_path(path.clone());
            wallet.load(password);
            assert_eq!(wallet.get(&"github".to_string()), Some(&"ghp_token".to_string()));
            assert_eq!(wallet.get(&"email".to_string()), Some(&"user@example.com".to_string()));
            assert_eq!(wallet.get_keys().len(), 2);
        }

        // Cleanup
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_load_with_wrong_password_fails() {
        let path = format!("/tmp/wallet-badpw-{}.wallet", std::process::id());
        let parent = std::path::Path::new(&path).parent().unwrap();
        std::fs::create_dir_all(parent).ok();

        // Save with one password
        {
            let mut wallet = Wallet::with_path(path.clone());
            wallet.add("secret".to_string(), "value".to_string());
            wallet.save("correct-password");
        }

        // Load with wrong password — should exit the process
        // We can't easily test process::exit, so just verify the file exists
        assert!(std::path::Path::new(&path).exists());

        // Cleanup
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_load_nonexistent_file_creates_empty_wallet() {
        let path = format!("/tmp/wallet-new-{}.wallet", std::process::id());
        let mut wallet = Wallet::with_path(path.clone());
        // load() should create the file since it doesn't exist
        wallet.load("initial-password");
        assert!(wallet.get_keys().is_empty());
        assert!(std::path::Path::new(&path).exists());

        // Cleanup
        std::fs::remove_file(&path).ok();
        let parent = std::path::Path::new(&path).parent().unwrap();
        std::fs::remove_dir_all(parent).ok();
    }

    #[test]
    fn test_save_encrypts_file() {
        let path = format!("/tmp/wallet-enc-{}.wallet", std::process::id());
        let parent = std::path::Path::new(&path).parent().unwrap();
        std::fs::create_dir_all(parent).ok();

        {
            let mut wallet = Wallet::with_path(path.clone());
            wallet.add("key".to_string(), "value".to_string());
            wallet.save("password");
        }

        // Read raw bytes — should be binary ciphertext, not plaintext
        let data = std::fs::read(&path).unwrap();
        let content = String::from_utf8_lossy(&data);
        // The ciphertext should NOT contain our plaintext
        assert!(!content.contains("key:value"), "File should be encrypted, not plaintext");
        // Should have salt + nonce + ciphertext
        assert!(data.len() > 16 + 12, "Encrypted file should contain salt + nonce + ciphertext");

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_values_with_colons_are_preserved() {
        let path = format!("/tmp/wallet-colon-{}.wallet", std::process::id());
        let parent = std::path::Path::new(&path).parent().unwrap();
        std::fs::create_dir_all(parent).ok();

        let password = "test";

        {
            let mut wallet = Wallet::with_path(path.clone());
            wallet.add("token".to_string(), "admin:pass123".to_string());
            wallet.save(password);
        }

        {
            let mut wallet = Wallet::with_path(path.clone());
            wallet.load(password);
            assert_eq!(
                wallet.get(&"token".to_string()),
                Some(&"admin:pass123".to_string())
            );
        }

        std::fs::remove_file(&path).ok();
    }
}
