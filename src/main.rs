use ab_glyph::FontRef;
use args::Command;
use clap::Parser;
use imageproc::drawing;

mod args;
mod driver;
mod app_error;
mod util;

use driver::{Display as Dev, DisplayMode, BLACK};
use log::{debug, error, info};
use util::*;


const IFNAME: &'static str = "wlan0";
const FONT: &[u8] = include_bytes!("/usr/share/fonts/truetype/jetbrains-mono/JetBrainsMono-Regular.ttf");
const FONT_SCALE: f32 = 16.0;

// With JetBrains Mono Regular:
//
// | Font Scale | Width (chars) |
// | ---------- | ------------- |
// | 20.0       | 29            |
// | 16.0       | 36            |

fn try_main(args: &args::Args) -> Result<(), Box<dyn std::error::Error>> {

    let font = FontRef::try_from_slice(FONT)?;

    let mut dev = Dev::new()?;

    let mode = match args.fast {
        false => DisplayMode::Full,
        true => DisplayMode::Fast,
    };

    loop {

        // Initialize

        dev.init(mode)?;
        dev.clear(mode)?;

        // Display an image

        let mut image = Dev::image_white();

        match args.command {
            Command::Clear => {},
            Command::Calendar => {
                // Letter X
                drawing::draw_line_segment_mut(&mut image, (5.0, 5.0), (15.0, 15.0), BLACK);
                drawing::draw_line_segment_mut(&mut image, (15.0, 5.0), (5.0, 15.0), BLACK);

                // Arrow to the right
                drawing::draw_line_segment_mut(&mut image, (15.0, 10.0), (30.0, 10.0), BLACK);
                drawing::draw_line_segment_mut(&mut image, (25.0, 5.0), (30.0, 10.0), BLACK);
                drawing::draw_line_segment_mut(&mut image, (25.0, 15.0), (30.0, 10.0), BLACK);

                // Another cross
                drawing::draw_line_segment_mut(&mut image, (50.0, 50.0), (70.0, 70.0), BLACK);
                drawing::draw_line_segment_mut(&mut image, (70.0, 50.0), (50.0, 70.0), BLACK);
            },
            Command::Network => {
                let output = net_info(IFNAME)?;
                for (line, text) in output.trim().split("\n").enumerate() {
                    let x = 0;
                    let y = (line as f32 * FONT_SCALE) as i32;
                    debug!("Drawing text '{}' at ({}, {})", text, x, y);
                    drawing::draw_text_mut(&mut image, BLACK, x, y, FONT_SCALE, &font, text);
                }
            },
        };

        dev.display(image, mode)?;
        dev.sleep()?;

        // Wait

        sleep_ms(10_000);

        // Deinitialize

        dev.init(DisplayMode::Full)?;
        dev.clear(DisplayMode::Full)?;
        dev.sleep()?;

        // Stop if not repeating

        if !args.repeat {
            break;
        }

        // Wait

        sleep_ms(10_000);
    }

    Ok(())
}

fn main() {
    dotenv::dotenv().ok(); // Don't fail when `.env` is not present
    pretty_env_logger::init();

    let args = args::Args::parse();
    info!("Received\n{:#?}", args);

    match try_main(&args) {
        Ok(_) => info!("Done!"),
        Err(e) => error!("Error: {}", e),
    };
}
