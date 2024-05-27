use cosmic_text::Color;
use imageproc::rect::Rect;
use log::{debug, warn};

use crate::{driver::DisplayImage, render, vault::{collect_tasks, note_to_ast, read_today_note, Task}};


const TIME_PRETTY: &'static str = "<~ UwU ~> %Y-%m-%d %H:%M";


pub fn tasks(img: &mut DisplayImage) -> Result<(), Box<dyn std::error::Error>> {
    let now = chrono::Local::now();

    match read_today_note() {
        Err(e) => {
            warn!("Could not open today note: {}", e);
        },
        Ok(note) => {
            let ast = note_to_ast(&note);
            let tasks = collect_tasks(&ast, &note, true)?;

            debug!("Found unchecked tasks:\n{:#?}", tasks);

            let text = format!("{}\n{}", now.format(TIME_PRETTY), format_tasks(tasks, 0));

            let rect = Rect::at(0, 0).of_size(img.width(), img.height());
            render::draw_text(img, Color::rgb(0, 0, 0), rect, &text)?;
        },
    }

    Ok(())
}


fn format_tasks(tasks: Vec<Task>, depth: u8) -> String {
    let mut result = String::new();

    for task in tasks {
        for _ in 0..depth {
            result.push_str(" ");
        }
        let text = format!("- {}\n", task.text);
        result.push_str(&text);
        result.push_str(&format_tasks(task.subtasks, depth + 2));
    }

    result
}
