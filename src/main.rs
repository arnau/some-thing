use clap::{AppSettings, Clap};
use some::cli;

#[derive(Debug, Clap)]
enum Subcommand {
    #[clap(alias = "a")]
    Add(cli::add::Cmd),
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
        Subcommand::Add(cmd) => match cmd.run() {
            Ok(msg) => {
                println!("{}", msg);
            }
            Err(err) => {
                eprintln!("{:?}", err);
            }
        },
    }
}
