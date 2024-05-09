use std::process::Command;

use ab_glyph::FontRef;
use imageproc::drawing;
use log::{debug, info};
use regex::RegexBuilder;

use crate::{app_error::AppError, driver::{DisplayImage, BLACK}};


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


pub fn network(img: &mut DisplayImage, font: &FontRef, font_scale: f32, ifname: &str) -> Result<(), Box<dyn std::error::Error>> {
    let output = net_info(ifname)?;

    for (line, text) in output.trim().split("\n").enumerate() {
        let x = 0;
        let y = (line as f32 * font_scale) as i32;
        debug!("Drawing text '{}' at ({}, {})", text, x, y);
        drawing::draw_text_mut(img, BLACK, x, y, font_scale, &font, text);
    }

    Ok(())
}
