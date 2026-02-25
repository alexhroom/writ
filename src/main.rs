use std::path::PathBuf;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

use clap::Parser;
use krilla::Document;
use krilla::page::PageSettings;
use krilla::text::{Font, TextDirection};
use krilla::geom::Point;

mod config;
use config::*;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The file to read
    #[arg(short, long)]
    input: String,
    #[arg(short, long)]
    output: String,
}


fn main() -> io::Result<()> {
    let args = Args::parse();

    let file = File::open(args.input)?;
    let reader = BufReader::new(file);

    let font = {
        let path = PathBuf::from(FONT_PATH);
        let data = std::fs::read(&path).unwrap();
        Font::new(data.into(), 0).unwrap()
    };

    let mut doc = Document::new();
    let mut page = doc.start_page_with(PageSettings::from_wh(PAGE_DIM.w, PAGE_DIM.h).unwrap());
    let mut surface = page.surface();
    surface.set_fill(Some(FG.into()));

    let mut current_pt: (f32, f32) = (PAGE_DIM.w * MARGIN * (9./16.), PAGE_DIM.h * MARGIN);

    let mut page_empty = true; 

    for line in reader.lines().map_while(Result::ok) {
        match line {
            l if l.is_empty() => {
                // create new page
                surface.finish();
                page.finish();
                page = doc.start_page_with(PageSettings::from_wh(PAGE_DIM.w, PAGE_DIM.h).unwrap());
                surface = page.surface();
                surface.set_fill(Some(FG.into()));
                page_empty = true;
                current_pt = (PAGE_DIM.w * MARGIN * (9./16.), PAGE_DIM.h * MARGIN);
            }
            l if l.starts_with("#") => {
                // comment, ignore
                continue;
            }
            l if l.starts_with("@") => {
                // image 
            }
            l if l.starts_with("$") => {
                // math
            }
            l if l.starts_with("`") => {
                // code
            }
            _ => {
                // anything else is text
                if page_empty {
                    // start text block  
                } else {
                    // add to current text block
                    current_pt.1 += FONT_SIZE as f32 * 1.2;
                }
                page_empty = false;
                surface.draw_text(
                    Point::from_xy(current_pt.0, current_pt.1),
                    font.clone(),
                    FONT_SIZE,
                    &line,
                    false,
                    TextDirection::Auto
                );
            }
        }
    }
    surface.finish();
    page.finish();
    let pdf = doc.finish().unwrap();
    let path = std::path::Path::new(&args.output);

    // Write the PDF to a file.
    std::fs::write(path, &pdf).unwrap();
    
    eprintln!("Saved PDF to '{}'", path.display());

    Ok(())
}
