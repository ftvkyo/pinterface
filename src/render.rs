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
        if color.a() == 0 || w != 1 || h != 1 {
            return;
        }

        let color = color.r() / 3 + color.g() / 3 + color.b() / 3;
        let rect = Rect::at(x, y).of_size(w, h);
        let color = Luma([color]);
        draw_filled_rect_mut(img, rect, color);
    });
}
