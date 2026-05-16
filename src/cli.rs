use crate::{shell_command, Manifest, Result};
use clap::Parser;
use iocore::Path;
use std::ops::Deref;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = "cargo cbt")]
pub struct Cli {
    #[arg()]
    path: Option<Path>,

    #[arg(short, long, conflicts_with = "quiet")]
    verbose: bool,

    #[arg(short, long, help = "build docs with `cargo docs'")]
    docs: bool,

    #[arg(
        short = 'O',
        long,
        requires = "docs",
        help = "runs `cargo docs --open'"
    )]
    open_docs: bool,

    #[arg(short = 'r', long, help = "execute with `cargo run' if suitable")]
    run: bool,

    #[arg(short, long)]
    purge: bool,

    #[arg(short, long, help = "wipe target/ folder after builds")]
    wipe: bool,

    #[arg(short, long)]
    ignore_errors: bool,

    #[arg(short, long, conflicts_with = "ignore_errors")]
    fail_fast: bool,

    #[arg(short = 'c', long, help = "capture test output")]
    test_capture: bool,

    #[arg(short, long, help = "do not clear console before running")]
    no_clear_console: bool,

    #[arg(long)]
    test: Option<String>,

    #[arg(flatten, last = true)]
    cargo_subcommands: CargoSubcommandOpt,
}
impl Cli {
    pub fn verbose_errors(&self) -> bool {
        !self.quiet
    }
    pub fn verbose(&self) -> bool {
        !self.quiet && self.verbose
    }
    pub fn manifest(&self) -> Result<Manifest> {
        Ok(Manifest::default()?)
    }
    pub fn opts(&self, subcommand_opts: &CargoSubcommandOpt) -> String {
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
    pub fn run_opts(&self, manifest: &Manifest) -> String {
        let opts = self.opts();
        if let Some(bin_names) = manifest.bin_names() {
            if let Some(name) = bin_names.first() {
                return format!("{opts} --bin {name}");
            }
        }
        if let Some(example_names) = manifest.example_names() {
            if let Some(name) = example_names.first() {
                return format!("{opts} --example {name}");
            }
        }
        opts
    }
    pub fn run_command_can_run(&self, manifest: &Manifest) -> bool {
        if !self.run {
            false
        } else if let Some(bin_names) = manifest.bin_names() {
            true
        } else if let Some(example_names) = manifest.example_names() {
            if let Some(name) = example_names.first() {
                true
            } else {
                false
            }
        } else {
            false
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

    pub fn run_command(&self, manifest: &Manifest) -> String {
        format!("cargo run {}", self.run_opts(manifest))
            .trim()
            .to_string()
    }
    pub fn post_run(&self) -> Result<()> {
        if self.wipe {
            let build_path = Path::new("target");
            match build_path.canonicalize() {
                Ok(target) => {
                    let path = target.abbreviate();
                    if target.is_dir() {
                        if self.verbose() {
                            eprintln!("deleting build directory: {path}")
                        }
                        target
                            .delete()
                            .map_err(|error| Error::PostRunError(format!("failed to delete")))?;
                    }
                }
                Err(error) => {
                    let target = build_path.try_canonicalize().to_string();
                    eprintln!("WARNING: post-run cannot delete {target:#?}: {error}");
                }
            }
        }
        Ok(())
    }
}
impl Deref for Cli {
    type Target = CargoSubcommandOpt;

    fn deref(&self) -> &Self::Target {
        &self.cargo_subcommands
    }
}

pub fn go(cli: &Cli) -> Result<()> {
    let manifest = cli.manifest()?;

    if cli.purge {
        let target = Path::new("target");
        if target.is_dir() {
            target.delete()?;
        }
    }
    if !cli.no_clear_console {
        print!("\r\x1b[2J\x1b[3J\x1b[H");
    }

    let mut commands = if let Some(_) = &cli.test {
        vec![cli.test_command()]
    } else {
        vec![cli.check_command(), cli.build_command(), cli.test_command()]
    };

    if cli.docs {
        commands.push(cli.docs_command());
    }
    if cli.run_command_can_run(&manifest) {
        commands.push(cli.run_command(&manifest));
    }

    let cwd = Path::cwd();

    for command in commands.into_iter() {
        match shell_command(&command, &cwd) {
            Ok(_) => {
                //
                if self.verbose() {
                    eprintln!("{command}: OK");
                }
            }
            Err(error) => {
                //
                if self.verbose_errors() {
                    eprintln!("{command}: ERROR");
                }
                if self.fail_fast() {
                    std::process::exit(1);
                }
            }
        }
    }
    if let Err(error) = cli.post_run() {
        if self.verbose_errors() {
            eprintln!("cargo-cbt post-run error: {error}");
        }
    }
    Ok(())
}

#[derive(Parser, Debug, Clone, Copy)]
pub struct CargoSubcommandOpt {
    #[arg(short = 'q', long)]
    quiet: bool,

    #[arg(short = 'R', long)]
    release: bool,

    #[arg(short = 'd', long)]
    debug: bool,

    #[arg(short = 'A', long)]
    all_targets: bool,

    #[arg(short = 'a', long)]
    all_features: bool,

    #[arg(long = 'o', help = "runs cargo subcommands in `--offline' mode")]
    offline: bool,

    #[arg(last = true, allow_hyphen_values = true)]
    opts: Vec<String>,
}

impl CargoSubcommandOpt {
    pub fn quiet(&self) -> bool {
        self.quiet
    }
    pub fn release(&self) -> bool {
        self.release
    }
    pub fn debug(&self) -> bool {
        self.debug
    }
    pub fn offline(&self) -> bool {
        self.offline
    }
    pub fn all_targets(&self) -> bool {
        self.all_targets
    }
    pub fn all_features(&self) -> bool {
        self.all_features
    }

    pub fn rustc_and_cargo_opts(&self) -> String {
        let mut opts_list = Vec::<String>::new();
        if env_vars_color_heuristic() {
            opts_list.push("--color always".to_string());
        }
        if self.offline {
            opts_list.push("--offline".to_string());
        }
        opts_list.join(" ")
    }
    pub fn without_release(&self) -> CargoSubcommandOpt {
        let mut opts = self.clone();
        opts.release = false;
        opts
    }
    pub fn with_release(&self) -> CargoSubcommandOpt {
        let mut opts = self.clone();
        opts.release = false;
        opts
    }
}

pub fn env_vars_color_heuristic() -> bool {
    iocore::env::var("COLORTERM")
        .unwrap_or_default()
        .trim()
        .to_lowercase()
        == "truecolor"
        || iocore::env::var("TERM")
            .unwrap_or_default()
            .trim()
            .starts_with("xterm")
}
