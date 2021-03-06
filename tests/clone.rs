extern crate git2;
extern crate gws;

mod util;
mod util2;

use std::collections::HashSet;
use std::hash::Hash;

use git2::Repository;

use gws::color::palette::Palette;
use gws::commands::clone::Clone;
use gws::commands::common::DirectoryCommand;
use gws::config::data::Workspace;

use util::in_example_workspace;
use util::Error;
use util2::in_workspace_with_projects_file;
use util2::with_bundled_ssh_key_in_agent;

pub fn hash_set<I, T>(items: I) -> HashSet<T>
where
    I: IntoIterator<Item = T>,
    T: Eq + Hash,
{
    items.into_iter().collect()
}

#[test]
fn clone_creates_repo() -> Result<(), Error> {
    in_example_workspace(|working_dir, workspace: Workspace| {
        let command: Clone = Clone {
            projects: hash_set(vec!["missing_repository".to_string()]),
        };

        assert_eq!(false, working_dir.join("missing_repository").exists());
        command
            .run(working_dir, &workspace, &Palette::default())
            .expect("Clone command failed");
        assert!(working_dir.join("missing_repository").exists());
        assert_eq!(false, working_dir.join("missing_repository_2").exists());

        Ok(())
    })
}

#[test]
fn clone_supports_multiple_arguments() -> Result<(), Error> {
    in_example_workspace(|working_dir, workspace: Workspace| {
        let command: Clone = Clone {
            projects: hash_set(vec![
                "missing_repository".to_string(),
                "missing_repository_2".to_string(),
            ]),
        };

        assert_eq!(false, working_dir.join("missing_repository").exists());
        assert_eq!(false, working_dir.join("missing_repository_2").exists());
        command
            .run(working_dir, &workspace, &Palette::default())
            .expect("Clone command failed");
        assert!(working_dir.join("missing_repository").exists());
        assert!(working_dir.join("missing_repository_2").exists());

        Ok(())
    })
}

#[test]
fn clone_creates_extra_remotes() -> Result<(), Error> {
    in_example_workspace(|working_dir, workspace: Workspace| {
        let command: Clone = Clone {
            projects: hash_set(vec!["missing_repository".to_string()]),
        };

        let repo_path: String = workspace
            .projects
            .iter()
            .find(|proj| proj.path == "missing_repository")
            .unwrap()
            .path
            .clone();

        command
            .run(working_dir, &workspace, &Palette::default())
            .expect("Clone command failed");

        assert_eq!(
            Repository::open(working_dir.join(&repo_path))
                .expect("Failed to open repo")
                .remotes()
                .expect("Failed to get remotes")
                .iter()
                .map(Option::unwrap)
                .collect::<HashSet<&str>>(),
            hash_set(vec!["origin", "ahead"])
        );

        Ok(())
    })
}

#[test]
fn clone_works_with_public_https() -> Result<(), Error> {
    let projects_contents = "gws2 | https://github.com/emlun/gws2.git";

    in_workspace_with_projects_file(projects_contents, |working_dir, workspace: Workspace| {
        let command: Clone = Clone {
            projects: hash_set(vec!["gws2".to_string()]),
        };

        let repo_path: String = workspace
            .projects
            .iter()
            .find(|proj| proj.path == "gws2")
            .unwrap()
            .path
            .clone();

        command
            .run(working_dir, &workspace, &Palette::default())
            .expect("Clone command failed");

        assert_eq!(
            Repository::open(working_dir.join(&repo_path))
                .expect("Failed to open repo")
                .remotes()
                .expect("Failed to get remotes")
                .iter()
                .map(Option::unwrap)
                .collect::<HashSet<&str>>(),
            hash_set(vec!["origin"])
        );

        Ok(())
    })
}

#[test]
fn clone_works_with_ssh() -> Result<(), Error> {
    let projects_contents = "gws2 | git@github.com:emlun/gws2.git";

    in_workspace_with_projects_file(
        projects_contents,
        with_bundled_ssh_key_in_agent(|working_dir, workspace: Workspace| {
            let command: Clone = Clone {
                projects: hash_set(vec!["gws2".to_string()]),
            };

            let repo_path: String = workspace
                .projects
                .iter()
                .find(|proj| proj.path == "gws2")
                .unwrap()
                .path
                .clone();

            command
                .run(working_dir, &workspace, &Palette::default())
                .expect("Clone command failed");

            assert_eq!(
                Repository::open(working_dir.join(&repo_path))
                    .expect("Failed to open repo")
                    .remotes()
                    .expect("Failed to get remotes")
                    .iter()
                    .map(Option::unwrap)
                    .collect::<HashSet<&str>>(),
                hash_set(vec!["origin"])
            );

            Ok(())
        })?,
    )
}
