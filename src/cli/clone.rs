use std::collections::HashSet;

use clap::App;
use clap::Arg;
use clap::ArgMatches;
use clap::SubCommand;

use commands::clone::Clone;


pub fn subcommand_def<'a>() -> App<'a, 'a> {
    SubCommand::with_name("clone")
        .about("Selectively clone specific repositories from projects list")
        .arg(
            Arg::with_name("path")
                .multiple(true)
                .help("Project paths to be cloned")
        )
}

pub fn make_command(matches: &ArgMatches) -> Clone {
    Clone {
        projects: matches.values_of("path")
            .map(|values| values.into_iter().map(&str::to_string).collect())
            .unwrap_or(HashSet::new())
    }
}
