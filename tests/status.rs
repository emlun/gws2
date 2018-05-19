extern crate git2;
extern crate gws2;
extern crate tempdir;

use std::collections::BTreeSet;
use std::path::Path;
use std::path::PathBuf;

use git2::Repository;
use tempdir::TempDir;

use gws2::config::read::read_workspace_file;
use gws2::data::status::BranchStatus;
use gws2::data::status::DirtyState;
use gws2::data::status::RepositoryMethods;


fn in_example_workspace<R>(test: fn() -> R) {
    in_example_workspace_inner(test).unwrap();
}

fn in_example_workspace_inner<R>(test: fn() -> R) -> Result<R, std::io::Error> {
    let tmpdir = TempDir::new("gws2-test")?;

    let projects_gws_path: PathBuf = std::env::current_dir()?
        .join("tests")
        .join("test_projects.gws")
    ;

    let setup_script_path: PathBuf = std::env::current_dir()?
        .join("tests")
        .join("setup-workspace.sh")
    ;

    std::fs::copy(projects_gws_path.as_path(), tmpdir.path().join(".projects.gws").as_path())?;

    let setup_output = std::process::Command::new("sh")
        .arg(setup_script_path.as_os_str())
        .arg(tmpdir.path().as_os_str())
        .output()
        .expect("Failed to set up workspace");

    if !setup_output.status.success() {
        panic!("Failed to set up workspace:\n{}", String::from_utf8(setup_output.stderr).unwrap());
    }

    ::std::env::set_current_dir(tmpdir.path())?;

    Ok(test())
}


#[test]
fn status_produces_correct_data_structure() {
    in_example_workspace(|| {
        let workspace = read_workspace_file(Path::new(".projects.gws")).unwrap();

        // clean
        assert_eq!(
            Repository::open(&workspace.projects[0].path)
                .unwrap()
                .project_status(&workspace.projects[0])
                .unwrap(),
            vec![
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
            ].into_iter().collect::<BTreeSet<BranchStatus>>()
        );

        // new_commit/local
        assert_eq!(
            Repository::open(&workspace.projects[1].path)
                .unwrap()
                .project_status(&workspace.projects[1])
                .unwrap(),
            vec![
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
            ].into_iter().collect::<BTreeSet<BranchStatus>>()
        );

        // new_commit/remote
        assert_eq!(
            Repository::open(&workspace.projects[1].path)
                .unwrap()
                .project_status(&workspace.projects[1])
                .unwrap(),
            vec![
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
            ].into_iter().collect::<BTreeSet<BranchStatus>>()
        );

        // changes/new_files
        assert_eq!(
            Repository::open(&workspace.projects[1].path)
                .unwrap()
                .project_status(&workspace.projects[1])
                .unwrap(),
            vec![
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
            ].into_iter().collect::<BTreeSet<BranchStatus>>()
        );

        // changes/changed_files
        assert_eq!(
            Repository::open(&workspace.projects[1].path)
                .unwrap()
                .project_status(&workspace.projects[1])
                .unwrap(),
            vec![
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
            ].into_iter().collect::<BTreeSet<BranchStatus>>()
        );
    });
}
