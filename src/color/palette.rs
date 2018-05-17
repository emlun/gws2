use ::ansi_term::Colour;
use ::ansi_term::Style;


pub struct Palette {
    pub branch: Style,
    pub clean: Style,
    pub dirty: Style,
    pub error: Style,
    pub missing: Style,
    pub repo: Style,
}

impl Palette {
    pub fn default() -> Palette {
        Palette {
            branch: Colour::Fixed(13).normal(),
            clean: Colour::Fixed(10).normal(),
            dirty: Colour::Fixed(9).normal(),
            error: Colour::Fixed(9).normal(),
            missing: Colour::Fixed(11).normal(),
            repo: Colour::Fixed(12).normal(),
        }
    }
}
