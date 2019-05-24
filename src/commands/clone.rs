use std::collections::HashSet;
use std::path::Path;

use git2::Repository;

use super::common::exit_codes;
use super::common::format_message_line;
use super::common::format_project_header;
use super::common::DirectoryCommand;
use super::error::Error;
use crate::color::palette::Palette;
use crate::config::data::Workspace;

pub struct Clone {
    pub projects: HashSet<String>,
}

impl DirectoryCommand for Clone {
    fn run(
        &self,
        working_dir: &Path,
        workspace: &Workspace,
        palette: &Palette,
    ) -> Result<i32, Error> {
        let mut clone_failed: bool = false;
        let mut add_remote_failed: bool = false;

        for project in workspace
            .projects
            .iter()
            .filter(|proj| self.projects.contains(&proj.path))
        {
            println!("{}", format_project_header(&project, &palette));

            if working_dir.join(&project.path).exists() {
                println!(
                    "{}",
                    palette.clean.paint(format_message_line("Already exists"))
                );
            } else {
                println!(
                    "{}",
                    palette.cloning.paint(format_message_line("Cloning…"))
                );

                match Repository::clone_recurse(
                    &project.main_remote.url,
                    working_dir.join(&project.path),
                ) {
                    Ok(repo) => {
                        for extra_remote in &project.extra_remotes {
                            match repo.remote(&extra_remote.name, &extra_remote.url) {
                                Ok(_) => {}
                                Err(err) => {
                                    add_remote_failed = true;
                                    eprintln!(
                                        "Failed to add remote {}: {}",
                                        extra_remote.name, err
                                    );
                                }
                            }
                        }
                        println!("{}", palette.clean.paint(format_message_line("Cloned.")));
                    }
                    Err(err) => {
                        clone_failed = true;
                        eprintln!("Failed to clone project {}: {}", project.path, err);
                        println!("{}", palette.error.paint(format_message_line("Error")));
                    }
                }
            }
        }

        Ok(if clone_failed {
            exit_codes::CLONE_FAILED
        } else if add_remote_failed {
            exit_codes::CLONE_ADD_REMOTE_FAILED
        } else {
            exit_codes::OK
        })
    }
}
