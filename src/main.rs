use wallet::cli;
use wallet::tui;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // TUI mode: no args, or explicit --tui / -t
    if args.len() <= 1 || args[1] == "--tui" || args[1] == "-t" {
        if let Err(e) = tui::run() {
            eprintln!("TUI error: {e}");
            std::process::exit(1);
        }
        return;
    }

    // CLI mode: any subcommand or flag
    cli::run();
}
