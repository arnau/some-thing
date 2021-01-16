use clap::IntoApp;
use clap::{AppSettings, Clap};
use clap_generate::{generate, generators::*};
use some::cli;
use std::io::{prelude::*, stdout};

const CLI_NAME: &str = "some";

#[derive(Clap, Copy, Clone, Debug)]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
    PowerShell,
    Elvish,
}

impl Shell {
    fn generate(&self, writer: &mut dyn Write) {
        let mut app = Cli::into_app();
        match self {
            Shell::Bash => generate::<Bash, _>(&mut app, CLI_NAME, writer),
            Shell::Zsh => generate::<Zsh, _>(&mut app, CLI_NAME, writer),
            Shell::Fish => generate::<Fish, _>(&mut app, CLI_NAME, writer),
            Shell::PowerShell => generate::<PowerShell, _>(&mut app, CLI_NAME, writer),
            Shell::Elvish => generate::<Elvish, _>(&mut app, CLI_NAME, writer),
        }
    }
}

/// Generates the completions script for the given shell.
#[derive(Debug, Clap)]
#[clap(name = CLI_NAME, version, global_setting(AppSettings::ColoredHelp))]
struct CompletionsCmd {
    #[clap(arg_enum, value_name = "SHELL")]
    shell: Shell,
}

impl CompletionsCmd {
    pub fn run(&self) {
        let mut writer = stdout();

        self.shell.generate(&mut writer);
    }
}

#[derive(Debug, Clap)]
enum Subcommand {
    #[clap(alias = "a")]
    Init(cli::init::Cmd),
    Add(cli::add::Cmd),
    Build(cli::build::Cmd),
    Destroy(cli::destroy::Cmd),
    Completions(CompletionsCmd),
}

#[derive(Debug, Clap)]
#[clap(name = CLI_NAME, version, global_setting(AppSettings::ColoredHelp))]
struct Cli {
    #[clap(subcommand)]
    subcommand: Subcommand,
}

fn main() {
    let cli: Cli = Cli::parse();

    match cli.subcommand {
        Subcommand::Init(cmd) => match cmd.run() {
            Ok(msg) => {
                println!("{}", msg);
            }
            Err(err) => {
                eprintln!("{:?}", err);
            }
        },
        Subcommand::Destroy(cmd) => match cmd.run() {
            Ok(msg) => {
                println!("{}", msg);
            }
            Err(err) => {
                eprintln!("{:?}", err);
            }
        },
        Subcommand::Add(cmd) => match cmd.run() {
            Ok(msg) => {
                println!("{}", msg);
            }
            Err(err) => {
                eprintln!("{:?}", err);
            }
        },
        Subcommand::Build(cmd) => match cmd.run() {
            Ok(msg) => {
                println!("{}", msg);
            }
            Err(err) => {
                eprintln!("{:?}", err);
            }
        },
        Subcommand::Completions(cmd) => cmd.run(),
    }
}
