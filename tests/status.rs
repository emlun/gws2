extern crate git2;
extern crate gws2;
extern crate tempdir;

mod util;

use std::collections::BTreeSet;

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
    let project_stati: Vec<Option<RepositoryStatus>> = workspace.projects.iter()
      .map(|p| p.status(working_dir))
      .map(|r| r.map(|result| result.unwrap()))
      .collect();

    assert_eq!(
      project_stati,
      vec![
        // clean
        Some(tree_set(vec![
          BranchStatus {
            name: "master".to_string(),
            upstream_name: "origin/master".to_string(),
            dirty: DirtyState::Clean,
            is_head: true,
            in_sync: Some(true),
          },
          BranchStatus {
            name: "master2".to_string(),
            upstream_name: "remote2/master".to_string(),
            dirty: DirtyState::Clean,
            is_head: false,
            in_sync: Some(true),
          }
        ])),

        // new_commit/local
        Some(tree_set(vec![
          BranchStatus {
            name: "master".to_string(),
            upstream_name: "origin/master".to_string(),
            dirty: DirtyState::Clean,
            is_head: true,
            in_sync: Some(false),
          },
          BranchStatus {
            name: "master2".to_string(),
            upstream_name: "remote2/master".to_string(),
            dirty: DirtyState::Clean,
            is_head: false,
            in_sync: Some(true),
          }
        ])),

        // new_commit/remote
        Some(tree_set(vec![
          BranchStatus {
            name: "master".to_string(),
            upstream_name: "origin/master".to_string(),
            dirty: DirtyState::Clean,
            is_head: true,
            in_sync: Some(false),
          },
          BranchStatus {
            name: "master2".to_string(),
            upstream_name: "remote2/master".to_string(),
            dirty: DirtyState::Clean,
            is_head: false,
            in_sync: Some(true),
          }
        ])),

        // new_commit/unfetched_remote
        Some(tree_set(vec![
          BranchStatus {
            name: "master".to_string(),
            upstream_name: "origin/master".to_string(),
            dirty: DirtyState::Clean,
            is_head: true,
            in_sync: Some(true),
          },
          BranchStatus {
            name: "master2".to_string(),
            upstream_name: "remote2/master".to_string(),
            dirty: DirtyState::Clean,
            is_head: false,
            in_sync: Some(true),
          }
        ])),

        // changes/new_files
        Some(tree_set(vec![
          BranchStatus {
            name: "master".to_string(),
            upstream_name: "origin/master".to_string(),
            dirty: DirtyState::UntrackedFiles,
            is_head: true,
            in_sync: Some(true),
          },
          BranchStatus {
            name: "master2".to_string(),
            upstream_name: "remote2/master".to_string(),
            dirty: DirtyState::Clean,
            is_head: false,
            in_sync: Some(true),
          }
        ])),

        // changes/changed_files
        Some(tree_set(vec![
          BranchStatus {
            name: "master".to_string(),
            upstream_name: "origin/master".to_string(),
            dirty: DirtyState::UncommittedChanges,
            is_head: true,
            in_sync: Some(true),
          },
          BranchStatus {
            name: "master2".to_string(),
            upstream_name: "remote2/master".to_string(),
            dirty: DirtyState::Clean,
            is_head: false,
            in_sync: Some(true),
          }
        ])),

        // missing_repository
        None,

        // missing_repository_2
        None,
      ]
    );

    Ok(())
  });
}
