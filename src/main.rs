use clap::{AppSettings, Parser};
use some::cli;

const CLI_NAME: &str = "some";

#[derive(Debug, Parser)]
enum Subcommand {
    #[clap(alias = "a")]
    Init(cli::init::Cmd),
    Add(cli::add::Cmd),
    Build(cli::build::Cmd),
    Destroy(cli::destroy::Cmd),
    Shell(cli::shell::Cmd),
}

#[derive(Debug, Parser)]
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
        Subcommand::Shell(cmd) => match cmd.run() {
            Ok(_) => {}
            Err(err) => {
                eprintln!("{:?}", err);
            }
        },
    }
}
