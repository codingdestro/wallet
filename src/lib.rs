use std::collections::HashMap;
use std::io::Write;

pub mod cli;
pub mod clipboard;
pub mod crypto;

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
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() == 2 {
                    self.list.insert(parts[0].to_string(), parts[1].to_string());
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
        self.list.remove(key).expect("failed to  delete value");
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
        self.list.keys().cloned().collect()
    }

    pub fn key_exists(&self, key: &str) -> bool {
        self.list.contains_key(key)
    }
}
