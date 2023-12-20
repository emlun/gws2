use ansi_term::ANSIGenericString;
use glob::Pattern;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::path::Path;
use term_size;
use walkdir::WalkDir;

use super::common::exit_codes;
use super::common::format_message_line;
use super::common::DirectoryCommand;
use super::error::Error;
use crate::color::palette::Palette;
use crate::config::data::Workspace;

pub struct Check {}

impl DirectoryCommand for Check {
    fn run(
        &self,
        working_dir: &Path,
        workspace: &Workspace,
        palette: &Palette,
    ) -> Result<i32, Error> {
        let ignored_dirs = read_ignore_file(working_dir);
        let projects = check_projects(working_dir, workspace, &ignored_dirs);
        let (width, _) = term_size::dimensions().unwrap();

        for project in projects.iter() {
            let name = palette.repo.paint(&project.name);
            let path = palette.info.paint(format!("({})", &project.path));
            let padding = " ".repeat(width.saturating_sub(name.len() + path.len() + 2)); // +2 for the colon and space
            println!("{}: {}{}", name, padding, path);
            println!("{}", project.status.to_ansi(palette));
        }

        Ok(exit_codes::OK)
    }
}

struct CheckProject {
    name: String,
    path: String,
    status: ProjectStatus,
}

#[derive(Debug)]
enum ProjectStatus {
    Known,
    Missing,
    Unknown,
    Ignored,
}

impl ProjectStatus {
    fn to_ansi(&self, palette: &Palette) -> ANSIGenericString<str> {
        let missing = palette.missing;
        let clean = palette.clean;
        let error = palette.error;

        match self {
            ProjectStatus::Known => clean.paint(format_message_line(&self.to_string())),
            ProjectStatus::Missing => missing.paint(format_message_line(&self.to_string())),
            ProjectStatus::Unknown => error.paint(format_message_line(&self.to_string())),
            ProjectStatus::Ignored => missing.paint(format_message_line(&self.to_string())),
        }
    }
    fn to_string(&self) -> String {
        match self {
            ProjectStatus::Known => "Known",
            ProjectStatus::Missing => "Missing",
            ProjectStatus::Unknown => "Unknown",
            ProjectStatus::Ignored => "Ignored",
        }
        .to_string()
    }
}

fn read_ignore_file(dir: &Path) -> Vec<Pattern> {
    let file = File::open(dir.join(".ignore.gws")).unwrap();
    let reader = BufReader::new(file);
    reader
        .lines()
        .filter_map(Result::ok)
        .filter(|line| !line.starts_with("#"))
        .filter_map(|line| {
            let trimmed_line = line.trim();

            if trimmed_line.ends_with('/') {
                Pattern::new(&format!("{}**", trimmed_line)).ok()
            } else {
                Pattern::new(trimmed_line).ok()
            }
        })
        .collect()
}

fn check_projects(dir: &Path, workspace: &Workspace, ignored: &Vec<Pattern>) -> Vec<CheckProject> {
    let mut projects: Vec<CheckProject> = WalkDir::new(dir)
        .follow_links(true)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_dir())
        .filter(|e| e.path().join(".git").exists())
        .map(|e| {
            let path = e.path().to_str().unwrap().to_string();
            let name = path.replace("./", "");
            let status = if ignored.iter().any(|pattern| pattern.matches(&name)) {
                ProjectStatus::Ignored
            } else if workspace.projects.iter().any(|p| p.path == name) {
                ProjectStatus::Known
            } else {
                ProjectStatus::Unknown
            };
            CheckProject { name, path, status }
        })
        .collect();

    for project in &workspace.projects {
        let file_path = Path::new(&project.path);
        if !file_path.exists() {
            projects.push(CheckProject {
                name: project.path.clone(),
                path: file_path.to_str().unwrap().to_string(),
                status: ProjectStatus::Missing,
            });
        }
    }

    projects.sort_by(|a, b| a.name.cmp(&b.name));

    projects
}

#[cfg(test)]
mod test_glob {
    use glob::Pattern;

    #[test]
    fn test_glob() {
        let glob = Pattern::new("nextjs-blog/public/downloads/code/**").unwrap();

        assert_eq!(
            glob.matches("nextjs-blog/public/downloads/code/es-cluster-traefik"),
            true
        );
        assert_eq!(
            glob.matches("nextjs-blog/public/downloads/code/go-enum-tutorial"),
            true
        );
    }
}
