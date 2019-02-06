use clap::App;
use clap::ArgMatches;
use clap::SubCommand;

use commands::update::Update;


pub fn subcommand_def<'a>() -> App<'a, 'a> {
  SubCommand::with_name("update")
    .about("Clone any repositories in the projects list that are missing in the workspace")
}

pub fn make_command(_matches: &ArgMatches) -> Update {
  Update {}
}
