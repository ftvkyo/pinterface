use markdown::mdast::{ListItem, Node, Paragraph};
use serde::Deserialize;


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


#[derive(Debug)]
pub struct Task {
    pub text: String,
    pub checked: bool,
    pub subtasks: Vec<Task>,
}


pub fn collect_tasks(node: &Node, original: &str, only_unchecked: bool) -> Vec<Task> {
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
