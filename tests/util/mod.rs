extern crate git2;
extern crate gws;
extern crate tempdir;

use std::fs::create_dir_all;
use std::fs::write;
use std::path::Path;
use std::path::PathBuf;

use tempdir::TempDir;

use gws::config::data::Workspace;
use gws::config::read::read_workspace_file;

#[derive(Debug)]
pub enum Error {
    IoError(::std::io::Error),
    Git2Error(::git2::Error),
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

fn join_all<I, P>(path: &Path, segments: I) -> PathBuf
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    let mut pb = path.to_path_buf();
    for segment in segments {
        pb.push(segment);
    }
    pb
}

fn add_commit_to_repo(repo: &git2::Repository, msg: &str) -> Result<git2::Oid, Error> {
    let commit = repo.head()?.peel_to_commit()?;
    let tree = repo.find_tree(commit.tree_id())?;
    let sig = repo.signature()?;
    Ok(repo.commit(Some("HEAD"), &sig, &sig, msg, &tree, &[&commit])?)
}

pub fn make_example_workspace(meta_dir: &Path, workspace_dir: &Path) -> Result<(), Error> {
    let origin_path: &Path = &meta_dir.join("origin");
    let ahead_path: &Path = &meta_dir.join("ahead");

    make_origin_repo(origin_path)?;
    let ahead_repo = git2::Repository::clone(origin_path.to_str().unwrap(), ahead_path)?;
    add_commit_to_repo(&ahead_repo, "More upstream work")?;

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

    make_project_no_upstream(workspace_dir.join("no_upstream").as_path(), origin_path)?;

    make_project_new_commit_local(
        &join_all(workspace_dir, &["new_commit", "local"]),
        origin_path,
        ahead_path,
    )?;

    make_project_new_commit_remote(
        &join_all(workspace_dir, &["new_commit", "remote"]),
        origin_path,
        ahead_path,
    )?;

    make_project_new_commit_unfetched_remote(
        &join_all(workspace_dir, &["new_commit", "unfetched_remote"]),
        origin_path,
        ahead_path,
    )?;

    make_project_new_commit_diverged(
        &join_all(workspace_dir, &["new_commit", "diverged"]),
        ahead_path,
    )?;

    make_project_new_files(
        &join_all(workspace_dir, &["changes", "new_files"]),
        origin_path,
        ahead_path,
    )?;

    make_project_changed_files(
        &join_all(workspace_dir, &["changes", "changed_files"]),
        origin_path,
        ahead_path,
    )?;

    Ok(())
}

fn add_remote<'repo>(
    name: &str,
    repo: &'repo git2::Repository,
    remote_path: &Path,
) -> Result<git2::Remote<'repo>, Error> {
    let mut remote = repo.remote(name, remote_path.to_str().unwrap())?;
    remote.fetch(&["master"], None, None)?;
    Ok(remote)
}

fn add_ahead_remote<'repo>(
    repo: &'repo git2::Repository,
    remote_path: &Path,
) -> Result<git2::Remote<'repo>, Error> {
    add_remote("ahead", repo, remote_path)
}

fn add_master2_branch<'repo>(
    repo: &'repo git2::Repository,
    target_ref: &str,
    target_type: git2::BranchType,
) -> Result<git2::Branch<'repo>, Error> {
    add_master2_branch_with_upstream(repo, target_ref, target_type, target_ref)
}

fn add_master2_branch_with_upstream<'repo>(
    repo: &'repo git2::Repository,
    target_ref: &str,
    target_type: git2::BranchType,
    upstream: &str,
) -> Result<git2::Branch<'repo>, Error> {
    let target_commit = repo
        .find_branch(target_ref, target_type)?
        .get()
        .peel_to_commit()?;
    let mut master2 = repo.branch("master2", &target_commit, false)?;
    master2.set_upstream(Some(upstream))?;
    Ok(master2)
}

fn add_default_master2_branch<'repo>(
    repo: &'repo git2::Repository,
) -> Result<git2::Branch<'repo>, Error> {
    add_master2_branch(repo, "ahead/master", git2::BranchType::Remote)
}

fn make_project_clean(
    path: &Path,
    origin_path: &Path,
    ahead_path: &Path,
) -> Result<git2::Repository, Error> {
    let repo = git2::Repository::clone(origin_path.to_str().unwrap(), path)?;
    add_ahead_remote(&repo, ahead_path)?;

    {
        let target_commit = repo
            .find_branch("master", git2::BranchType::Local)?
            .get()
            .peel_to_commit()?;
        repo.branch("feature", &target_commit, false)?;
    }

    Ok(repo)
}

fn make_project_no_upstream(path: &Path, origin_path: &Path) -> Result<git2::Repository, Error> {
    let repo = git2::Repository::clone(origin_path.to_str().unwrap(), path)?;

    repo.find_branch("master", git2::BranchType::Local)?
        .set_upstream(None)?;

    Ok(repo)
}

fn make_project_new_commit_local(
    path: &Path,
    origin_path: &Path,
    ahead_path: &Path,
) -> Result<git2::Repository, Error> {
    let repo = git2::Repository::clone(origin_path.to_str().unwrap(), path)?;
    add_commit_to_repo(&repo, "More local work")?;
    add_ahead_remote(&repo, ahead_path)?;
    add_master2_branch(&repo, "ahead/master", git2::BranchType::Remote)?;
    Ok(repo)
}

fn make_project_new_commit_remote(
    path: &Path,
    origin_path: &Path,
    ahead_path: &Path,
) -> Result<git2::Repository, Error> {
    let repo = git2::Repository::clone(origin_path.to_str().unwrap(), path)?;
    add_ahead_remote(&repo, ahead_path)?;
    add_master2_branch_with_upstream(&repo, "master", git2::BranchType::Local, "ahead/master")?;
    Ok(repo)
}

fn make_project_new_commit_unfetched_remote(
    path: &Path,
    origin_path: &Path,
    ahead_path: &Path,
) -> Result<git2::Repository, Error> {
    let repo = make_project_new_commit_remote(path, origin_path, origin_path)?;
    repo.remote_set_url("ahead", ahead_path.to_str().unwrap())?;
    Ok(repo)
}

fn make_project_new_commit_diverged(
    path: &Path,
    ahead_path: &Path,
) -> Result<git2::Repository, Error> {
    let repo = git2::Repository::clone(ahead_path.to_str().unwrap(), path)?;

    {
        let master = repo.find_branch("master", git2::BranchType::Local)?;
        let target_commit = master.get().peel_to_commit()?.parent(0)?;
        master
            .into_reference()
            .set_target(target_commit.id(), "Prepare for divergence")?;
    }

    add_commit_to_repo(&repo, "More local work")?;

    Ok(repo)
}

fn make_project_new_files(
    path: &Path,
    origin_path: &Path,
    ahead_path: &Path,
) -> Result<git2::Repository, Error> {
    let repo = git2::Repository::clone(origin_path.to_str().unwrap(), path)?;
    write(path.join("foo.txt").as_path(), &[])?;
    add_ahead_remote(&repo, ahead_path)?;
    add_default_master2_branch(&repo)?;
    Ok(repo)
}

fn make_project_changed_files(
    path: &Path,
    origin_path: &Path,
    ahead_path: &Path,
) -> Result<git2::Repository, Error> {
    let repo = git2::Repository::clone(origin_path.to_str().unwrap(), path)?;
    write(path.join("README.md").as_path(), "flrglgrgldrgl\n")?;
    add_ahead_remote(&repo, ahead_path)?;
    add_default_master2_branch(&repo)?;
    Ok(repo)
}

fn write_test_projects_file(
    path: &Path,
    origin_path: &Path,
    ahead_path: &Path,
) -> Result<(), Error> {
    let content = format!(
        "clean                       | {origin} | {ahead} ahead
no_upstream                 | {origin}
new_commit/diverged         | {origin} | {ahead} ahead
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

pub fn in_example_workspace<T>(test: fn(&Path, Workspace) -> Result<T, Error>) {
    let result = in_example_workspace_inner(test);
    assert!(result.is_ok(), format!("{:?}", result.err()));
}

fn in_example_workspace_inner<T, E>(test: fn(&Path, Workspace) -> Result<T, E>) -> Result<T, Error>
where
    Error: From<E>,
{
    let tmpdir = TempDir::new("gws-test")?;
    let meta_dir = tmpdir.path().join("meta");
    let workspace_dir = tmpdir.path().join("workspace");

    make_example_workspace(&meta_dir, &workspace_dir)?;

    let workspace = read_workspace_file(workspace_dir.join(".projects.gws")).unwrap();
    let result = test(&workspace_dir, workspace);

    result.map_err(|e| Error::from(e))
}
