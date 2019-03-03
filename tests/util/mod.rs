extern crate git2;
extern crate gws2;
extern crate tempdir;

use std::fs::create_dir_all;
use std::fs::write;
use std::path::Path;

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

fn make_origin_repo(path: &Path) -> Result<git2::Repository, Error> {
  let repo = git2::Repository::init(path)?;

  let mut repo_config = repo.config()?;
  repo_config.set_str("user.name", "Alice Hypothetical")?;
  repo_config.set_str("user.email", "alice@example.com")?;
  let mut repo_index = repo.index()?;

  let readme_path = Path::new("README.md");
  write(path.join(readme_path).as_path(), &[])?;
  repo_index.add_path(readme_path)?;

  let tree_id = repo_index.write_tree()?;

  {
    let tree = repo.find_tree(tree_id)?;
    let sig = repo.signature()?;
    repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])?;
  }

  Ok(repo)
}

fn add_commit_to_repo(repo: &git2::Repository) -> Result<git2::Oid, Error> {
  let commit = repo.head()?.peel_to_commit()?;
  let tree = repo.find_tree(commit.tree_id())?;
  let sig = repo.signature()?;
  Ok(repo.commit(Some("HEAD"), &sig, &sig, "More work", &tree, &[&commit])?)
}

pub fn make_example_workspace(meta_dir: &Path, workspace_dir: &Path) -> Result<(), Error> {
  let origin_path: &Path = &meta_dir.join("origin");
  let ahead_path: &Path = &meta_dir.join("ahead");

  make_origin_repo(origin_path)?;
  let ahead_repo = git2::Repository::clone(origin_path.to_str().unwrap(), ahead_path)?;
  add_commit_to_repo(&ahead_repo)?;

  create_dir_all(workspace_dir)?;

  write_test_projects_file(
    workspace_dir.join(".projects.gws").as_path(),
    origin_path,
    ahead_path,
  )?;

  make_project_clean(
    workspace_dir.join("clean").as_path(),
    origin_path,
    ahead_path,
  )?;

  {
    let mut pb = workspace_dir.join("new_commit");
    pb.push("local");
    make_project_new_commit_local(
      pb.as_path(),
      origin_path,
      ahead_path,
    )?;
  }

  {
    let mut pb = workspace_dir.join("new_commit");
    pb.push("remote");
    make_project_new_commit_remote(
      pb.as_path(),
      origin_path,
      ahead_path,
    )?;
  }

  {
    let mut pb = workspace_dir.join("new_commit");
    pb.push("unfetched_remote");
    make_project_new_commit_unfetched_remote(
      pb.as_path(),
      origin_path,
      ahead_path,
    )?;
  }

  {
    let mut pb = workspace_dir.join("changes");
    pb.push("new_files");
    make_project_new_files(
      pb.as_path(),
      origin_path,
      ahead_path,
    )?;
  }

  {
    let mut pb = workspace_dir.join("changes");
    pb.push("changed_files");
    make_project_changed_files(
      pb.as_path(),
      origin_path,
      ahead_path,
    )?;
  }

  Ok(())
}

fn make_project_clean(
  path: &Path,
  origin_path: &Path,
  ahead_path: &Path,
) -> Result<git2::Repository, Error> {
  let repo = git2::Repository::clone(origin_path.to_str().unwrap(), path)?;
  repo
    .remote("ahead", ahead_path.to_str().unwrap())?
    .fetch(&["master"], None, None)?;
  {
    let ahead_master_commit = repo
      .find_branch("ahead/master", git2::BranchType::Remote)?
      .get()
      .peel_to_commit()?;
    let mut master2 = repo.branch("master2", &ahead_master_commit, false)?;
    master2.set_upstream(Some("ahead/master"))?;
  }
  Ok(repo)
}

fn make_project_new_commit_local(
  path: &Path,
  origin_path: &Path,
  ahead_path: &Path,
) -> Result<git2::Repository, Error> {
  let repo = git2::Repository::clone(origin_path.to_str().unwrap(), path)?;
  add_commit_to_repo(&repo)?;
  repo
    .remote("ahead", ahead_path.to_str().unwrap())?
    .fetch(&["master"], None, None)?;

  {
    let master_commit = repo
      .find_branch("master", git2::BranchType::Local)?
      .get()
      .peel_to_commit()?;
    let mut master2 = repo.branch("master2", &master_commit, false)?;
    master2.set_upstream(Some("ahead/master"))?;
  }

  Ok(repo)
}

fn make_project_new_commit_remote(
  path: &Path,
  origin_path: &Path,
  ahead_path: &Path,
) -> Result<git2::Repository, Error> {
  let repo = git2::Repository::clone(origin_path.to_str().unwrap(), path)?;
  repo
    .remote("ahead", ahead_path.to_str().unwrap())?
    .fetch(&["master"], None, None)?;

  {
    let master_commit = repo
      .find_branch("master", git2::BranchType::Local)?
      .get()
      .peel_to_commit()?;
    let mut master2 = repo.branch("master2", &master_commit, false)?;
    master2.set_upstream(Some("ahead/master"))?;
  }

  Ok(repo)
}

fn make_project_new_commit_unfetched_remote(
  path: &Path,
  origin_path: &Path,
  ahead_path: &Path,
) -> Result<git2::Repository, Error> {
  let repo = git2::Repository::clone(origin_path.to_str().unwrap(), path)?;

  {
    let mut ahead_remote = repo.remote("ahead", origin_path.to_str().unwrap())?;
    ahead_remote.fetch(&["master"], None, None)?;

    let master_commit = repo
      .find_branch("master", git2::BranchType::Local)?
      .get()
      .peel_to_commit()?;
    let mut master2 = repo.branch("master2", &master_commit, false)?;
    master2.set_upstream(Some("ahead/master"))?;

    repo.remote_set_url("ahead", ahead_path.to_str().unwrap())?;
  }

  Ok(repo)
}

fn make_project_new_files(
  path: &Path,
  origin_path: &Path,
  ahead_path: &Path,
) -> Result<git2::Repository, Error> {
  let repo = git2::Repository::clone(origin_path.to_str().unwrap(), path)?;
  write(path.join("foo.txt").as_path(), &[])?;
  repo
    .remote("ahead", ahead_path.to_str().unwrap())?
    .fetch(&["master"], None, None)?;
  {
    let ahead_master_commit = repo
      .find_branch("ahead/master", git2::BranchType::Remote)?
      .get()
      .peel_to_commit()?;
    let mut master2 = repo.branch("master2", &ahead_master_commit, false)?;
    master2.set_upstream(Some("ahead/master"))?;
  }
  Ok(repo)
}

fn make_project_changed_files(
  path: &Path,
  origin_path: &Path,
  ahead_path: &Path,
) -> Result<git2::Repository, Error> {
  let repo = git2::Repository::clone(origin_path.to_str().unwrap(), path)?;
  write(path.join("README.md").as_path(), "flrglgrgldrgl\n")?;
  repo
    .remote("ahead", ahead_path.to_str().unwrap())?
    .fetch(&["master"], None, None)?;
  {
    let ahead_master_commit = repo
      .find_branch("ahead/master", git2::BranchType::Remote)?
      .get()
      .peel_to_commit()?;
    let mut master2 = repo.branch("master2", &ahead_master_commit, false)?;
    master2.set_upstream(Some("ahead/master"))?;
  }

  Ok(repo)
}

fn write_test_projects_file(
  path: &Path,
  origin_path: &Path,
  ahead_path: &Path,
) -> Result<(), Error> {
  let content = format!(
    "clean                       | {origin} | {ahead} ahead
new_commit/local            | {origin} | {ahead} ahead
new_commit/remote           | {origin} | {ahead} ahead
new_commit/unfetched_remote | {origin} | {ahead} ahead
changes/new_files           | {origin} | {ahead} ahead
changes/changed_files       | {origin} | {ahead} ahead
missing_repository          | {origin} | {ahead} ahead
missing_repository_2        | {origin} | {ahead} ahead
",
    origin = origin_path.to_str().unwrap(),
    ahead = ahead_path.to_str().unwrap(),
  );
  Ok(write(path, content)?)
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
  let meta_dir = tmpdir.path().join("meta");
  let workspace_dir = tmpdir.path().join("workspace");

  make_example_workspace(&meta_dir, &workspace_dir)?;

  let workspace = read_workspace_file(workspace_dir.join(".projects.gws")).unwrap();
  let result = test(&workspace_dir, workspace);

  result.map_err(|e| Error::from(e))
}
