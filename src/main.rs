use wallet::*;
fn main() {
    let cmd = Command::new();
    let mut wallet = Wallet::new();
    wallet.load();

    if cmd.args[1] == "-a" && cmd.args.len() >= 4 {
        let key = cmd.args[2].clone();
        let value = cmd.args[3].clone();
        wallet.add(key, value);
        println!("adding new {}:{}", cmd.args[2].clone(), cmd.args[3].clone());
    } else if cmd.args[1] == "-l" {
        wallet.print();
    } else if cmd.args[1] == "-r" && cmd.args.len() >= 3 {
        let key = cmd.args[2].clone().trim().to_string();
        wallet.del(&key);
    } else if cmd.args[1] == "-s" && cmd.args.len() >= 3 {
        let key = cmd.args[2].clone();
        match wallet.get(&key) {
            Some(value) => println!("{}:\n{}", key, value),
            None => println!("not found {}", key),
        }
    }

    wallet.save();
}
