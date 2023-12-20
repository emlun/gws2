use ansi_term::Colour;
use ansi_term::Style;

#[derive(Debug, PartialEq)]
pub struct Palette {
    pub branch: Style,
    pub clean: Style,
    pub cloning: Style,
    pub dirty: Style,
    pub info: Style,
    pub error: Style,
    pub missing: Style,
    pub repo: Style,
    pub repo_exists: Style,
}

impl Palette {
    pub fn default() -> Palette {
        Palette {
            branch: Colour::Fixed(13).normal(),
            clean: Colour::Fixed(10).normal(),
            cloning: Colour::Fixed(14).normal(),
            dirty: Colour::Fixed(9).normal(),
            info: Colour::Fixed(242).normal(),
            error: Colour::Fixed(9).normal(),
            missing: Colour::Fixed(11).normal(),
            repo: Colour::Fixed(12).normal(),
            repo_exists: Colour::Fixed(10).normal(),
        }
    }
}
