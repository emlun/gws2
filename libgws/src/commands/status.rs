use std::collections::BTreeSet;

extern crate core;

use git2::Branch;
use git2::Commit;
use git2::BranchType;
use git2::Reference;
use git2::Repository;
use git2::Status;

use config::data::Project;


fn upstream_name(branch: &Branch) -> Result<Option<String>, ::git2::Error> {
    branch.upstream()
        .and_then(|ups| ups.name().map(|ups| ups.map(&str::to_string)))
}

fn is_up_to_date_with_upstream<'repo>(branch: &'repo Branch<'repo>) -> Result<Option<bool>, ::git2::Error> {
    let branch_commit: Commit<'repo> = try!(branch.get().peel_to_commit());

    let upstream: Option<Branch<'repo>> = branch.upstream().ok();

    match upstream {
        None => Ok(None),
        Some(ups) => {
            let upstream_commit: Commit<'repo> = try!(ups.get().peel_to_commit());
            Ok(Some(branch_commit.id() == upstream_commit.id()))
        },
    }
}

fn is_dirty(status: &Status) -> bool {
    let mut acceptable_flags = Status::CURRENT;
    acceptable_flags.insert(Status::IGNORED);
    let acceptable_flags = acceptable_flags;

    acceptable_flags.bits() & status.bits() != status.bits()
}

fn is_untracked(status: &Status) -> bool {
    status.contains(Status::WT_NEW)
}

fn is_modified(status: &Status) -> bool {
    is_dirty(status) && !is_untracked(&status)
}

fn any_file(repo: &Repository, pred: fn(&Status) -> bool) -> bool {
    repo.statuses(None)
        .iter()
        .flat_map(|ss| ss.iter())
        .map(|s| s.status())
        .any(|s| pred(&s))
}

fn branch_name<'a>(branch: &'a Branch) -> Result<&'a str, ::git2::Error> {
    branch.name()
        .and_then(|name|
            name.ok_or(::git2::Error::from_str("No branch name found"))
        )
}

fn is_head(repo: &Repository, branch: &Branch) -> Result<bool, ::git2::Error> {
    let head: Reference = try!(repo.head());
    let br: &Reference = branch.get();
    Ok(head.name() == br.name())
}

#[derive(Eq)]
#[derive(Ord)]
pub struct BranchStatus {
    pub name: String,
    pub upstream_name: String,
    pub dirty: DirtyState,
    pub is_head: bool,
    pub in_sync: Option<bool>,
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
#[derive(Eq)]
#[derive(PartialEq)]
#[derive(Ord)]
#[derive(PartialOrd)]
pub enum DirtyState {
    Clean,
    UncommittedChanges,
    UntrackedFiles,
}

type RepositoryStatus = BTreeSet<BranchStatus>;

pub fn repo_status(project: &Project, repo: &Repository) -> Result<RepositoryStatus, ::git2::Error> {
    let dirty_status =
        if any_file(&repo, is_dirty) {
            if any_file(&repo, is_modified) {
                DirtyState::UncommittedChanges
            } else {
                DirtyState::UntrackedFiles
            }
        } else {
            DirtyState::Clean
        };

    let branch_stati: Vec<BranchStatus> = repo.branches(None)
        .unwrap()
        .map(Result::unwrap)
        .filter(|&(_, bt)| bt == BranchType::Local)
        .map(|(b, _)| {
            let b_name = branch_name(&b).unwrap();

            let is_head_branch = is_head(&repo, &b).unwrap();
            let is_in_sync = is_up_to_date_with_upstream(&b).unwrap();

            BranchStatus {
                name: b_name.to_string(),
                upstream_name: upstream_name(&b)
                    .ok()
                    .and_then(|s| s)
                    .map(|s| s.to_string())
                    .unwrap_or(format!("{}/{}", project.remotes[0].name, b_name))
                ,
                dirty: dirty_status.clone(),
                is_head: is_head_branch,
                in_sync: is_in_sync,
            }
        })
        .collect()
    ;

    Ok(branch_stati.into_iter().collect())
}

