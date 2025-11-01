use wallet::*;

fn main() {
    let cmd = Command::new();
    let usage = Usage::default();
    let mut wallet = Wallet::new();
    wallet.load();

    if cmd.args.len() <= 1 {
        usage.show();
        return;
    }

    let command = &cmd.args[1];
    
    match command.as_str() {
        "-a" | "--add" => {
            if cmd.args.len() >= 4 {
                let key = cmd.args[2].clone();
                let value = cmd.args[3].clone();
                wallet.add(key.clone(), value.clone());
                println!("✓ Added '{}' to wallet", key);
            } else {
                eprintln!("Error: Missing arguments for add command");
                eprintln!("Usage: wallet -a <KEY> <VALUE>");
                std::process::exit(1);
            }
        },
        "-l" | "--list" => {
            let keys = wallet.get_keys();
            if keys.is_empty() {
                println!("Wallet is empty. Use 'wallet -a <key> <value>' to add entries.");
            } else {
                println!("Available keys:");
                for key in keys {
                    println!("  • {}", key);
                }
            }
        },
        "-r" | "--remove" => {
            if cmd.args.len() >= 3 {
                let key = cmd.args[2].clone().trim().to_string();
                if wallet.key_exists(&key) {
                    wallet.del(&key);
                    println!("✓ Removed '{}' from wallet", key);
                } else {
                    eprintln!("Error: Key '{}' not found in wallet", key);
                    std::process::exit(1);
                }
            } else {
                eprintln!("Error: Missing key for remove command");
                eprintln!("Usage: wallet -r <KEY>");
                std::process::exit(1);
            }
        },
        "-s" | "--show" => {
            if cmd.args.len() >= 3 {
                let key = cmd.args[2].clone();
                match wallet.get(&key) {
                    Some(value) => {
                        println!("{}:", key);
                        println!("{}", value);
                    },
                    None => {
                        eprintln!("Error: Key '{}' not found in wallet", key);
                        std::process::exit(1);
                    }
                }
            } else {
                eprintln!("Error: Missing key for show command");
                eprintln!("Usage: wallet -s <KEY>");
                std::process::exit(1);
            }
        },
        "-c" | "--copy" => {
            if cmd.args.len() >= 3 {
                let key = cmd.args[2].clone();
                match wallet.copy(&key) {
                    Ok(()) => println!("✓ Copied '{}' to clipboard", key),
                    Err(()) => {
                        eprintln!("Error: Key '{}' not found in wallet", key);
                        std::process::exit(1);
                    }
                }
            } else {
                eprintln!("Error: Missing key for copy command");
                eprintln!("Usage: wallet -c <KEY>");
                std::process::exit(1);
            }
        },
        "-h" | "--help" => {
            usage.show();
        },
        _ => {
            eprintln!("Error: Unknown command '{}'", command);
            eprintln!("Use 'wallet --help' to see available commands");
            std::process::exit(1);
        }
    }

    wallet.save();
}
