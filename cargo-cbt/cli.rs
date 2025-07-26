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

    #[arg(short, long, help = "build docs with `cargo docs'")]
    docs: bool,

    #[arg(short = 'O', long, requires = "docs", help = "runs `cargo docs --open'")]
    open_docs: bool,

    #[arg(short, long)]
    purge: bool,

    #[arg(short, long)]
    release: bool,

    #[arg(short = 'A', long)]
    all_targets: bool,

    #[arg(short, long)]
    all_features: bool,

    #[arg(short, long)]
    ignore_errors: bool,

    #[arg(short = 'c', long, help = "capture test output")]
    test_capture: bool,

    #[arg(short, long, help = "do not clear console before running")]
    no_clear_console: bool,

    #[arg(long)]
    test: Option<String>,

    #[arg()]
    opts: Vec<String>,
}
impl Cli {
    pub fn rustc_and_cargo_opts(&self) -> String {
        if iocore::env::var("COLORTERM")
            .unwrap_or_default()
            .trim()
            .to_lowercase()
            == "truecolor"
            || iocore::env::var("TERM")
                .unwrap_or_default()
                .trim()
                .starts_with("xterm")
        {
            format!("--color always")
        } else {
            String::new()
        }
    }
    pub fn opts(&self) -> String {
        [
            self.rustc_and_cargo_opts(),
            self.release
                .then_some("--release".to_string())
                .unwrap_or_default(),
            self.all_targets
                .then_some("--all-targets".to_string())
                .unwrap_or_default(),
            self.all_features
                .then_some("--all-features".to_string())
                .unwrap_or_default(),
            self.opts.join(" "),
        ]
        .join(" ")
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
                    " -j 1{}{} -- --nocapture {}",
                    self.opts.join(" "),
                    if let Some(test) = &self.test {
                        format!("--test {}", test)
                    } else {
                        String::new()
                    },
                    self.rustc_and_cargo_opts()
                )
            }
        )
    }
    pub fn docs_opts(&self) -> String {
        let opts = self.opts();
        if self.open_docs {
            format!("{opts} --open --no-deps")
        } else {
            format!("{opts} --no-deps")
        }
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
    pub fn docs_command(&self) -> String {
        format!("cargo docs {}", self.docs_opts())
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
    if !cli.no_clear_console {
        shell_command(format!("tput clear"), Path::cwd())?;
    }

    let mut commands = if let Some(_) = &cli.test {
        vec![cli.test_command()]
    } else {
        vec![cli.check_command(), cli.build_command(), cli.test_command()]
    };

    if cli.docs {
        commands.push(cli.docs_command());
    }

    let cwd = Path::cwd();
    if cli.ignore_errors {
        for command in commands.into_iter() {
            if shell_command(&command, &cwd).is_ok() {
                println!("{command}: OK");
            } else {
                eprintln!("{command}: ERROR");
            }
        }
    } else {
        for command in commands.into_iter() {
            shell_command(&command, &cwd)?;
        }
    }
    Ok(())
}
