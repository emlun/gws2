extern crate ansi_term;
extern crate gws;

use ansi_term::Colour;

use gws::color::palette::Palette;

#[test]
fn default_palette_is_correct() {
    let expected = Palette {
        branch: Colour::Fixed(13).normal(),
        clean: Colour::Fixed(10).normal(),
        cloning: Colour::Fixed(14).normal(),
        dirty: Colour::Fixed(9).normal(),
        error: Colour::Fixed(9).normal(),
        missing: Colour::Fixed(11).normal(),
        repo: Colour::Fixed(12).normal(),
        repo_exists: Colour::Fixed(10).normal(),
    };

    assert_eq!(Palette::default(), expected);
}
