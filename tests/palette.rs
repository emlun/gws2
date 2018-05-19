extern crate ansi_term;
extern crate gws2;

use ansi_term::Colour;

use gws2::color::palette::Palette;


#[test]
fn default_palette_is_correct() {
    let expected = Palette {
        branch: Colour::Fixed(13).normal(),
        clean: Colour::Fixed(10).normal(),
        dirty: Colour::Fixed(9).normal(),
        error: Colour::Fixed(9).normal(),
        missing: Colour::Fixed(11).normal(),
        repo: Colour::Fixed(12).normal(),
    };

    assert_eq!(Palette::default(), expected);
}