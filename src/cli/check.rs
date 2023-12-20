use clap::App;
use clap::Arg;
use clap::ArgMatches;
use clap::SubCommand;

use crate::commands::check::Check;
use crate::commands::common::Command;

pub fn subcommand_def<'a>() -> App<'a, 'a> {
    SubCommand::with_name("check")
        .about(
            "Print difference between projects list and workspace (known/unknown/missing/ignored)",
        )
        .after_help(
            "Your entire working directory is checked and compared against your .projects.gws file.

- If projects are in your filesystem and in .projects.gws, they are listed as known.
- If projects are in your filesystem but not in .projects.gws, they are listed as unknown.
- If projects are in .projects.gws but not in your filesystem, they are listed as missing.
- If projects are in .ignore.gws, they are listed as ignored.",
        )
        .arg(
            Arg::with_name("path")
                .multiple(true)
                .help("Project paths to be checked"),
        )
}

pub fn make_command(_matches: &ArgMatches) -> Check {
    Check {}
}

pub fn make_cli_command(matches: &ArgMatches) -> Command {
    Command::DirectoryCommand(Box::new(make_command(matches)))
}
