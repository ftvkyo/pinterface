use cosmic_text::Color;

use crate::{driver::{DisplayImage, BLACK}, render};


pub fn debug(img: &mut DisplayImage) -> Result<(), Box<dyn std::error::Error>> {

    let text = "\n  X ->\n Y\n\n |\n V";

    // Letter X
    render::text(img, Color::rgb(0, 0, 0), text)?;

    let (w, h) = (img.width(), img.height());

    const BORDER: u32 = 2;

    for (x, y, pixel) in img.enumerate_pixels_mut() {
        if x < BORDER || y < BORDER || x >= w - BORDER || y >= h - BORDER {
            *pixel = BLACK;
        }
    }

    Ok(())
}
