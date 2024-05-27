use std::process::Command;

use cosmic_text::Color;
use imageproc::rect::Rect;
use log::info;
use regex::RegexBuilder;

use crate::{app_error::AppError, driver::DisplayImage, render};


fn net_info(interface: &str) -> Result<String, AppError> {
    info!("Acquiring network info");
    let re_inet = RegexBuilder::new(r#"^\s*(inet6?\s+\S+?)\s.*$"#)
        .multi_line(true)
        .build()?;

    let output = Command::new("ip")
        .arg("addr")
        .arg("show")
        .arg("dev")
        .arg(interface)
        .output()?;

    let stdout = String::from_utf8(output.stdout)?;

    let mut output = String::new();
    for caps in re_inet.captures_iter(stdout.as_str()) {
        output.push_str(&caps[1]);
        output.push_str("\n");
    }

    Ok(output)
}


pub fn network(img: &mut DisplayImage, ifname: &str) -> Result<(), Box<dyn std::error::Error>> {
    let text = net_info(ifname)?;

    let rect = Rect::at(0, 0).of_size(img.width(), img.height());
    render::draw_text(img, Color::rgb(0, 0, 0), rect, text.trim())?;

    Ok(())
}
