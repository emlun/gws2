/// Workaround to silence false-positive warnings about dead code for tests that
/// don't use these functions
use std::fs::create_dir_all;
use std::path::Path;

use tempdir::TempDir;

use gws::config::data::Workspace;
use gws::config::read::read_workspace_file;

use super::util::write_projects_file;
use super::util::Error;

pub fn in_workspace_with_projects_file<T, F>(projects_contents: &str, test: F) -> Result<T, Error>
where
    F: Fn(&Path, Workspace) -> Result<T, Error>,
{
    let tmpdir = TempDir::new("gws-test")?;
    let workspace_dir = tmpdir.path().join("workspace");

    create_dir_all(&workspace_dir)?;
    write_projects_file(
        workspace_dir.join(".projects.gws").as_path(),
        projects_contents,
    )?;

    let workspace = read_workspace_file(workspace_dir.join(".projects.gws")).unwrap();
    Ok(test(&workspace_dir, workspace)?)
}
