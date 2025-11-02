use std::{collections::HashMap, env, io::Write};

pub mod clipboard;
pub mod crypto;

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

    pub fn load(&mut self, password: &str) {
        //check is file exists
        if !std::path::Path::new(&self.file_path).exists() {
            // If the file doesn't exist, initialize an empty wallet and return
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

pub struct Usage {
    pub program_name: String,
    pub version: String,
    pub description: String,
    pub commands: Vec<CommandHelp>,
}

pub struct CommandHelp {
    pub flag: String,
    pub usage: String,
    pub description: String,
    pub examples: Vec<String>,
}

impl Usage {
    pub fn new() -> Self {
        Self {
            program_name: "wallet".to_string(),
            version: "1.0.0".to_string(),
            description: "A secure command-line key-value store and password manager".to_string(),
            commands: Vec::new(),
        }
    }

    pub fn default() -> Self {
        let mut usage = Self::new();
        usage.add_command(CommandHelp {
            flag: "-a, --add".to_string(),
            usage: "<KEY> <VALUE>".to_string(),
            description: "Add or update a key-value pair in the wallet".to_string(),
            examples: vec![
                "wallet -a email john@example.com".to_string(),
                "wallet --add github-token ghp_xxxxxxxxxxxx".to_string(),
            ],
        });

        usage.add_command(CommandHelp {
            flag: "-s, --show".to_string(),
            usage: "<KEY>".to_string(),
            description: "Display the value for the specified key".to_string(),
            examples: vec![
                "wallet -s email".to_string(),
                "wallet --show github-token".to_string(),
            ],
        });

        usage.add_command(CommandHelp {
            flag: "-c, --copy".to_string(),
            usage: "<KEY>".to_string(),
            description: "Copy the value of the specified key to clipboard".to_string(),
            examples: vec![
                "wallet -c password".to_string(),
                "wallet --copy api-key".to_string(),
            ],
        });

        usage.add_command(CommandHelp {
            flag: "-l, --list".to_string(),
            usage: "".to_string(),
            description: "List all available keys (values are hidden for security)".to_string(),
            examples: vec!["wallet -l".to_string(), "wallet --list".to_string()],
        });

        usage.add_command(CommandHelp {
            flag: "-r, --remove".to_string(),
            usage: "<KEY>".to_string(),
            description: "Remove a key-value pair from the wallet".to_string(),
            examples: vec![
                "wallet -r old-password".to_string(),
                "wallet --remove expired-token".to_string(),
            ],
        });

        usage.add_command(CommandHelp {
            flag: "-h, --help".to_string(),
            usage: "".to_string(),
            description: "Show this help message".to_string(),
            examples: vec!["wallet -h".to_string(), "wallet --help".to_string()],
        });

        usage
    }

    pub fn add_command(&mut self, command: CommandHelp) {
        self.commands.push(command);
    }

    pub fn show(&self) {
        // Header
        println!("{} v{}", self.program_name, self.version);
        println!("{}", self.description);
        println!();

        // Usage
        println!("USAGE:");
        println!("    {} [OPTIONS] [ARGS...]", self.program_name);
        println!();

        // Options
        println!("OPTIONS:");
        for cmd in &self.commands {
            let flag_width = 20;
            let usage_part = if cmd.usage.is_empty() {
                String::new()
            } else {
                format!(" {}", cmd.usage)
            };

            println!(
                "    {:<width$} {}",
                format!("{}{}", cmd.flag, usage_part),
                cmd.description,
                width = flag_width + cmd.usage.len()
            );
        }
        println!();

        // Examples
        println!("EXAMPLES:");
        for cmd in &self.commands {
            if !cmd.examples.is_empty() {
                for example in &cmd.examples {
                    println!("    {}", example);
                }
            }
        }
        println!();

        // Footer
        println!("STORAGE:");
        println!("    Data is stored in: ~/wallet.txt");
        println!("    Each line follows the format: key:value");
        println!();

        println!("SECURITY NOTE:");
        println!("    This tool stores data in plain text. For sensitive data,");
        println!("    consider using additional encryption or a dedicated password manager.");
    }

    // Legacy method for backward compatibility
    pub fn from(messages: Vec<String>) -> Self {
        let mut usage = Self::new();
        for msg in messages {
            usage.commands.push(CommandHelp {
                flag: "".to_string(),
                usage: "".to_string(),
                description: msg,
                examples: vec![],
            });
        }
        usage
    }
}
