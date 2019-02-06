use std::collections::HashSet;

use clap::App;
use clap::Arg;
use clap::ArgMatches;
use clap::SubCommand;

use commands::fetch::Fetch;


pub fn subcommand_def<'a>() -> App<'a, 'a> {
  SubCommand::with_name("fetch")
    .about("Print status for each project, but fetch remotes first")
    .arg(
      Arg::with_name("path")
        .multiple(true)
        .help("Project paths to be fetched")
    )
}

pub fn make_command(matches: &ArgMatches) -> Fetch {
  Fetch {
    projects: matches.values_of("path")
      .map(|values| values.into_iter().map(&str::to_string).collect())
      .unwrap_or(HashSet::new())
  }
}
