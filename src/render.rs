use std::sync::Mutex;

use lazy_static::lazy_static;
use cosmic_text::{Attrs, Buffer, Color, Family, FontSystem, Metrics, Shaping, SwashCache};
use image::Luma;
use imageproc::{drawing::{draw_antialiased_line_segment_mut, draw_filled_rect_mut, draw_hollow_rect_mut}, pixelops::interpolate, rect::Rect};

use crate::driver::{DisplayImage, DisplayImagePixel};

lazy_static! {
    static ref FONT_SYSTEM: Mutex<FontSystem> = Mutex::new(FontSystem::new());
    static ref SWASH_CACHE: Mutex<SwashCache> = Mutex::new(SwashCache::new());
}


pub fn draw_line(img: &mut DisplayImage, color: DisplayImagePixel, start: (i32, i32), end: (i32, i32)) {
    draw_antialiased_line_segment_mut(img, start, end, color, interpolate);
}


pub fn draw_box(img: &mut DisplayImage, color: DisplayImagePixel, rect: Rect) {
    draw_hollow_rect_mut(img, rect, color);
}


pub fn draw_text(img: &mut DisplayImage, color: Color, rect: Rect, text: &str) -> Result<(), Box<dyn std::error::Error>> {
    let metrics = Metrics::new(16.0, 18.0);

    let mut font_system = FONT_SYSTEM.lock()?;
    let mut swash_cache = SWASH_CACHE.lock()?;

    // Should be 1 per text widget
    let mut buffer = Buffer::new(&mut font_system, metrics);
    let mut buffer = buffer.borrow_with(&mut font_system);

    buffer.set_size(rect.width() as f32, rect.height() as f32);

    let attrs = Attrs::new()
        .family(Family::Name("JetBrains Mono"));

    buffer.set_text(text, attrs, Shaping::Advanced);
    buffer.shape_until_scroll(true);

    buffer.draw(&mut swash_cache, color, |x, y, w, h, color| {
        if w != 1 || h != 1 {
            // We don't like non-pixels (it's fiiine...)
            return;
        }

        // Calculate the X and Y with the offset specified by the limiting rect
        let x = x + rect.left();
        let y = y + rect.top();

        if x < 0 || y < 0 || x as u32 > img.width() || y as u32 > img.height() {
            // Out of image bounds
            return;
        }

        // TODO: in theory, shaped text can escape the rectangle, this needs filtering out

        // The input is RGBA, but the output is grayscale, so get the average color across R, G & B
        let grey = (color.r() as f32 + color.g() as f32 + color.b() as f32) / 3.0;
        // Extract the alpha as [0, 1] float so it's easier to mix the colors
        let alpha = color.a() as f32 / 255.0;

        let color_current = img.get_pixel(x as u32, y as u32).0[0] as f32;
        let color_new = grey * alpha + color_current * (1.0 - alpha);

        let pixel = Rect::at(x, y).of_size(w, h);
        let color = Luma([color_new as u8]);
        draw_filled_rect_mut(img, pixel, color);
    });

    Ok(())
}
