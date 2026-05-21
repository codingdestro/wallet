use crate::*;
use clap::{Parser, Subcommand};

/// A secure command-line key-value store and password manager
#[derive(Parser, Debug)]
#[command(name = "wallet", version, about)]
pub struct Cli {
    /// Skip password prompt — read password from WALLET_PASSWORD env var
    #[arg(long, env = "WALLET_PASSWORD", hide_env_values = true)]
    pub password: Option<String>,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Add or update a key-value pair
    #[command(visible_alias = "a")]
    Add {
        /// The key to store
        key: String,
        /// The value to store
        value: String,
    },

    /// Show the value for a key
    #[command(visible_alias = "s")]
    Show {
        /// The key to display
        key: String,
    },

    /// Copy a value to the clipboard
    #[command(visible_alias = "c")]
    Copy {
        /// The key whose value to copy
        key: String,
    },

    /// List all stored keys
    #[command(visible_alias = "l")]
    List,

    /// Remove a key-value pair
    #[command(visible_alias = "r")]
    Remove {
        /// The key to remove
        key: String,
    },

    /// Generate a random password
    Generate {
        /// Length of the generated password (default: 24)
        #[arg(short, long, default_value_t = 24)]
        length: usize,

        /// Include special characters
        #[arg(short, long, default_value_t = false)]
        special: bool,
    },
}

/// Prompt the user for a password and return the trimmed result
pub fn prompt_password(prompt: &str) -> String {
    use std::io::Write;
    let mut password = String::new();
    print!("{prompt}");
    std::io::stdout().flush().expect("Failed to flush stdout");
    std::io::stdin()
        .read_line(&mut password)
        .expect("Failed to read password");
    password.trim().to_string()
}

/// Run the CLI by parsing args and dispatching to the wallet
pub fn run() {
    let cli = Cli::parse();

    // Handle no subcommand (just --help or --version or --password)
    let cmd = match &cli.command {
        Some(c) => c,
        None => {
            // If only --password was given, show help
            return;
        }
    };

    // Commands that don't need the wallet
    match cmd {
        Command::Generate { length, special } => {
            let value = generate_password(*length, *special);
            println!("{value}");
            return;
        }
        _ => {}
    }

    // All remaining commands need the wallet
    let mut wallet = Wallet::new();

    // Get the password
    let password = match &cli.password {
        Some(pw) => pw.clone(),
        None => prompt_password("Enter wallet password: "),
    };

    if password.is_empty() {
        eprintln!("Error: Password cannot be empty");
        std::process::exit(1);
    }

    wallet.load(&password);

    match cmd {
        Command::Add { key, value } => {
            wallet.add(key.clone(), value.clone());
            wallet.save(&password);
            println!("✓ Added '{key}' to wallet");
        }
        Command::Show { key } => match wallet.get(key) {
            Some(value) => {
                println!("{key}:");
                println!("{value}");
            }
            None => {
                eprintln!("Error: Key '{key}' not found in wallet");
                std::process::exit(1);
            }
        },
        Command::Copy { key } => match wallet.copy(key) {
            Ok(()) => println!("✓ Copied '{key}' to clipboard"),
            Err(()) => {
                eprintln!("Error: Key '{key}' not found in wallet");
                std::process::exit(1);
            }
        },
        Command::List => {
            let keys = wallet.get_keys();
            if keys.is_empty() {
                println!("Wallet is empty. Use 'wallet add <key> <value>' to add entries.");
            } else {
                println!("Available keys:");
                for key in keys {
                    println!("  • {key}");
                }
            }
        }
        Command::Remove { key } => {
            if wallet.key_exists(key) {
                wallet.del(key);
                wallet.save(&password);
                println!("✓ Removed '{key}' from wallet");
            } else {
                eprintln!("Error: Key '{key}' not found in wallet");
                std::process::exit(1);
            }
        }
        // Generate is handled above before wallet load; unreachable here
        Command::Generate { .. } => unreachable!(),
    }
}

/// Generate a random password
fn generate_password(length: usize, special: bool) -> String {
    use rand::Rng;

    let chars = if special {
        "ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnpqrstuvwxyz23456789!@#$%&*"
    } else {
        "ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnpqrstuvwxyz23456789"
    };

    let mut rng = rand::rng();
    (0..length).map(|_| {
        let idx = rng.random_range(0..chars.len());
        chars.as_bytes()[idx] as char
    }).collect()
}
