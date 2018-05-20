pub mod exit_codes;

use ansi_term::ANSIString;

use color::palette::Palette;
use config::data::Project;
use config::data::Workspace;


pub trait Command {
    fn run(&self, workspace: Workspace, palette: &Palette) -> Result<i32, ::git2::Error>;
}

fn ellipsisize(s: &str, length: usize) -> String {
    if s.len() >= length {
        format!("{}…", &s[0..(length - 1)])
    } else {
        s.to_string()
    }
}

pub fn format_branch_line(palette: &Palette, is_head: bool, name: &str, description: &ANSIString) -> String {
    format!(
        "  {} {} {}",
        if is_head { "*" } else { " " },
        palette.branch.paint(format!("{: <25}", format!("{} :", ellipsisize(name, 23)))),
        description
    )
}

pub fn format_message_line(message: &str) -> String {
    format!("{: <30 }{}", "", message)
}

pub fn format_project_header(project: &Project, palette: &Palette) -> String {
    format!("{}:", palette.repo.paint(project.path.clone()))
}
