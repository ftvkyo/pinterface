use image::Luma;
use imageproc::drawing;

mod driver;
mod util;

use driver::Display as Dev;
use util::*;


fn try_main() -> Result<(), Box<dyn std::error::Error>> {
    let mut dev = Dev::new()?;

    // Initialize

    dev.init()?;
    dev.clear()?;

    // Display an image

    let mut image = Dev::image_white();
    drawing::draw_line_segment_mut(&mut image, (10.0, 10.0), (30.0, 30.0), Luma([0]));
    dev.display(image)?;

    // Wait

    sleep_ms(10_000);

    // Deinitialize

    dev.init()?;
    dev.clear()?;
    dev.sleep()?;

    Ok(())
}

fn main() {
    match try_main() {
        Ok(_) => println!("Done!"),
        Err(e) => println!("Error: {}", e),
    };
}
