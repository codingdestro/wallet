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

/// Prompt the user for a password (hidden input) and return the trimmed result
pub fn prompt_password(prompt: &str) -> String {
    rpassword::prompt_password(prompt)
        .unwrap_or_else(|_| {
            // Fallback to plain-text input if rpassword fails
            use std::io::Write;
            let mut password = String::new();
            print!("{prompt}");
            std::io::stdout().flush().expect("Failed to flush stdout");
            std::io::stdin()
                .read_line(&mut password)
                .expect("Failed to read password");
            password
        })
        .trim()
        .to_string()
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
pub(crate) fn generate_password(length: usize, special: bool) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_password_default_length() {
        let pw = generate_password(24, false);
        assert_eq!(pw.len(), 24);
    }

    #[test]
    fn test_generate_password_custom_length() {
        let pw = generate_password(8, false);
        assert_eq!(pw.len(), 8);
    }

    #[test]
    fn test_generate_password_very_long() {
        let pw = generate_password(128, false);
        assert_eq!(pw.len(), 128);
    }

    #[test]
    fn test_generate_password_with_special_chars() {
        let pw = generate_password(32, true);
        assert_eq!(pw.len(), 32);
        // Should contain at least some chars from the special set
        let special_chars: &[char] = &['!', '@', '#', '$', '%', '&', '*'];
        let has_special = pw.chars().any(|c| special_chars.contains(&c));
        assert!(has_special, "Password with special flag should contain special chars: {pw}");
    }

    #[test]
    fn test_generate_password_without_special_no_specials() {
        let pw = generate_password(100, false);
        let special_chars: &[char] = &['!', '@', '#', '$', '%', '&', '*'];
        let has_special = pw.chars().any(|c| special_chars.contains(&c));
        assert!(!has_special, "Password without special flag should not contain special chars: {pw}");
    }

    #[test]
    fn test_generate_password_no_ambiguous_chars() {
        // The charset excludes 0, O, I, l, 1 to avoid ambiguity
        let pw = generate_password(100, false);
        let ambiguous: &[char] = &['0', 'O', 'I', 'l', '1'];
        let has_ambiguous = pw.chars().any(|c| ambiguous.contains(&c));
        assert!(!has_ambiguous, "Password should not contain ambiguous chars: {pw}");
    }

    #[test]
    fn test_generate_password_unique_each_call() {
        let pw1 = generate_password(24, true);
        let pw2 = generate_password(24, true);
        assert_ne!(pw1, pw2, "Two generated passwords should be different");
    }

    #[test]
    fn test_generate_password_length_zero() {
        let pw = generate_password(0, false);
        assert!(pw.is_empty());
    }

    #[test]
    fn test_clap_cli_parses_add() {
        let args = vec!["wallet", "add", "mykey", "myvalue"];
        let cli = Cli::try_parse_from(args).unwrap();
        let cmd = cli.command.unwrap();
        match cmd {
            Command::Add { key, value } => {
                assert_eq!(key, "mykey");
                assert_eq!(value, "myvalue");
            }
            _ => panic!("Expected Add command"),
        }
    }

    #[test]
    fn test_clap_cli_parses_add_with_alias() {
        let args = vec!["wallet", "a", "k", "v"];
        let cli = Cli::try_parse_from(args).unwrap();
        let cmd = cli.command.unwrap();
        match cmd {
            Command::Add { key, value } => {
                assert_eq!(key, "k");
                assert_eq!(value, "v");
            }
            _ => panic!("Expected Add command via alias"),
        }
    }

    #[test]
    fn test_clap_cli_parses_show() {
        let args = vec!["wallet", "show", "mykey"];
        let cli = Cli::try_parse_from(args).unwrap();
        let cmd = cli.command.unwrap();
        match cmd {
            Command::Show { key } => assert_eq!(key, "mykey"),
            _ => panic!("Expected Show command"),
        }
    }

    #[test]
    fn test_clap_cli_parses_show_with_alias() {
        let args = vec!["wallet", "s", "k"];
        let cli = Cli::try_parse_from(args).unwrap();
        let cmd = cli.command.unwrap();
        match cmd {
            Command::Show { key } => assert_eq!(key, "k"),
            _ => panic!("Expected Show command via alias"),
        }
    }

    #[test]
    fn test_clap_cli_parses_list() {
        let args = vec!["wallet", "list"];
        let cli = Cli::try_parse_from(args).unwrap();
        assert!(cli.command.is_some());
        match cli.command.unwrap() {
            Command::List => {} // expected
            _ => panic!("Expected List command"),
        }
    }

    #[test]
    fn test_clap_cli_parses_list_with_alias() {
        let args = vec!["wallet", "l"];
        let cli = Cli::try_parse_from(args).unwrap();
        match cli.command.unwrap() {
            Command::List => {} // expected
            _ => panic!("Expected List command via alias"),
        }
    }

    #[test]
    fn test_clap_cli_parses_remove() {
        let args = vec!["wallet", "remove", "oldkey"];
        let cli = Cli::try_parse_from(args).unwrap();
        match cli.command.unwrap() {
            Command::Remove { key } => assert_eq!(key, "oldkey"),
            _ => panic!("Expected Remove command"),
        }
    }

    #[test]
    fn test_clap_cli_parses_remove_with_alias() {
        let args = vec!["wallet", "r", "k"];
        let cli = Cli::try_parse_from(args).unwrap();
        match cli.command.unwrap() {
            Command::Remove { key } => assert_eq!(key, "k"),
            _ => panic!("Expected Remove command via alias"),
        }
    }

    #[test]
    fn test_clap_cli_parses_copy() {
        let args = vec!["wallet", "copy", "mykey"];
        let cli = Cli::try_parse_from(args).unwrap();
        match cli.command.unwrap() {
            Command::Copy { key } => assert_eq!(key, "mykey"),
            _ => panic!("Expected Copy command"),
        }
    }

    #[test]
    fn test_clap_cli_parses_copy_with_alias() {
        let args = vec!["wallet", "c", "k"];
        let cli = Cli::try_parse_from(args).unwrap();
        match cli.command.unwrap() {
            Command::Copy { key } => assert_eq!(key, "k"),
            _ => panic!("Expected Copy command via alias"),
        }
    }

    #[test]
    fn test_clap_cli_parses_generate_default() {
        let args = vec!["wallet", "generate"];
        let cli = Cli::try_parse_from(args).unwrap();
        match cli.command.unwrap() {
            Command::Generate { length, special } => {
                assert_eq!(length, 24);
                assert!(!special);
            }
            _ => panic!("Expected Generate command"),
        }
    }

    #[test]
    fn test_clap_cli_parses_generate_with_options() {
        let args = vec!["wallet", "generate", "-l", "16", "-s"];
        let cli = Cli::try_parse_from(args).unwrap();
        match cli.command.unwrap() {
            Command::Generate { length, special } => {
                assert_eq!(length, 16);
                assert!(special);
            }
            _ => panic!("Expected Generate command with options"),
        }
    }

    #[test]
    fn test_clap_cli_parses_password_flag() {
        let args = vec!["wallet", "--password", "mypass", "list"];
        let cli = Cli::try_parse_from(args).unwrap();
        assert_eq!(cli.password, Some("mypass".to_string()));
    }

    #[test]
    fn test_clap_cli_no_command_returns_none() {
        let args = vec!["wallet"];
        let cli = Cli::try_parse_from(args).unwrap();
        assert!(cli.command.is_none());
    }
}
