use std::path:: Path;

use git2::Repository;

use super::Remote;


#[derive(Debug)]
#[derive(Eq)]
#[derive(Hash)]
#[derive(PartialEq)]
pub struct Project {
  pub path: String,
  pub main_remote: Remote,
  pub extra_remotes: Vec<Remote>,
}

impl Project {

  pub fn open_repository<P: AsRef<Path>>(&self, working_dir: P) -> Option<Result<Repository, ::git2::Error>> {
    let repo_dir = working_dir.as_ref().join(&self.path);
    if repo_dir.exists() {
      Some(Repository::open(repo_dir))
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
