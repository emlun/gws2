use clap::App;
use clap::Arg;

use std::path::Path;

use color::palette::Palette;
use crate_info::crate_author;
use crate_info::crate_description;
use crate_info::crate_name;
use crate_info::crate_version;
use super::exit_codes;


pub fn main() -> i32 {

    let chdir_arg = Arg::with_name("dir")
        .short("C")
        .long("chdir")
        .help("Change to <dir> before doing anything")
        .takes_value(true)
    ;

    let matches = App::new(crate_name())
        .version(crate_version())
        .about(crate_description())
        .author(crate_author())

        .arg(chdir_arg)

        .subcommand(super::clone::subcommand_def())
        .subcommand(super::status::subcommand_def())

        .get_matches();

    println!("{:?}", matches);

    if let Some(chdir_arg) = matches.args.get("dir") {
        ::std::env::set_current_dir(
            Path::new(
                chdir_arg.vals[0]
                    .to_str()
                    .expect("Did not understand <dir> argument")
            )
        ).unwrap();
    }

    let palette = Palette::default();

    let subcommand_run: fn(&Palette) -> Result<i32, _> = match &matches.subcommand {
        None => ::commands::status::run,
        Some(sc) => match sc.name.as_ref() {
            "status" => ::commands::status::run,
            _ => panic!("Unknown subcommand: {}", sc.name),
        },
    };

    match subcommand_run(&palette) {
        Ok(status) => status,
        Err(_) => exit_codes::UNKNOWN_ERROR,
    }
}
