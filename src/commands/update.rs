use std::path::Path;

use super::common::Command;
use super::error::Error;
use color::palette::Palette;
use config::data::Workspace;

pub struct Update {}

impl Command for Update {
    fn run(
        &self,
        working_dir: &Path,
        workspace: &Workspace,
        palette: &Palette,
    ) -> Result<i32, Error> {
        super::clone::Clone {
            projects: workspace.projects.iter().map(|p| p.path.clone()).collect(),
        }
        .run(working_dir, workspace, palette)
    }
}
