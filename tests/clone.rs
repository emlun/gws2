extern crate git2;
extern crate gws2;
extern crate tempdir;

mod util;

use std::collections::HashSet;
use std::hash::Hash;

use git2::Repository;

use gws2::color::palette::Palette;
use gws2::commands::clone::Clone;
use gws2::commands::common::Command;
use gws2::config::data::Workspace;

use util::in_example_workspace;


pub fn hash_set<I, T>(items: I) -> HashSet<T>
  where I: IntoIterator<Item=T>,
        T: Eq + Hash,
{
  items.into_iter().collect()
}

#[test]
fn clone_creates_repo() {
  in_example_workspace(|working_dir, workspace: Workspace| {

    let command: Clone = Clone {
      projects: hash_set(vec!["missing_repository".to_string()])
    };

    assert_eq!(false, working_dir.join("missing_repository").exists());
    command.run(working_dir, &workspace, &Palette::default())
      .expect("Clone command failed");
    assert!(working_dir.join("missing_repository").exists());
    assert_eq!(false, working_dir.join("missing_repository_2").exists());

    Ok(())
  });
}

#[test]
fn clone_supports_multiple_arguments() {
  in_example_workspace(|working_dir, workspace: Workspace| {

    let command: Clone = Clone {
      projects: hash_set(vec!["missing_repository".to_string(), "missing_repository_2".to_string()])
    };

    assert_eq!(false, working_dir.join("missing_repository").exists());
    assert_eq!(false, working_dir.join("missing_repository_2").exists());
    command.run(working_dir, &workspace, &Palette::default())
      .expect("Clone command failed");
    assert!(working_dir.join("missing_repository").exists());
    assert!(working_dir.join("missing_repository_2").exists());

    Ok(())
  });
}

#[test]
fn clone_creates_extra_remotes() {
  in_example_workspace(|working_dir, workspace: Workspace| {

    let command: Clone = Clone {
      projects: hash_set(vec!["missing_repository".to_string()])
    };

    let repo_path: String = workspace.projects.iter()
      .find(|proj| proj.path == "missing_repository")
      .unwrap()
      .path
      .clone();

    command.run(working_dir, &workspace, &Palette::default())
      .expect("Clone command failed");

    assert_eq!(
      Repository::open(working_dir.join(&repo_path))
        .expect("Failed to open repo")
        .remotes()
        .expect("Failed to get remotes")
        .iter()
        .map(Option::unwrap)
        .collect::<HashSet<&str>>(),
      hash_set(vec!["origin", "remote2"])
    );

    Ok(())
  });
}
