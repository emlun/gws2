use clap::App;
use clap::Arg;
use clap::ArgMatches;
use clap::SubCommand;

use crate::commands::common::Command;
use crate::commands::status::Status;

pub fn subcommand_def<'a>() -> App<'a, 'a> {
    SubCommand::with_name("status")
        .about("Print status for all repositories in the workspace")
        .arg(
            Arg::with_name("only-changes")
                .long("only-changes")
                .help("Only print out-of-sync repositories and branches"),
        )
        .arg(
            Arg::with_name("path")
                .multiple(true)
                .help("Project paths to show status for"),
        )
}

pub fn make_command(matches: &ArgMatches) -> Status {
    Status {
        only_changes: matches.is_present("only-changes"),
        projects: matches
            .values_of("path")
            .map(|values| values.map(&str::to_string).collect())
            .unwrap_or_default(),
    }
}

pub fn make_cli_command(matches: &ArgMatches) -> Command {
    Command::RepositoryCommand(Box::new(make_command(matches)))
}
