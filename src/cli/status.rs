use clap::App;
use clap::Arg;
use clap::ArgMatches;
use clap::SubCommand;

use commands::status::Status;


pub fn subcommand_def<'a>() -> App<'a, 'a> {
  SubCommand::with_name("status")
    .about("Print status for all repositories in the workspace")
    .arg(
      Arg::with_name("only-changes")
        .long("only-changes")
        .help("Only print out-of-sync repositories and branches")
    )
}

pub fn make_command(matches: &ArgMatches) -> Status {
  Status {
    only_changes: matches.is_present("only-changes"),
  }
}
