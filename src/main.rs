/*
gws2: Colorful KISS helper for git workspaces
Copyright (C) 2018  Emil Lundberg <emil@emlun.se>

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

extern crate clap;
extern crate git2;

mod cli;
mod commands;
mod config;
mod crate_info;

use crate_info::crate_author;
use crate_info::crate_description;
use crate_info::crate_name;
use crate_info::crate_version;

use clap::App;


fn real_main() -> i32 {
    let matches = App::new(crate_name())
        .version(crate_version())
        .about(crate_description())
        .author(crate_author())

        .subcommand(cli::status::subcommand_def())

        .get_matches();

    println!("{:?}", matches);

    let subcommand = match matches.subcommand {
        None => cli::status::subcommand_def().get_matches(),
        Some(sc) => sc.matches,
    };

    println!("subcommand: {:?}", subcommand);

    match commands::status::run() {
        Ok(()) => 0,
        Err(_) => 1,
    }
}

fn main() {
    std::process::exit(real_main());
}
