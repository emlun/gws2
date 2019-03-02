extern crate git2;
extern crate gws2;
extern crate tempdir;

use std::fs::copy;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use tempdir::TempDir;

use gws2::config::data::Workspace;
use gws2::config::read::read_workspace_file;


#[derive(Debug)]
pub enum Error {
  IoError(::std::io::Error),
  Git2Error(::git2::Error)
}

impl From<::std::io::Error> for Error {
  fn from(e: ::std::io::Error) -> Error {
    Error::IoError(e)
  }
}

impl From<::git2::Error> for Error {
  fn from(e: ::git2::Error) -> Error {
    Error::Git2Error(e)
  }
}

pub fn in_example_workspace<T>(
  test: fn(&Path, Workspace) -> Result<T, Error>
) {
  let result = in_example_workspace_inner(test);
  assert!(result.is_ok(), format!("{:?}", result.err()));
}

fn in_example_workspace_inner<T, E>(
  test: fn(&Path, Workspace) -> Result<T, E>
) -> Result<T, Error>
  where Error: From<E>
{
  let tmpdir = TempDir::new("gws2-test")?;

  let projects_gws_path: PathBuf = Path::new("tests").join("test_projects.gws");
  let setup_script_path: PathBuf = Path::new("tests").join("setup-workspace.sh");

  copy(projects_gws_path, tmpdir.path().join(".projects.gws"))?;

  let setup_output = Command::new("sh")
    .arg(setup_script_path.as_os_str())
    .arg(tmpdir.path().as_os_str())
    .output()
    .expect("Failed to set up workspace");

  if !setup_output.status.success() {
    panic!(
      "Failed to set up workspace:\n{}\n{}",
      String::from_utf8(setup_output.stdout).unwrap(),
      String::from_utf8(setup_output.stderr).unwrap()
    );
  }

  let workspace = read_workspace_file(tmpdir.path().join(".projects.gws")).unwrap();
  let result = test(tmpdir.path(), workspace);
  result.map_err(|e| Error::from(e))
}
