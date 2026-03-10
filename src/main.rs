use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::{self, BufRead, BufReader};

use clap::Parser;
use krilla::{Document, Data};
use krilla::page::PageSettings;
use krilla::text::{Font, TextDirection};
use krilla::geom::{Point, Size, Transform};
use krilla::image::Image;

mod config;
use config::*;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The file to read
    #[arg()]
    input: String,
    #[arg(short, long)]
    output: Option<String>,
}

// consts for margins
const TOP_LEFT: (f32, f32) = (PAGE_DIM.w * MARGIN * (9./16.), PAGE_DIM.h * MARGIN);
const BOTTOM_RIGHT: (f32, f32) = (PAGE_DIM.w - TOP_LEFT.0, PAGE_DIM.h - TOP_LEFT.1);


fn main() -> io::Result<()> {
    let args = Args::parse();

    let file = File::open(&args.input)?;
    let reader = BufReader::new(file);

    let font = {
        let path = PathBuf::from(FONT_PATH);
        let data = std::fs::read(&path).unwrap();
        Font::new(data.into(), 0).unwrap()
    };

    let code_font = {
        let path = PathBuf::from(CODE_FONT_PATH);
        let data = std::fs::read(&path).unwrap();
        Font::new(data.into(), 0).unwrap()
    };

    let mut doc = Document::new();
    let mut page = doc.start_page_with(PageSettings::from_wh(PAGE_DIM.w, PAGE_DIM.h).unwrap());
    let mut surface = page.surface();
    surface.set_fill(Some(FG.into()));

    let mut current_pt: (f32, f32) = TOP_LEFT;
    let mut image: Image;

    let mut page_empty = true;
    let mut image_centre: (f32, f32);

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
                current_pt = TOP_LEFT;
            }
            l if l.starts_with("#") => {
                // comment, ignore
                continue;
            }
            l if l.starts_with("@") => {
                // image
      
                image = get_image(&l[1..])?;
                let (width, height) = image.size();

                if !page_empty {
                    // put image in right side 
                    if width > height {
                    // if image is wider than it is tall, put in bottom 
                    image_centre = (PAGE_DIM.w * 0.5, PAGE_DIM.h * 0.7);
                    } else {
                    // if image is taller than it is wide, put in right
                    image_centre = (PAGE_DIM.w * 0.7, PAGE_DIM.h * 0.5);
                    }
                } else {
                    // put in centre of page
                    image_centre = (PAGE_DIM.w * 0.5, PAGE_DIM.h * 0.5)
                }
                let (transform, image_size) = get_image_centre_scaling(&image, image_centre);
                surface.push_transform(&transform);
                surface.draw_image(image, image_size);
                surface.pop();
            }
            l if l.starts_with("$") => {
                // math
            }
            ref l if l.starts_with("`") => {
                // code
                if !page_empty {
                    // add to current text block
                    current_pt.1 += FONT_SIZE as f32 * 1.2;
                }
                page_empty = false;
                surface.draw_text(
                    Point::from_xy(current_pt.0 + PAGE_DIM.w * MARGIN * (4.5 / 16.), 
                                   current_pt.1),
                    code_font.clone(),
                    FONT_SIZE,
                    &line[1..],
                    false,
                    TextDirection::Auto
                );
            }
            l if l == "/" => {
                // line break
                current_pt.1 += FONT_SIZE as f32 * 1.2;
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
    let output_path = args.output.unwrap_or_else(|| Path::new(&args.input).file_stem().unwrap().to_str().unwrap().to_string() + ".pdf");
    let path = std::path::Path::new(&output_path);

    // Write the PDF to a file.
    std::fs::write(path, &pdf).unwrap();
    
    eprintln!("Saved PDF to '{}'", path.display());

    Ok(())
}

// get an image from a file path, supporting png, jpg/jpeg, gif, and webp
fn get_image(filename: &str) -> Result<Image, std::io::Error> {
    let path = PathBuf::from(filename);
    let data: Data = std::fs::read(&path).unwrap().into();
    let image = match path.extension().and_then(|ext| ext.to_str()) {
        Some("png") => Image::from_png(data, true),
        Some("jpg") | Some("jpeg") => Image::from_jpeg(data, true),
        Some("gif") => Image::from_gif(data, true),
        Some("webp") => Image::from_webp(data, true),
        _ => Err(format!("Unsupported image format: {}", filename)), 
    };
    if image.is_err() {
        Err(std::io::Error::new(std::io::ErrorKind::InvalidData, image.err().unwrap()))
    } else {
        Ok(image.unwrap())
    }
}

// given an image and a centre point, return a transform that scales the image to fit within the
// page margins and is centred on the given point, as well as the size of the scaled image 
fn get_image_centre_scaling(image: &Image, centre: (f32, f32)) -> (Transform, Size) {
    let (width, height) = image.size();
    let scale: f32;
    let scaled_width: f32;
    let scaled_height: f32;
    let max_width = match centre.0 > PAGE_DIM.w * 0.5 {
        true => BOTTOM_RIGHT.0 - centre.0,
        false => centre.0 - TOP_LEFT.0,
    };
    let max_height = match centre.1 > PAGE_DIM.h * 0.5 {
        true => BOTTOM_RIGHT.1 - centre.1,
        false => centre.1 - TOP_LEFT.1,
    };
    if max_width < max_height {
        scaled_width = max_width * 2.0;
        scale = scaled_width / width as f32;
        scaled_height = height as f32 * scale;
    } else {
        scaled_height = max_height * 2.0;
        scale = scaled_height / height as f32;
        scaled_width = width as f32 * scale;
    } 
    let transform = Transform::from_translate(centre.0 as f32 - scaled_width / 2.0, centre.1 as f32 - scaled_height / 2.0);
    (transform, Size::from_wh(scaled_width, scaled_height).unwrap())
}
