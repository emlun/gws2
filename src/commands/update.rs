use std::path::Path;

use super::common::DirectoryCommand;
use super::error::Error;
use crate::color::palette::Palette;
use crate::config::data::Workspace;

pub struct Update {}

impl DirectoryCommand for Update {
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
