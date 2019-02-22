use std::collections::BTreeSet;
use std::path::Path;

use git2::Branch;
use git2::BranchType;
use git2::Commit;
use git2::Reference;
use git2::Repository;
use git2::Status;

use commands::error::Error;
use config::data::Project;


pub type RepositoryStatus = BTreeSet<BranchStatus>;

trait BranchMethods<'repo> {
  fn branch_name<'a>(&'a self) -> Result<&'a str, Error>;
  fn is_up_to_date_with_upstream(&'repo self) -> Result<Option<bool>, Error>;
  fn upstream_name(&self) -> Result<Option<String>, Error>;
}

pub trait ProjectStatusMethods {
  fn status(&self, working_dir: &Path) -> Result<RepositoryStatus, Error>;
}

trait RepositoryMethods {
  fn any_file(&self, pred: fn(&Status) -> bool) -> bool;
  fn is_head(&self, branch: &Branch) -> Result<bool, Error>;
  fn project_status(&self, project: &Project) -> Result<RepositoryStatus, Error>;
}

trait StatusMethods {
  fn is_dirty(&self) -> bool;
  fn is_modified(&self) -> bool;
  fn is_untracked(&self) -> bool;
}

impl <'repo> BranchMethods<'repo> for Branch<'repo> {
  fn branch_name<'a>(&'a self) -> Result<&'a str, Error> {
    self.name()
      .map_err(Error::from)
      .and_then(|name|
        name.ok_or(Error::NoBranchNameFound)
      )
  }

  fn is_up_to_date_with_upstream(&'repo self) -> Result<Option<bool>, Error> {
    let branch_commit: Commit<'repo> = try!(self.get().peel_to_commit());

    let upstream: Option<Branch<'repo>> = self.upstream().ok();

    match upstream {
      None => Ok(None),
      Some(ups) => {
        let upstream_commit: Commit<'repo> = try!(ups.get().peel_to_commit());
        Ok(Some(branch_commit.id() == upstream_commit.id()))
      },
    }
  }

  fn upstream_name(&self) -> Result<Option<String>, Error> {
    Ok(
      self.upstream()?
      .name()?
      .map(&str::to_string)
    )
  }

}

impl ProjectStatusMethods for Project {
  fn status(&self, working_dir: &Path) -> Result<RepositoryStatus, Error> {
    self.open_repository(working_dir)?
      .project_status(&self)
  }
}

impl RepositoryMethods for Repository {

  fn any_file(&self, pred: fn(&Status) -> bool) -> bool {
    self.statuses(None)
      .iter()
      .flat_map(|ss| ss.iter())
      .map(|s| s.status())
      .any(|s| pred(&s))
  }

  fn is_head(&self, branch: &Branch) -> Result<bool, Error> {
    let head: Reference = try!(self.head());
    let br: &Reference = branch.get();
    Ok(head.name() == br.name())
  }

  fn project_status(&self, _project: &Project) -> Result<RepositoryStatus, Error> {
    let dirty_status =
      if self.any_file(StatusMethods::is_dirty) {
        if self.any_file(StatusMethods::is_modified) {
          DirtyState::UncommittedChanges
        } else {
          DirtyState::UntrackedFiles
        }
      } else {
        DirtyState::Clean
      };

    let branch_stati: Vec<BranchStatus> = self.branches(Some(BranchType::Local))
      .unwrap()
      .map(Result::unwrap)
      .map(|(b, _)| {
        let b_name = b.branch_name().unwrap();

        let is_head_branch = self.is_head(&b).unwrap();
        let is_in_sync = b.is_up_to_date_with_upstream().unwrap();

        BranchStatus {
          name: b_name.to_string(),
          upstream_name: match b.upstream_name() {
            Ok(Some(s)) => Some(s.to_string()),
            _ => None,
          },
          dirty: dirty_status.clone(),
          is_head: is_head_branch,
          in_sync: is_in_sync,
          upstream_fetched: false,
        }
      })
      .collect()
    ;

    Ok(branch_stati.into_iter().collect())
  }

}

impl StatusMethods for Status {
  fn is_dirty(&self) -> bool {
    let mut acceptable_flags = Status::CURRENT;
    acceptable_flags.insert(Status::IGNORED);
    let acceptable_flags = acceptable_flags;

    acceptable_flags.bits() & self.bits() != self.bits()
  }

  fn is_modified(&self) -> bool {
    self.is_dirty() && !self.is_untracked()
  }

  fn is_untracked(&self) -> bool {
    self.contains(Status::WT_NEW)
  }
}

#[derive(Debug)]
#[derive(Eq)]
#[derive(Ord)]
pub struct BranchStatus {
  pub name: String,
  pub upstream_name: Option<String>,
  pub dirty: DirtyState,
  pub is_head: bool,
  pub in_sync: Option<bool>,
  pub upstream_fetched: bool,
}

impl PartialEq for BranchStatus {
  fn eq(&self, rhs: &Self) -> bool {
    self.name == rhs.name
  }
}

impl PartialOrd for BranchStatus {
  fn partial_cmp(&self, rhs: &Self) -> Option<::std::cmp::Ordering> {
    self.name.partial_cmp(&rhs.name)
  }
}

#[derive(Clone)]
#[derive(Debug)]
#[derive(Eq)]
#[derive(PartialEq)]
#[derive(Ord)]
#[derive(PartialOrd)]
pub enum DirtyState {
  Clean,
  UncommittedChanges,
  UntrackedFiles,
}
