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

mod crate_info;
mod commands;

use crate_info::crate_author;
use crate_info::crate_description;
use crate_info::crate_name;
use crate_info::crate_version;

use clap::App;
use clap::Arg;
use clap::SubCommand;


fn main() {
    let matches = App::new(crate_name())
        .version(crate_version())
        .about(crate_description())
        .author(crate_author())

        .subcommand(
            SubCommand::with_name("status")
                .about("Print status for all repositories in the workspace")
                .arg(
                    Arg::with_name("only-changes")
                        .help("Only print out-of-sync repositories and branches")
                )
        )

        .get_matches();
    println!("{:?}", matches);
}
