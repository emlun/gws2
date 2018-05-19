use color::palette::Palette;
use config::data::Project;


pub fn print_project_header(project: &Project, palette: &Palette) -> () {
    println!("{}:", palette.repo.paint(project.path.clone()));
}
