use clap::App;
use clap::Arg;
use clap::ArgMatches;
use clap::SubCommand;

use crate::commands::clone::Clone;
use crate::commands::common::Command;

pub fn subcommand_def<'a>() -> App<'a, 'a> {
    SubCommand::with_name("clone")
        .about("Selectively clone specific repositories from projects list")
        .arg(
            Arg::with_name("path")
                .multiple(true)
                .help("Project paths to be cloned"),
        )
}

pub fn make_command(matches: &ArgMatches) -> Clone {
    Clone {
        projects: matches
            .values_of("path")
            .map(|values| values.map(&str::to_string).collect())
            .unwrap_or_default(),
    }
}

pub fn make_cli_command(matches: &ArgMatches) -> Command {
    Command::DirectoryCommand(Box::new(make_command(matches)))
}
