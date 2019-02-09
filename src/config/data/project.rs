use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::path::Path;

use super::Branch;
use super::Remote;


#[derive(Debug)]
#[derive(Eq)]
#[derive(Hash)]
#[derive(Ord)]
#[derive(PartialEq)]
#[derive(PartialOrd)]
pub struct Project {
  pub path: String,
  pub main_remote: Remote,
  pub extra_remotes: Vec<Remote>,
}

impl Project {

  pub fn open_repository<P: AsRef<Path>>(&self, working_dir: P) -> Option<Result<git2::Repository, ::git2::Error>> {
    let repo_dir = working_dir.as_ref().join(&self.path);
    if repo_dir.exists() {
      Some(git2::Repository::open(repo_dir))
    } else {
      None
    }
  }

  pub fn remotes(&self) -> Vec<&Remote> {
    let mut extras: Vec<&Remote> = self.extra_remotes.iter().collect();
    let mut result = vec![&self.main_remote];
    result.append(&mut extras);
    result
  }

  fn local_branches_internal<'repo>(&self, repo: &'repo git2::Repository) -> Result<Vec<(Branch, git2::Branch<'repo>, Option<git2::Branch<'repo>>)>, git2::Error> {
    Ok(
      repo
        .branches(Some(git2::BranchType::Local))?
        .flatten()
        .map(|(branch, _)| {
          let upstream_name = branch
            .upstream()
            .map(|upstream| upstream.name().ok().unwrap_or(None).map(String::from))
            .ok();
          let upstream: Option<git2::Branch<'repo>> = branch.upstream().ok();
          (
            Branch {
              name: branch.name().ok().unwrap_or(None).map(String::from),
              upstream_name: upstream_name,
            },
            branch,
            upstream,
          )
        })
        .collect()
    )
  }

  pub fn local_branches(&self, repo: &git2::Repository) -> Result<BTreeSet<Branch>, git2::Error> {
    self.local_branches_internal(repo)
      .map(|branches|
           branches
           .into_iter()
           .map(|(b, _, _)| b)
           .collect()
      )
  }

  pub fn current_heads(&self, repo: &git2::Repository) -> Result<BTreeMap<Branch, git2::Oid>, git2::Error> {
    self.local_branches_internal(repo)
      .map(|branches|
           branches
           .into_iter()
           .map(|(branch, gbranch, _)|
                (branch, gbranch.get().peel_to_commit().unwrap().id()),
           )
           .collect()
      )
  }

  pub fn current_upstream_heads(&self, repo: &git2::Repository) -> Result<BTreeMap<Branch, git2::Oid>, git2::Error> {
    self.local_branches_internal(repo)
      .map(|branches|
           branches
           .into_iter()
           .flat_map(|(branch, _, gupstream)|
                     gupstream.map(|gupstream| (branch, gupstream))
           )
           .map(|(branch, gupstream)|
                (branch, gupstream.get().peel_to_commit().unwrap().id()),
           )
           .collect()
      )
  }
}

#[cfg(test)]
mod tests {
  use super::Project;
  use super::Remote;

  #[test]
  fn remotes_returns_main_remote_and_then_extras() {
    assert_eq!(
      Project {
        path: "foo".to_string(),
        main_remote: Remote {
          name: "origin".to_string(),
          url: "git@github.com:foo/boo.git".to_string(),
        },
        extra_remotes: vec![
          Remote {
            name: "myone".to_string(),
            url: "http://coool".to_string(),
          },
          Remote {
            name: "upstream".to_string(),
            url: "testurl".to_string(),
          },
        ],
      }.remotes(),
      vec![
        Remote {
          name: "origin".to_string(),
          url: "git@github.com:foo/boo.git".to_string(),
        },
        Remote {
          name: "myone".to_string(),
          url: "http://coool".to_string(),
        },
        Remote {
          name: "upstream".to_string(),
          url: "testurl".to_string(),
        },
      ].iter().collect::<Vec<&Remote>>()
    );
  }
}
