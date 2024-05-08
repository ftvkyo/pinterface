use ab_glyph::FontRef;
use args::Mode;
use clap::Parser;
use imageproc::drawing;

mod args;
mod driver;
mod error;
mod util;

use driver::{Display as Dev, BLACK};
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

    loop {
        println!("Initializing!");

        // Initialize

        dev.init()?;
        dev.clear()?;

        // Display an image

        let mut image = Dev::image_white_h();

        match args.mode {
            Mode::Clear => {},
            Mode::Calendar => {
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
            Mode::Network => {
                let output = net_info(IFNAME)?;
                for (line, text) in output.trim().split("\n").enumerate() {
                    let x = 0;
                    let y = (line as f32 * FONT_SCALE) as i32;
                    println!("Drawing text '{}' at ({}, {})", text, x, y);
                    drawing::draw_text_mut(&mut image, BLACK, x, y, FONT_SCALE, &font, text);
                }
            },
        };

        dev.display_h(image)?;
        dev.sleep()?;

        // Wait

        sleep_ms(10_000);

        // Deinitialize

        println!("Deinitializing!");
        dev.init()?;
        dev.clear()?;
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
    let args = args::Args::parse();
    println!("Received {:#?}", args);

    match try_main(&args) {
        Ok(_) => println!("Done!"),
        Err(e) => println!("Error: {}", e),
    };
}
