use log::warn;

use crate::{driver::DisplayImage, vault::{collect_tasks, note_to_ast, read_today_note}};


pub fn calendar(img: &mut DisplayImage) -> Result<(), Box<dyn std::error::Error>> {
    let now = chrono::Local::now();

    match read_today_note() {
        Err(e) => {
            warn!("Could not open today note: {}", e);
        },
        Ok(note) => {
            let ast = note_to_ast(&note);
            let tasks = collect_tasks(&ast, &note, true)?;

            for task in tasks {

            }
        },
    }

    Ok(())
}
