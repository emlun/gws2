use std::path::Path;
use std::collections::BTreeSet;

extern crate core;

use git2::Branch;
use git2::Commit;
use git2::BranchType;
use git2::Reference;
use git2::Repository;
use git2::Status;

use config::read::read_workspace_file;


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
struct BranchStatus {
    name: String,
    upstream_name: String,
    dirty: DirtyState,
    is_head: bool,
    in_sync: Option<bool>,
}

impl BranchStatus {
    fn describe_sync_status(&self) -> String {
        match self.in_sync {
            Some(true) => "Clean".to_string(),
            Some(false) => format!("Not in sync with {}", self.upstream_name),
            None => format!("No remote branch {}", self.upstream_name),
        }
    }

    fn describe_status(&self) -> String {
        if self.is_head {
            match self.dirty {
                DirtyState::Clean => self.describe_sync_status(),
                DirtyState::UncommittedChanges => "Dirty (Uncommitted changes)".to_string(),
                DirtyState::UntrackedFiles => "Dirty (Untracked files)".to_string(),
            }
        } else {
            self.describe_sync_status()
        }
    }
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
enum DirtyState {
    Clean,
    UncommittedChanges,
    UntrackedFiles,
}

type RepositoryStatus = BTreeSet<BranchStatus>;

fn repo_status(repo: &Repository) -> Result<RepositoryStatus, ::git2::Error> {
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
                    .unwrap_or(format!("origin/{}", b_name))
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

fn ellipsisize(s: &str, length: usize) -> String {
    if s.len() >= length {
        format!("{}…", &s[0..(length - 1)]).to_string()
    } else {
        s.to_string()
    }
}

pub fn run() -> Result<(), ::git2::Error> {

    let ws_file_path = Path::new(".projects.gws");
    let ws = read_workspace_file(ws_file_path).unwrap();

    for project in ws.projects {
        println!("{}:", project.path);

        match Repository::open(
            ws_file_path
                .parent()
                .unwrap()
                .join(project.path)
                .as_path()
            )
        {
            Ok(repo) => {
                for b in try!(repo_status(&repo)) {
                    println!(
                        "    {: <25} {}",
                        format!("{} :", ellipsisize(&b.name, 23)),
                        b.describe_status()
                    );
                }
            },
            Err(err) => {
                match err.code() {
                    ::git2::ErrorCode::NotFound => println!("    {: <25 } {}", "", "Missing repository"),
                    _ => println!("Failed to open repo: {}", err),
                }
            }
        }

    }

    Ok(())
}
