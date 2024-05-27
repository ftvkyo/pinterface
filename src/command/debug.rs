use cosmic_text::Color;
use imageproc::rect::Rect;

use crate::{driver::{DisplayImage, BLACK}, render};


pub fn debug(img: &mut DisplayImage) -> Result<(), Box<dyn std::error::Error>> {
    let rect = Rect::at(0, 0).of_size(img.width(), img.height());
    render::draw_box(img, BLACK, rect);
    let rect = Rect::at(1, 1).of_size(img.width() - 2, img.height() - 2);
    render::draw_box(img, BLACK, rect);

    let text = "\n  X ->\n Y\n\n |\n V";

    let rect = Rect::at(0, 0).of_size(img.width(), img.height());
    render::draw_text(img, Color::rgb(0, 0, 0), rect, text)?;


    let rect = Rect::at(50, 50).of_size(100, 100);
    render::draw_box(img, BLACK, rect);

    render::draw_line(img, BLACK, (55, 55), (145, 145));
    render::draw_line(img, BLACK, (135, 55), (65, 145));

    Ok(())
}
