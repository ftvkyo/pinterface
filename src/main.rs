use clap::Parser;

mod app_error;
mod args;
mod command;
mod driver;
mod render;
mod vault;
mod util;

use args::{Command, DisplayMode};
use driver::Display as Dev;
use log::{error, info};
use util::*;


const IFNAME: &'static str = "wlan0";

// With JetBrains Mono Regular:
//
// | Font Scale | Width (chars) |
// | ---------- | ------------- |
// | 20.0       | 29            |
// | 16.0       | 36            |


fn try_main(args: &args::Args) -> Result<(), Box<dyn std::error::Error>> {

    let mut dev = Dev::new()?;

    loop {
        let mut img = Dev::image_white_h();

        match args.command {
            Command::Clear => {},
            Command::Debug => {
                command::debug::debug(&mut img)?;
            },
            Command::Tasks => {
                command::tasks::tasks(&mut img)?;
            },
            Command::Calendar => {},
            Command::Network => {
                command::network::network(&mut img, IFNAME)?;
            },
        };

        // Save the image if required

        if args.debug {
            img.save("out/debug.png")?;
        }

        // Initialize and clear

        dev.init(DisplayMode::Fast)?;
        dev.clear(DisplayMode::Fast)?;

        // Reinitialize and display something

        dev.init(args.mode)?;

        dev.display(img, args.mode)?;
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
