extern crate git2;
extern crate gws2;
extern crate tempdir;

use std::collections::BTreeSet;
use std::env::current_dir;
use std::env::set_current_dir;
use std::fs::copy;
use std::io::Error;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use tempdir::TempDir;

use gws2::config::data::Workspace;
use gws2::config::read::read_workspace_file;


pub fn in_example_workspace<R>(test: fn(Workspace) -> R) {
    in_example_workspace_inner(test).unwrap();
}

fn in_example_workspace_inner<R>(test: fn(Workspace) -> R) -> Result<R, Error> {
    let tmpdir = TempDir::new("gws2-test")?;

    let projects_gws_path: PathBuf = current_dir()?
        .join("tests")
        .join("test_projects.gws")
    ;

    let setup_script_path: PathBuf = current_dir()?
        .join("tests")
        .join("setup-workspace.sh")
    ;

    copy(projects_gws_path.as_path(), tmpdir.path().join(".projects.gws").as_path())?;

    let setup_output = Command::new("sh")
        .arg(setup_script_path.as_os_str())
        .arg(tmpdir.path().as_os_str())
        .output()
        .expect("Failed to set up workspace");

    if !setup_output.status.success() {
        panic!("Failed to set up workspace:\n{}", String::from_utf8(setup_output.stderr).unwrap());
    }

    set_current_dir(tmpdir.path())?;

    let workspace = read_workspace_file(Path::new(".projects.gws")).unwrap();
    Ok(test(workspace))
}

pub fn set<I, T>(items: I) -> BTreeSet<T>
    where I: IntoIterator<Item=T>,
          T: Ord,
{
    items.into_iter().collect()
}
