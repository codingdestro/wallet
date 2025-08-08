use std::{collections::HashMap, env};

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
}

impl Wallet {
    pub fn new() -> Self {
        let list = HashMap::new();
        Wallet { list }
    }
    pub fn add(&mut self, key: String, value: String) {
        self.list.insert(key, value);
    }
    pub fn remove(&mut self, key: &str) {
        self.list.remove(key);
    }
    pub fn get(&self, key: &str) -> Option<&String> {
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

    pub fn find(&self, key: &str) -> Vec<&String> {
        self.list
            .iter()
            .filter(|(k, _)| k.contains(key))
            .map(|(_, v)| v)
            .collect()
    }
}
