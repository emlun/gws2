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

extern crate ansi_term;
extern crate clap;
extern crate git2;

mod cli;
mod color;
mod commands;
mod config;
mod crate_info;
mod data;

fn main() {
    std::process::exit(cli::main::main());
}
