extern crate git2;
extern crate gws2;
extern crate tempdir;

mod util;

use std::collections::BTreeSet;

use gws2::commands::error::Error;
use gws2::data::status::BranchStatus;
use gws2::data::status::DirtyState;
use gws2::data::status::ProjectStatusMethods;
use gws2::data::status::RepositoryStatus;

use util::in_example_workspace;


pub fn tree_set<I, T>(items: I) -> BTreeSet<T>
  where I: IntoIterator<Item=T>,
        T: Ord,
{
  items.into_iter().collect()
}

#[test]
fn status_produces_correct_data_structure() {
  in_example_workspace(|working_dir, workspace| {
    let project_stati: Vec<Result<RepositoryStatus, Error>> = workspace.projects.iter()
      .map(|p| p.status(working_dir))
      .collect();

    assert_eq!(
      project_stati,
      vec![
        // clean
        Ok(tree_set(vec![
          BranchStatus {
            name: "master".to_string(),
            upstream_name: "origin/master".to_string(),
            dirty: DirtyState::Clean,
            is_head: true,
            in_sync: Some(true),
            upstream_fetched: false,
          },
          BranchStatus {
            name: "master2".to_string(),
            upstream_name: "remote2/master".to_string(),
            dirty: DirtyState::Clean,
            is_head: false,
            in_sync: Some(true),
            upstream_fetched: false,
          }
        ])),

        // new_commit/local
        Ok(tree_set(vec![
          BranchStatus {
            name: "master".to_string(),
            upstream_name: "origin/master".to_string(),
            dirty: DirtyState::Clean,
            is_head: true,
            in_sync: Some(false),
            upstream_fetched: false,
          },
          BranchStatus {
            name: "master2".to_string(),
            upstream_name: "remote2/master".to_string(),
            dirty: DirtyState::Clean,
            is_head: false,
            in_sync: Some(true),
            upstream_fetched: false,
          }
        ])),

        // new_commit/remote
        Ok(tree_set(vec![
          BranchStatus {
            name: "master".to_string(),
            upstream_name: "origin/master".to_string(),
            dirty: DirtyState::Clean,
            is_head: true,
            in_sync: Some(false),
            upstream_fetched: false,
          },
          BranchStatus {
            name: "master2".to_string(),
            upstream_name: "remote2/master".to_string(),
            dirty: DirtyState::Clean,
            is_head: false,
            in_sync: Some(true),
            upstream_fetched: false,
          }
        ])),

        // new_commit/unfetched_remote
        Ok(tree_set(vec![
          BranchStatus {
            name: "master".to_string(),
            upstream_name: "origin/master".to_string(),
            dirty: DirtyState::Clean,
            is_head: true,
            in_sync: Some(true),
            upstream_fetched: false,
          },
          BranchStatus {
            name: "master2".to_string(),
            upstream_name: "remote2/master".to_string(),
            dirty: DirtyState::Clean,
            is_head: false,
            in_sync: Some(true),
            upstream_fetched: false,
          }
        ])),

        // changes/new_files
        Ok(tree_set(vec![
          BranchStatus {
            name: "master".to_string(),
            upstream_name: "origin/master".to_string(),
            dirty: DirtyState::UntrackedFiles,
            is_head: true,
            in_sync: Some(true),
            upstream_fetched: false,
          },
          BranchStatus {
            name: "master2".to_string(),
            upstream_name: "remote2/master".to_string(),
            dirty: DirtyState::Clean,
            is_head: false,
            in_sync: Some(true),
            upstream_fetched: false,
          }
        ])),

        // changes/changed_files
        Ok(tree_set(vec![
          BranchStatus {
            name: "master".to_string(),
            upstream_name: "origin/master".to_string(),
            dirty: DirtyState::UncommittedChanges,
            is_head: true,
            in_sync: Some(true),
            upstream_fetched: false,
          },
          BranchStatus {
            name: "master2".to_string(),
            upstream_name: "remote2/master".to_string(),
            dirty: DirtyState::Clean,
            is_head: false,
            in_sync: Some(true),
            upstream_fetched: false,
          }
        ])),

        // missing_repository
        Err(Error::RepositoryMissing),

        // missing_repository_2
        Err(Error::RepositoryMissing),
      ]
    );

    Ok(())
  });
}
