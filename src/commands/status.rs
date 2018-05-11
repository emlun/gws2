use std::path::Path;
use std::collections::BTreeSet;

extern crate core;

use ansi_term::ANSIString;
use ansi_term::Colour;
use ansi_term::Style;
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

struct Palette {
    branch: Style,
    clean: Style,
    dirty: Style,
    error: Style,
    missing: Style,
    repo: Style,
}

impl BranchStatus {
    fn describe_sync_status(&self, palette: &Palette) -> ANSIString {
        match self.in_sync {
            Some(true) => palette.clean.paint("Clean".to_string()),
            Some(false) => palette.dirty.paint(format!("Not in sync with {}", self.upstream_name)),
            None => palette.missing.paint(format!("No remote branch {}", self.upstream_name)),
        }
    }

    fn describe_status(&self, palette: &Palette) -> ANSIString {
        if self.is_head {
            match self.dirty {
                DirtyState::Clean => self.describe_sync_status(palette),
                DirtyState::UncommittedChanges => palette.dirty.paint("Dirty (Uncommitted changes)".to_string()),
                DirtyState::UntrackedFiles => palette.dirty.paint("Dirty (Untracked files)".to_string()),
            }
        } else {
            self.describe_sync_status(palette)
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

    let palette = Palette {
        branch: Colour::Fixed(13).normal(),
        clean: Colour::Fixed(10).normal(),
        dirty: Colour::Fixed(9).normal(),
        error: Colour::Fixed(9).normal(),
        missing: Colour::Fixed(11).normal(),
        repo: Colour::Fixed(12).normal(),
    };

    for project in ws.projects {
        println!("{}:", palette.repo.paint(project.path.clone()));

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
                        "  {} {} {}",
                        if b.is_head { "*" } else { " " },
                        palette.branch.paint(format!("{: <25}", format!("{} :", ellipsisize(&b.name, 23)))),
                        b.describe_status(&palette)
                    );
                }
            },
            Err(err) => {
                match err.code() {
                    ::git2::ErrorCode::NotFound => println!("{}", palette.missing.paint(format!("    {: <25 } {}", "", "Missing repository"))),
                    _ => println!("{}", palette.error.paint(format!("Failed to open repo: {}", err))),
                }
            }
        }

    }

    Ok(())
}
