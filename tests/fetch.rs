extern crate git2;
extern crate gws2;
extern crate tempdir;

mod util;

use std::collections::HashSet;
use std::hash::Hash;
use std::path::Path;

use git2::BranchType;
use git2::Commit;
use git2::Repository;

use gws2::color::palette::Palette;
use gws2::commands::common::RepositoryCommand;
use gws2::commands::fetch::Fetch;
use gws2::commands::status::Status;
use gws2::config::data::Workspace;

use util::in_example_workspace;
use util::Error;

pub fn hash_set<I, T>(items: I) -> HashSet<T>
where
    I: IntoIterator<Item = T>,
    T: Eq + Hash,
{
    items.into_iter().collect()
}

#[test]
fn fetch_gets_refs_from_named_remotes() {
    fn resolve_ref<'repo>(name: &str, repo: &'repo Repository) -> Result<Commit<'repo>, Error> {
        Ok(repo
            .find_branch(name, BranchType::Remote)?
            .get()
            .peel_to_commit()?)
    }

    in_example_workspace(|working_dir, workspace: Workspace| {
        let project_path = "new_commit/unfetched_remote";

        let command: Fetch = Fetch {
            status_command: Status {
                only_changes: false,
                projects: HashSet::new(),
            },
        };

        let repo: Repository = Repository::open(working_dir.join(project_path))?;

        let master_reference_before: Commit = resolve_ref("origin/master", &repo)?;
        let master2_reference_before: Commit = resolve_ref("ahead/master", &repo)?;

        command
            .run(working_dir, &workspace, &Palette::default())
            .expect("Fetch command failed");

        let master_reference_after: Commit = resolve_ref("origin/master", &repo)?;
        let master2_reference_after: Commit = resolve_ref("ahead/master", &repo)?;

        assert_eq!(master_reference_after.id(), master_reference_before.id());

        assert_ne!(master2_reference_after.id(), master2_reference_before.id());
        assert_eq!(
            master2_reference_after.parents().next().unwrap().id(),
            master2_reference_before.id()
        );

        Ok(())
    });
}

#[test]
fn fetch_reports_updates() {
    in_example_workspace(|working_dir, workspace: Workspace| {
        let project_path = "new_commit/unfetched_remote";

        let command: Fetch = Fetch {
            status_command: Status {
                only_changes: false,
                projects: HashSet::new(),
            },
        };

        let status_report_1 = command.make_report(working_dir, &workspace);

        for (project, project_status) in status_report_1 {
            if project.path == project_path {
                let project_status = project_status.unwrap();
                for branch_status in project_status {
                    if branch_status.name == "master2" {
                        assert_eq!(branch_status.upstream_fetched, true);
                    } else {
                        assert_eq!(branch_status.upstream_fetched, false);
                    }
                }
            }
        }

        let status_report_2 = command.make_report(working_dir, &workspace);

        for (project, project_status) in status_report_2 {
            if project.path == project_path {
                let project_status = project_status.unwrap();
                for branch_status in project_status {
                    assert_eq!(branch_status.upstream_fetched, false);
                }
            }
        }

        Ok(())
    });
}

#[test]
fn fetch_fetches_all_projects_if_none_are_named() {
    in_example_workspace(|working_dir, workspace: Workspace| {
        let project_path = "new_commit/unfetched_remote";

        let command: Fetch = Fetch {
            status_command: Status {
                only_changes: false,
                projects: HashSet::new(),
            },
        };

        let status_report = command.make_report(working_dir, &workspace);

        for (project, project_status) in status_report {
            if project.path == project_path {
                let project_status = project_status.unwrap();
                for branch_status in project_status {
                    if branch_status.name == "master2" {
                        assert_eq!(branch_status.upstream_fetched, true);
                    } else {
                        assert_eq!(branch_status.upstream_fetched, false);
                    }
                }
            }
        }

        Ok(())
    });
}

#[test]
fn fetch_fetches_only_named_projects_if_any_are_named() {
    fn run_test(
        working_dir: &Path,
        workspace: Workspace,
        projects: HashSet<String>,
        should_fetch: bool,
    ) -> Result<(), Error> {
        let project_path = "new_commit/unfetched_remote";

        let command: Fetch = Fetch {
            status_command: Status {
                only_changes: false,
                projects,
            },
        };

        let status_report = command.make_report(working_dir, &workspace);

        for (project, project_status) in status_report {
            if project.path == project_path {
                let project_status = project_status.unwrap();
                for branch_status in project_status {
                    if branch_status.name == "master2" {
                        assert_eq!(branch_status.upstream_fetched, should_fetch);
                    } else {
                        assert_eq!(branch_status.upstream_fetched, false);
                    }
                }
            }
        }

        Ok(())
    }

    in_example_workspace(|working_dir, workspace: Workspace| {
        run_test(
            working_dir,
            workspace,
            hash_set(vec!["new_commit/local".to_string()]),
            false,
        )
    });

    in_example_workspace(|working_dir, workspace: Workspace| {
        run_test(
            working_dir,
            workspace,
            hash_set(vec!["new_commit/unfetched_remote".to_string()]),
            true,
        )
    })
}
