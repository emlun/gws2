use std::path::Path;

use clap::App;
use clap::Arg;

use color::palette::Palette;
use crate_info::crate_author;
use crate_info::crate_description;
use crate_info::crate_name;
use crate_info::crate_version;


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

        .subcommand(super::status::subcommand_def())

        .get_matches();

    println!("{:?}", matches);

    let subcommand = match matches.subcommand {
        None => super::status::subcommand_def().get_matches(),
        Some(sc) => sc.matches,
    };

    if let Some(chdir_arg) = matches.args.get("dir") {
        ::std::env::set_current_dir(
            Path::new(
                chdir_arg.vals[0]
                    .to_str()
                    .expect("Did not understand <dir> argument")
            )
        ).unwrap();
    }

    println!("subcommand: {:?}", subcommand);

    let palette = Palette::default();

    match ::commands::status::run(&palette) {
        Ok(()) => 0,
        Err(_) => 1,
    }
}
