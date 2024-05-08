use std::process::Command;
use std::thread;
use std::time::Duration;

use regex::RegexBuilder;

use crate::error::AppError;


pub fn sleep_ms(ms: u64) {
    thread::sleep(Duration::from_millis(ms));
}


pub fn net_info(interface: &str) -> Result<String, AppError> {
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
