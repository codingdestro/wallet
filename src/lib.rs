use std::{
    collections::HashMap,
    env,
    io::{Read, Write},
};

pub mod clipboard;

pub struct Command {
    pub args: Vec<String>,
}

impl Command {
    pub fn new() -> Self {
        let args = env::args().collect();
        Command { args }
    }
}

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
            file_path: String::from(format!("{}/wallet.txt", homepath)),
        }
    }

    pub fn load(&mut self) {
        let mut buf: Vec<u8> = Vec::new();
        if let Ok(mut file) = std::fs::File::options().read(true).open(&self.file_path) {
            file.read_to_end(&mut buf).expect("Failed to read file");
        }
        let buf = String::from_utf8(buf).unwrap();
        if !buf.is_empty() {
            for line in buf.lines() {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() == 2 {
                    self.list.insert(parts[0].to_string(), parts[1].to_string());
                }
            }
        }
    }

    pub fn save(&mut self) {
        let mut file = std::fs::File::options()
            .create(true)
            .write(true)
            .truncate(true)
            .open(self.file_path.to_string())
            .expect("Failed to open file");
        for (key, value) in &self.list {
            writeln!(file, "{}:{}", key, value).expect("Failed to write to file");
        }
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
}

pub struct Usage {
    pub messages: Vec<String>,
}

impl Usage {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
        }
    }

    pub fn from(messages: Vec<String>) -> Self {
        Self { messages }
    }

    pub fn show(&self) {
        println!("Usage:");
        for message in self.messages.clone() {
            println!("{}", message);
        }
    }
}
