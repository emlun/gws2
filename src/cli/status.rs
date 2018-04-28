use clap::App;
use clap::Arg;
use clap::SubCommand;


pub fn subcommand_def<'a>() -> App<'a, 'a> {
    SubCommand::with_name("status")
        .about("Print status for all repositories in the workspace")
        .arg(
            Arg::with_name("only-changes")
                .help("Only print out-of-sync repositories and branches")
        )
}
