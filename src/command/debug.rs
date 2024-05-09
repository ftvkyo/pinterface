use ab_glyph::FontRef;
use imageproc::drawing;

use crate::driver::{DisplayImage, BLACK};


pub fn debug(img: &mut DisplayImage, font: &FontRef, font_scale: f32) -> Result<(), Box<dyn std::error::Error>> {

    // Letter X
    drawing::draw_text_mut(img, BLACK, 20, 0, font_scale, font, "X");

    // Arrow right
    drawing::draw_line_segment_mut(img, (20.0, 15.0), (40.0, 15.0), BLACK);
    drawing::draw_line_segment_mut(img, (35.0, 10.0), (40.0, 15.0), BLACK);
    drawing::draw_line_segment_mut(img, (35.0, 20.0), (40.0, 15.0), BLACK);

    // Letter X
    drawing::draw_text_mut(img, BLACK, 5, 15, font_scale, font, "Y");

    // Arrow down
    drawing::draw_line_segment_mut(img, (15.0, 20.0), (15.0, 40.0), BLACK);
    drawing::draw_line_segment_mut(img, (10.0, 35.0), (15.0, 40.0), BLACK);
    drawing::draw_line_segment_mut(img, (20.0, 35.0), (15.0, 40.0), BLACK);

    let (w, h) = (img.width(), img.height());

    const BORDER: u32 = 2;

    for (x, y, pixel) in img.enumerate_pixels_mut() {
        if x < BORDER || y < BORDER || x >= w - BORDER || y >= h - BORDER {
            *pixel = BLACK;
        }
    }

    Ok(())
}
