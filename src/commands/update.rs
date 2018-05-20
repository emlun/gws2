use color::palette::Palette;
use config::data::Workspace;
use super::common::Command;


pub struct Update {
}

impl Command for Update {
    fn run(&self, workspace: Workspace, palette: &Palette) -> Result<i32, ::git2::Error> {
        super::clone::Clone {
            projects: workspace.projects.iter().map(|p| p.path.clone()).collect(),
        }.run(workspace, palette)
    }
}
