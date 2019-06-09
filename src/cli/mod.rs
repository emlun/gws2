// pub mod check;
pub mod clone;
pub mod fetch;
pub mod ff;
// pub mod init;
pub mod main;
mod status;
pub mod update;

use clap::App;
use clap::Arg;
use clap::Shell;
use clap::SubCommand;

use crate::crate_info::crate_author;
use crate::crate_info::crate_description;
use crate::crate_info::crate_name;
use crate::crate_info::crate_version;

pub fn build_cli() -> App<'static, 'static> {
    let chdir_arg = Arg::with_name("dir")
        .short("C")
        .long("chdir")
        .help("Change to <dir> before doing anything")
        .takes_value(true);

    App::new(crate_name())
        .version(crate_version())
        .about(crate_description())
        .author(crate_author())
        .arg(chdir_arg)
        .arg(
            Arg::with_name("no-config")
                .long("no-config")
                .help("Don't read any config files"),
        )
        .subcommand(clone::subcommand_def())
        .subcommand(completions())
        .subcommand(fetch::subcommand_def())
        .subcommand(ff::subcommand_def())
        .subcommand(status::subcommand_def())
        .subcommand(update::subcommand_def())
}

fn completions<'a>() -> App<'a, 'a> {
    SubCommand::with_name("completions")
        .about("Generate shell completion scripts")
        .long_about("Result is written to standard output.")
        .arg(
            Arg::with_name("shell")
                .required(true)
                .possible_values(&Shell::variants()),
        )
}
