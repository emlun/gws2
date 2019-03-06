use clap::App;
use clap::Arg;
use clap::ArgMatches;
use clap::SubCommand;

use commands::common::Command;
use commands::fetch::Fetch;

pub fn subcommand_def<'a>() -> App<'a, 'a> {
    SubCommand::with_name("fetch")
        .about("Print status for each project, but fetch remotes first")
        .after_help("If no <path>s are given, fetch all projects.")
        .arg(
            Arg::with_name("only-changes")
                .long("only-changes")
                .help("Only print out-of-sync repositories and branches"),
        )
        .arg(
            Arg::with_name("path")
                .multiple(true)
                .help("Project paths to be fetched"),
        )
}

pub fn make_command(matches: &ArgMatches) -> Fetch {
    Fetch {
        status_command: super::status::make_command(matches),
    }
}

pub fn make_cli_command(matches: &ArgMatches) -> Command {
    Command::RepositoryCommand(Box::new(make_command(matches)))
}
