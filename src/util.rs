use std::thread;
use std::time::Duration;

use log::debug;


pub fn sleep_ms(ms: u64) {
    debug!("Sleeping for {} ms", ms);
    thread::sleep(Duration::from_millis(ms));
    debug!("Done sleeping for {} ms", ms);
}
