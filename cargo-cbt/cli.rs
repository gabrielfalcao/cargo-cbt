use crate::{shell_command, Result};
use clap::Parser;
use iocore::Path;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = "cargo_cbt command-line utility")]
pub struct Cli {
    #[arg()]
    path: Option<Path>,

    #[arg(short, long)]
    quiet: bool,

    #[arg(short, long)]
    purge: bool,

    #[arg(short = 'c', long, help = "capture test output")]
    test_capture: bool,

    #[arg()]
    opts: Vec<String>,
}
impl Cli {
    pub fn rustc_and_cargo_opts(&self) -> String {
        format!("--color always",).trim().to_string()
    }
    pub fn opts(&self) -> String {
        format!(
            " {} {}",
            self.rustc_and_cargo_opts(),
            if self.opts.is_empty() {
                String::new()
            } else {
                self.opts.join(" ")
            }
        )
        .trim()
        .to_string()
    }
    pub fn check_opts(&self) -> String {
        format!("{}", self.opts())
    }
    pub fn build_opts(&self) -> String {
        format!("{}", self.opts())
    }
    pub fn test_opts(&self) -> String {
        format!(
            "{}",
            if self.test_capture {
                self.opts.join(" ")
            } else {
                format!(
                    " -j 1{} -- --nocapture {}",
                    self.opts.join(" "),
                    self.rustc_and_cargo_opts()
                )
            }
        )
    }
    pub fn check_command(&self) -> String {
        format!("cargo check {}", self.check_opts())
            .trim()
            .to_string()
    }
    pub fn build_command(&self) -> String {
        format!("cargo build {}", self.build_opts())
            .trim()
            .to_string()
    }
    pub fn test_command(&self) -> String {
        format!("cargo test {}", self.test_opts())
            .trim()
            .to_string()
    }
}

pub fn go(cli: &Cli) -> Result<()> {
    if cli.purge {
        let target = Path::new("target");
        if target.is_dir() {
            target.delete()?;
        }
    }
    shell_command(format!("tput clear"), Path::cwd())?;
    shell_command(cli.check_command(), Path::cwd())?;
    shell_command(cli.build_command(), Path::cwd())?;
    shell_command(cli.test_command(), Path::cwd())?;
    Ok(())
}
