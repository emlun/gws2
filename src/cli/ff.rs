use clap::App;
use clap::Arg;
use clap::ArgMatches;
use clap::SubCommand;

use commands::common::Command;
use commands::ff::FastForward;

pub fn subcommand_def<'a>() -> App<'a, 'a> {
    SubCommand::with_name("ff")
        .about("Print status for each project, but fetch remotes and fast-forward branches first")
    .after_help("Branches are fast-forwarded to their upstreams as configured in git, outside the workspace configuration file.

If no <path>s are given, fast-forward all projects.")
        .arg(
            Arg::with_name("path")
                .multiple(true)
                .help("Project paths to be fast-forwarded"),
        )
}

pub fn make_command(matches: &ArgMatches) -> FastForward {
    FastForward {
        fetch_command: super::fetch::make_command(matches),
    }
}

pub fn make_cli_command(matches: &ArgMatches) -> Command {
    Command::RepositoryCommand(Box::new(make_command(matches)))
}
