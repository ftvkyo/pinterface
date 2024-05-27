use chrono::{Duration, NaiveTime};
use lazy_static::lazy_static;
use markdown::mdast::{ListItem, Node, Paragraph};
use regex::Regex;
use serde::Deserialize;

use crate::app_error::AppError;


#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PluginConfig {
    #[serde(default)]
    pub group_by_year: bool,
    #[serde(default)]
    pub folder_periodic: String,
}


pub fn read_today_note() -> Result<String, Box<dyn std::error::Error>> {
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
    let note_s = std::fs::read_to_string(&note_path)?;

    return Ok(note_s);
}


pub fn note_to_ast(note: &String) -> Node {
    return markdown::to_mdast(&note, &markdown::ParseOptions::gfm())
        .expect("Markdown can't have syntax errors");
}


lazy_static! {
    static ref RE_TIME: Regex = Regex::new(r"\[time::\s*(?<h>\d{1,2}):(?<m>\d{2})\s+(?:(?<dh>\d+)h)?(?:(?<dm>\d+)m)?\s*\]").unwrap();
}


#[derive(Debug)]
pub struct Task {
    pub text: String,
    pub checked: bool,
    pub subtasks: Vec<Task>,
    pub time: Option<(NaiveTime, Duration)>,
}


pub fn collect_tasks(node: &Node, original: &str, only_unchecked: bool) -> Result<Vec<Task>, Box<dyn std::error::Error>> {
    // Expecting the top of recursion to not be a list item node :)

    let mut tasks = Vec::new();

    if let Some(children) = node.children() {
        for child in children {
            if let (
                Node::ListItem(li),
                Node::ListItem(ListItem {
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
                        Node::Paragraph(
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

                let subtasks = collect_tasks(child, original, only_unchecked)?;

                let time = if let Some(caps) = RE_TIME.captures(&text) {
                    let all = caps.get(0).unwrap();

                    let h: u32 = caps.name("h").unwrap().as_str().parse().unwrap();
                    let m: u32 = caps.name("m").unwrap().as_str().parse().unwrap();
                    let dh: u64 = caps.name("dh").map(|m| m.as_str().parse().unwrap()).unwrap_or(0);
                    let dm: u64 = caps.name("dm").map(|m| m.as_str().parse().unwrap()).unwrap_or(0);

                    let t = NaiveTime::from_hms_opt(h, m, 0).ok_or(AppError::Data(
                        format!("Wrong time in task '{}': {}:{}", text, h, m)
                    ))?;
                    let d = Duration::from_std(std::time::Duration::from_secs(
                        dh * 3600 + dm * 60
                    ))?;

                    // Remove the inline field from the task text
                    text.replace_range(all.start()..all.end(), "");

                    Some((t, d))
                } else {
                    None
                };

                tasks.push(Task {
                    text,
                    checked: *checked,
                    subtasks,
                    time,
                })
            } else {
                // Then this is not a task and we should propagate children tasks

                tasks.extend(collect_tasks(child, original, only_unchecked)?);
            }
        }
    }

    Ok(tasks)
}
