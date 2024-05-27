use cosmic_text::{Attrs, Buffer, Color, Family, FontSystem, Metrics, Shaping, SwashCache};
use image::Luma;
use imageproc::{drawing::draw_filled_rect_mut, rect::Rect};

use crate::driver::DisplayImage;


pub fn text(img: &mut DisplayImage, color: Color, text: &str) {
    let metrics = Metrics::new(16.0, 18.0);

    // Should be 1 per application
    let mut font_system = FontSystem::new();
    // Should be 1 per application
    let mut swash_cache = SwashCache::new();

    // Should be 1 per text widget
    let mut buffer = Buffer::new(&mut font_system, metrics);
    let mut buffer = buffer.borrow_with(&mut font_system);

    buffer.set_size(img.width() as f32, img.height() as f32);

    let attrs = Attrs::new()
        .family(Family::Name("JetBrains Mono"));

    buffer.set_text(text, attrs, Shaping::Advanced);
    buffer.shape_until_scroll(true);

    buffer.draw(&mut swash_cache, color, |x, y, w, h, color| {
        if x < 0 || y < 0 {
            // Out of bounds
            return;
        }

        if x as u32 > img.width() || y as u32 > img.height() {
            // Out of bounds
            return;
        }

        if w != 1 || h != 1 {
            // We don't like non-pixels (it's fiiine...)
            return;
        }

        // The input is RGBA, but the output is grayscale, so get the average color across R, G & B
        let grey = (color.r() as f32 + color.g() as f32 + color.b() as f32) / 3.0;
        // Extract the alpha as [0, 1] float so it's easier to mix the colors
        let alpha = color.a() as f32 / 255.0;

        let color_current = img.get_pixel(x as u32, y as u32).0[0] as f32;
        let color_new = grey * alpha + color_current * (1.0 - alpha);

        let rect = Rect::at(x, y).of_size(w, h);
        let color = Luma([color_new as u8]);
        draw_filled_rect_mut(img, rect, color);
    });
}
