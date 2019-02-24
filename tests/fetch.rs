extern crate git2;
extern crate gws2;
extern crate tempdir;

mod util;

use std::collections::HashSet;
use std::hash::Hash;

use git2::BranchType;
use git2::Commit;
use git2::Repository;

use gws2::color::palette::Palette;
use gws2::commands::fetch::Fetch;
use gws2::commands::common::Command;
use gws2::commands::status::Status;
use gws2::config::data::Workspace;

use util::in_example_workspace;


pub fn hash_set<I, T>(items: I) -> HashSet<T>
  where I: IntoIterator<Item=T>,
        T: Eq + Hash,
{
  items.into_iter().collect()
}

#[test]
fn fetch_gets_refs_from_named_remotes() {
  fn resolve_ref<'repo>(name: &str, repo: &'repo Repository) -> Result<Commit<'repo>, ::git2::Error> {
    repo
      .find_branch(name, BranchType::Remote)?
      .get()
      .peel_to_commit()
  }

  in_example_workspace(|working_dir, workspace: Workspace| {
    let project_path = "new_commit/unfetched_remote";

    let command: Fetch = Fetch {
      status_command: Status { only_changes: false },
      projects: hash_set(vec!["new_commit/unfetched_remote".to_string()])
    };

    let repo: Repository = Repository::open(working_dir.join(project_path))?;

    let master_reference_before: Commit = resolve_ref("origin/master", &repo)?;
    let master2_reference_before: Commit = resolve_ref("remote2/master", &repo)?;

    command.run(working_dir, &workspace, &Palette::default())
      .expect("Fetch command failed");

    let master_reference_after: Commit = resolve_ref("origin/master", &repo)?;
    let master2_reference_after: Commit = resolve_ref("remote2/master", &repo)?;

    assert_eq!(master_reference_after.id(), master_reference_before.id());

    assert_ne!(master2_reference_after.id(), master2_reference_before.id());
    assert_eq!(master2_reference_after.parents().next().unwrap().id(), master2_reference_before.id());

    Ok(())
  });
}
