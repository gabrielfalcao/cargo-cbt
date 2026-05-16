use cargo_cbt::go;
use cargo_cbt::Cli;
use clap::Parser;

fn main() {
    let cli = Cli::parse_from(std::env::args().map(|a| a.to_string()));
    match go(&cli) {
        Ok(()) => {}
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1)
        }
    }
}
