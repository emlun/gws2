extern crate git2;
extern crate gws;
extern crate tempdir;

mod util;

use std::collections::BTreeSet;
use std::collections::HashSet;

use gws::commands::common::RepositoryCommand;
use gws::commands::error::Error;
use gws::commands::status::Status;
use gws::data::status::BranchStatus;
use gws::data::status::DirtyState;
use gws::data::status::RepositoryStatus;

use util::in_example_workspace;

pub fn tree_set<I, T>(items: I) -> BTreeSet<T>
where
    I: IntoIterator<Item = T>,
    T: Ord,
{
    items.into_iter().collect()
}

#[test]
fn status_produces_correct_data_structure() {
    in_example_workspace(|working_dir, workspace| {
        let command = Status {
            only_changes: false,
            projects: HashSet::new(),
        };

        let project_stati: Vec<Result<RepositoryStatus, Error>> = command
            .make_report(working_dir, &workspace)
            .into_iter()
            .map(|(_, s)| s)
            .collect();

        assert_eq!(
            project_stati,
            vec![
                // changes/changed_files
                Ok(tree_set(vec![
                    BranchStatus {
                        name: "master".to_string(),
                        upstream_name: Some("origin/master".to_string()),
                        dirty: DirtyState::UncommittedChanges,
                        is_head: true,
                        in_sync: Some(true),
                        upstream_fetched: false,
                        fast_forwarded: false,
                    },
                    BranchStatus {
                        name: "master2".to_string(),
                        upstream_name: Some("ahead/master".to_string()),
                        dirty: DirtyState::Clean,
                        is_head: false,
                        in_sync: Some(true),
                        upstream_fetched: false,
                        fast_forwarded: false,
                    }
                ])),
                // changes/new_files
                Ok(tree_set(vec![
                    BranchStatus {
                        name: "master".to_string(),
                        upstream_name: Some("origin/master".to_string()),
                        dirty: DirtyState::UntrackedFiles,
                        is_head: true,
                        in_sync: Some(true),
                        upstream_fetched: false,
                        fast_forwarded: false,
                    },
                    BranchStatus {
                        name: "master2".to_string(),
                        upstream_name: Some("ahead/master".to_string()),
                        dirty: DirtyState::Clean,
                        is_head: false,
                        in_sync: Some(true),
                        upstream_fetched: false,
                        fast_forwarded: false,
                    }
                ])),
                // clean
                Ok(tree_set(vec![
                    BranchStatus {
                        name: "master".to_string(),
                        upstream_name: Some("origin/master".to_string()),
                        dirty: DirtyState::Clean,
                        is_head: true,
                        in_sync: Some(true),
                        upstream_fetched: false,
                        fast_forwarded: false,
                    },
                    BranchStatus {
                        name: "feature".to_string(),
                        upstream_name: None,
                        dirty: DirtyState::Clean,
                        is_head: false,
                        in_sync: None,
                        upstream_fetched: false,
                        fast_forwarded: false,
                    }
                ])),
                // missing_repository
                Err(Error::RepositoryMissing),
                // missing_repository_2
                Err(Error::RepositoryMissing),
                // new_commit/diverged
                Ok(tree_set(vec![BranchStatus {
                    name: "master".to_string(),
                    upstream_name: Some("origin/master".to_string()),
                    dirty: DirtyState::Clean,
                    is_head: true,
                    in_sync: Some(false),
                    upstream_fetched: false,
                    fast_forwarded: false,
                },])),
                // new_commit/local
                Ok(tree_set(vec![
                    BranchStatus {
                        name: "master".to_string(),
                        upstream_name: Some("origin/master".to_string()),
                        dirty: DirtyState::Clean,
                        is_head: true,
                        in_sync: Some(false),
                        upstream_fetched: false,
                        fast_forwarded: false,
                    },
                    BranchStatus {
                        name: "master2".to_string(),
                        upstream_name: Some("ahead/master".to_string()),
                        dirty: DirtyState::Clean,
                        is_head: false,
                        in_sync: Some(true),
                        upstream_fetched: false,
                        fast_forwarded: false,
                    }
                ])),
                // new_commit/remote
                Ok(tree_set(vec![
                    BranchStatus {
                        name: "master".to_string(),
                        upstream_name: Some("origin/master".to_string()),
                        dirty: DirtyState::Clean,
                        is_head: true,
                        in_sync: Some(true),
                        upstream_fetched: false,
                        fast_forwarded: false,
                    },
                    BranchStatus {
                        name: "master2".to_string(),
                        upstream_name: Some("ahead/master".to_string()),
                        dirty: DirtyState::Clean,
                        is_head: false,
                        in_sync: Some(false),
                        upstream_fetched: false,
                        fast_forwarded: false,
                    }
                ])),
                // new_commit/unfetched_remote
                Ok(tree_set(vec![
                    BranchStatus {
                        name: "master".to_string(),
                        upstream_name: Some("origin/master".to_string()),
                        dirty: DirtyState::Clean,
                        is_head: true,
                        in_sync: Some(true),
                        upstream_fetched: false,
                        fast_forwarded: false,
                    },
                    BranchStatus {
                        name: "master2".to_string(),
                        upstream_name: Some("ahead/master".to_string()),
                        dirty: DirtyState::Clean,
                        is_head: false,
                        in_sync: Some(true),
                        upstream_fetched: false,
                        fast_forwarded: false,
                    }
                ])),
                // no_upstream
                Ok(tree_set(vec![BranchStatus {
                    name: "master".to_string(),
                    upstream_name: None,
                    dirty: DirtyState::Clean,
                    is_head: true,
                    in_sync: None,
                    upstream_fetched: false,
                    fast_forwarded: false,
                },])),
            ]
        );

        Ok(())
    });
}

#[test]
fn status_ignores_clean_repos_with_only_changes() {
    in_example_workspace(|working_dir, workspace| {
        let command = Status {
            only_changes: true,
            projects: HashSet::new(),
        };

        let project_stati: Vec<Result<RepositoryStatus, Error>> = command
            .make_report(working_dir, &workspace)
            .into_iter()
            .map(|(_, s)| s)
            .collect();

        assert_eq!(
            project_stati,
            vec![
                // changes/changed_files
                Ok(tree_set(vec![
                    BranchStatus {
                        name: "master".to_string(),
                        upstream_name: Some("origin/master".to_string()),
                        dirty: DirtyState::UncommittedChanges,
                        is_head: true,
                        in_sync: Some(true),
                        upstream_fetched: false,
                        fast_forwarded: false,
                    },
                    BranchStatus {
                        name: "master2".to_string(),
                        upstream_name: Some("ahead/master".to_string()),
                        dirty: DirtyState::Clean,
                        is_head: false,
                        in_sync: Some(true),
                        upstream_fetched: false,
                        fast_forwarded: false,
                    }
                ])),
                // changes/new_files
                Ok(tree_set(vec![
                    BranchStatus {
                        name: "master".to_string(),
                        upstream_name: Some("origin/master".to_string()),
                        dirty: DirtyState::UntrackedFiles,
                        is_head: true,
                        in_sync: Some(true),
                        upstream_fetched: false,
                        fast_forwarded: false,
                    },
                    BranchStatus {
                        name: "master2".to_string(),
                        upstream_name: Some("ahead/master".to_string()),
                        dirty: DirtyState::Clean,
                        is_head: false,
                        in_sync: Some(true),
                        upstream_fetched: false,
                        fast_forwarded: false,
                    }
                ])),
                // new_commit/diverged
                Ok(tree_set(vec![BranchStatus {
                    name: "master".to_string(),
                    upstream_name: Some("origin/master".to_string()),
                    dirty: DirtyState::Clean,
                    is_head: true,
                    in_sync: Some(false),
                    upstream_fetched: false,
                    fast_forwarded: false,
                },])),
                // new_commit/local
                Ok(tree_set(vec![
                    BranchStatus {
                        name: "master".to_string(),
                        upstream_name: Some("origin/master".to_string()),
                        dirty: DirtyState::Clean,
                        is_head: true,
                        in_sync: Some(false),
                        upstream_fetched: false,
                        fast_forwarded: false,
                    },
                    BranchStatus {
                        name: "master2".to_string(),
                        upstream_name: Some("ahead/master".to_string()),
                        dirty: DirtyState::Clean,
                        is_head: false,
                        in_sync: Some(true),
                        upstream_fetched: false,
                        fast_forwarded: false,
                    }
                ])),
                // new_commit/remote
                Ok(tree_set(vec![
                    BranchStatus {
                        name: "master".to_string(),
                        upstream_name: Some("origin/master".to_string()),
                        dirty: DirtyState::Clean,
                        is_head: true,
                        in_sync: Some(true),
                        upstream_fetched: false,
                        fast_forwarded: false,
                    },
                    BranchStatus {
                        name: "master2".to_string(),
                        upstream_name: Some("ahead/master".to_string()),
                        dirty: DirtyState::Clean,
                        is_head: false,
                        in_sync: Some(false),
                        upstream_fetched: false,
                        fast_forwarded: false,
                    }
                ])),
                // no_upstream
                Ok(tree_set(vec![BranchStatus {
                    name: "master".to_string(),
                    upstream_name: None,
                    dirty: DirtyState::Clean,
                    is_head: true,
                    in_sync: None,
                    upstream_fetched: false,
                    fast_forwarded: false,
                },])),
            ]
        );

        Ok(())
    });
}
