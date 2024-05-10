use ab_glyph::FontRef;
use imageproc::drawing::draw_text_mut;
use log::{debug, info, warn};
use markdown::{mdast::{ListItem, Node as MdNode, Paragraph}, ParseOptions};
use serde::Deserialize;

use crate::driver::{DisplayImage, BLACK};


const TIME_PRETTY: &'static str = "<~ UwU ~> %Y-%m-%d %H:%M";


#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PluginConfig {
    #[serde(default)]
    pub group_by_year: bool,
    #[serde(default)]
    pub folder_periodic: String,
}


#[derive(Debug)]
struct Task {
    text: String,
    checked: bool,
    subtasks: Vec<Task>,
}


pub fn tasks(img: &mut DisplayImage, font: &FontRef, font_scale: f32) -> Result<(), Box<dyn std::error::Error>> {
    let vault = std::path::PathBuf::from(std::env::var("VAULT")?);
    let now = chrono::Local::now();

    let config_path = vault.join(".obsidian/plugins/ftvkyo/data.json");
    let config_s = std::fs::read_to_string(config_path)?;
    let config: PluginConfig = serde_json::from_str(&config_s)?;

    let note_pattern = if config.group_by_year {
        "%Y/%Y%m%d.md"
    } else {
        "%Y%m%d.md"
    };

    let note_filename = now.format(note_pattern).to_string();
    let note_path = vault.join(config.folder_periodic).join(note_filename);
    let note_s = std::fs::read_to_string(&note_path);

    match note_s {
        Err(e) => {
            warn!("Could not open {:?}: {}", note_path, e);
        },
        Ok(note_s) => {
            let note_ast = markdown::to_mdast(&note_s, &ParseOptions::gfm())
                .expect("Markdown can't have syntax errors");
            let tasks = collect_tasks(&note_ast, &note_s, true);

            debug!("Found unchecked tasks:\n{:#?}", tasks);

            draw_text_mut(
                img,
                BLACK,
                0,
                0,
                font_scale,
                font,
                &now.format(TIME_PRETTY).to_string(),
            );

            let mut x = 0;
            let mut y = font_scale as i32;

            draw_tasks(
                img,
                font,
                font_scale,
                &mut x,
                &mut y,
                tasks,
            );
        },
    }

    Ok(())
}


fn collect_tasks(node: &MdNode, original: &str, only_unchecked: bool) -> Vec<Task> {
    // Expecting the top of recursion to not be a list item node :)

    let mut tasks = Vec::new();

    if let Some(children) = node.children() {
        for child in children {
            if let (
                MdNode::ListItem(li),
                MdNode::ListItem(ListItem {
                    checked: Some(checked),
                    ..
                })
            ) = (child, child) {
                // Then this is a task

                if only_unchecked && *checked {
                    // Skip tasks that are checked if we only care about unchecked tasks
                    continue;
                }

                // Extract the text contents of the list item
                let mut text = String::new();
                for li_child in &li.children {
                    match li_child {
                        MdNode::Paragraph(
                            Paragraph {
                                position: Some(position),
                                ..
                            }
                        ) => {
                            let p_text = &original[position.start.offset..position.end.offset];
                            text.push_str(p_text);
                        },
                        _ => {
                            // Only extract paragraphs in the beginning.
                            break;
                        }
                    }
                }

                let subtasks = collect_tasks(child, original, only_unchecked);

                tasks.push(Task {
                    text,
                    checked: *checked,
                    subtasks,
                })
            } else {
                // Then this is not a task and we should propagate children tasks

                tasks.extend(collect_tasks(child, original, only_unchecked));
            }
        }
    }

    tasks
}


fn draw_tasks(img: &mut DisplayImage, font: &FontRef, font_scale: f32, x: &mut i32, y: &mut i32, tasks: Vec<Task>) {
    for task in tasks {
        let text = format!("- {}", task.text);
        draw_text_mut(img, BLACK, *x, *y, font_scale, font, &text);
        *y += font_scale as i32;
        *x += font_scale as i32;
        draw_tasks(img, font, font_scale, x, y, task.subtasks);
        *x -= font_scale as i32;
    }
}
