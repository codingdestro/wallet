use std::{
    collections::HashMap,
    env,
    io::{Read, Write},
};

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
        Wallet {
            list,
            file_path: String::from("wallet.txt"),
        }
    }

    pub fn load(&mut self) {
        let mut buf: Vec<u8> = Vec::new();
        if let Ok(mut file) = std::fs::File::options()
            .read(true)
            .open(self.file_path.to_string())
        {
            file.read_to_end(&mut buf).expect("Failed to read file");
        }
        let buf = String::from_utf8(buf).unwrap();
        if buf.is_empty() {
            println!("file is empty");
        } else {
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

    pub fn clear(&mut self) {
        self.list.clear();
    }

    pub fn print(&self) {
        for (key, value) in &self.list {
            println!("{}: {}", key, value);
        }
    }
}
