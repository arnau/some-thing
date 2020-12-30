use clap::{AppSettings, Clap};
use some::cli;

#[derive(Debug, Clap)]
enum Subcommand {
    #[clap(alias = "a")]
    Init(cli::init::Cmd),
    Add(cli::add::Cmd),
    Build(cli::build::Cmd),
    Destroy(cli::destroy::Cmd),
}

#[derive(Debug, Clap)]
#[clap(name = "some", version, global_setting(AppSettings::ColoredHelp))]
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
    }
}
