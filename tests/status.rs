extern crate git2;
extern crate gws2;
extern crate tempdir;

mod util;

use std::path::Path;

use gws2::config::read::read_workspace_file;
use gws2::data::status::BranchStatus;
use gws2::data::status::DirtyState;
use gws2::data::status::ProjectStatusMethods;
use gws2::data::status::RepositoryStatus;

use util::in_example_workspace;
use util::set;


#[test]
fn status_produces_correct_data_structure() {
    in_example_workspace(|| {
        let workspace = read_workspace_file(Path::new(".projects.gws")).unwrap();

        let project_stati: Vec<Option<RepositoryStatus>> = workspace.projects.iter()
            .map(ProjectStatusMethods::status)
            .map(|r| r.map(|result| result.unwrap()))
            .collect();

        assert_eq!(
            project_stati,
            vec![
                // clean
                Some(set(vec![
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
                Some(set(vec![
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
                Some(set(vec![
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

                // changes/new_files
                Some(set(vec![
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
                Some(set(vec![
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

                None
            ]
        );
    });
}
