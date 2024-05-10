use ab_glyph::FontRef;
use clap::Parser;

mod args;
mod command;
mod driver;
mod app_error;
mod util;

use args::Command;
use driver::{Display as Dev, DisplayMode};
use log::{error, info};
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

        let mut img = Dev::image_white_h();

        match args.command {
            Command::Clear => {},
            Command::Debug => {
                command::debug::debug(&mut img, &font, FONT_SCALE)?;
            },
            Command::Tasks => {
                command::tasks::tasks(&mut img, &font, FONT_SCALE)?;
            },
            Command::Calendar => {},
            Command::Network => {
                command::network::network(&mut img, &font, FONT_SCALE, IFNAME)?;
            },
        };

        dev.display(img, mode)?;
        dev.sleep()?;

        // Wait

        sleep_ms(5_000);

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
