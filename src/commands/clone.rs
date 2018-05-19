use std::collections::HashSet;
use std::path::Path;

use git2::Repository;

use cli::exit_codes;
use color::palette::Palette;
use config::data::Workspace;
use data::status::ProjectStatusMethods;
use super::common::Command;
use super::common::format_message_line;
use super::common::format_project_header;


pub struct Clone {
    pub projects: HashSet<String>,
}

impl Command for Clone {
    fn run(&self, workspace: Workspace, palette: &Palette) -> Result<i32, ::git2::Error> {
        let mut exit_code = exit_codes::OK;

        for project in workspace.projects.into_iter()
            .filter(|proj|
                self.projects.contains(&proj.path)
            )
        {
            println!("{}", format_project_header(&project, &palette));

            match project.status() {
                Some(_) => {
                    println!("{}", palette.clean.paint(format_message_line("Already exists")));
                },
                None => {
                    println!("{}", palette.cloning.paint(format_message_line("Cloning…")));
                    match Repository::clone(
                        &project.remotes.into_iter().next().unwrap().url,
                        Path::new(&project.path)
                    ) {
                        Ok(_) => println!("{}", palette.clean.paint(format_message_line("Cloned."))),
                        Err(err) => {
                            exit_code = exit_codes::CLONE_FAILED;
                            eprintln!("{}", err);
                            println!("{}", palette.error.paint(format_message_line("Error")));
                        }
                    }
                },
            }
        }

        Ok(exit_code)
    }
}
