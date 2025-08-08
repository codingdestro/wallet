use wallet::*;
fn main() {
    let cmd = Command::new();
    let mut wallet = Wallet::new();

    if cmd.args[1] == "-a" && cmd.args.len() >= 4 {
        let key = cmd.args[2].clone();
        let value = cmd.args[3].clone();
        wallet.add(key, value);
        println!("adding new {}:{}", cmd.args[2].clone(), cmd.args[3].clone());
    }

    wallet.print();
}
