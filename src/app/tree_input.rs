use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::cli::MarkdownMode;

#[derive(Debug, Deserialize)]
pub(super) struct TopicTreeInputDto {
    pub(super) id: Option<String>,
    pub(super) path: Option<String>,
    pub(super) title: String,
    pub(super) note: Option<String>,
    #[serde(default)]
    pub(super) labels: Vec<String>,
    #[serde(default)]
    pub(super) markers: Vec<String>,
    pub(super) image: Option<TopicTreeImageInputDto>,

    #[serde(default)]
    pub(super) children: Vec<TopicTreeInputDto>,
}

impl TopicTreeInputDto {
    fn new(title: String) -> Self {
        Self {
            id: None,
            path: None,
            title,
            note: None,
            labels: Vec::new(),
            markers: Vec::new(),
            image: None,
            children: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(super) struct TopicTreeImageInputDto {
    pub(super) path: Option<String>,
    pub(super) asset_id: Option<String>,
    pub(super) alt: Option<String>,
    pub(super) title: Option<String>,
}

#[derive(Default, Debug, Deserialize)]
struct TopicTreeDefaultsDto {
    id: Option<String>,
    path: Option<String>,
    title: Option<String>,
    note: Option<String>,
    #[serde(default)]
    labels: Vec<String>,
    #[serde(default)]
    markers: Vec<String>,
    image: Option<TopicTreeImageInputDto>,
}

impl TopicTreeDefaultsDto {
    fn into_topic_tree(self) -> Option<TopicTreeInputDto> {
        Some(TopicTreeInputDto {
            id: self.id,
            path: self.path,
            title: self.title?,
            note: self.note,
            labels: self.labels,
            markers: self.markers,
            image: self.image,
            children: Vec::new(),
        })
    }
}

pub(super) struct TreeInputValidationError {
    pub(super) message: String,
    pub(super) field_path: String,
}

pub(super) fn read_tree_input(
    input: Option<&Path>,
    from_markdown: Option<&Path>,
    markdown_mode: Option<MarkdownMode>,
) -> Result<TopicTreeInputDto, String> {
    match (input, from_markdown) {
        (Some(input), None) => read_yaml_or_json_tree_input(input),
        (None, Some(input)) => read_markdown_tree_input(input, markdown_mode),
        (None, None) => Err("add-tree requires --input or --from-markdown.".to_owned()),
        (Some(_), Some(_)) => {
            Err("add-tree accepts only one of --input or --from-markdown.".to_owned())
        }
    }
}

pub(super) fn validate_topic_tree_input(
    tree: &TopicTreeInputDto,
) -> Result<(), TreeInputValidationError> {
    validate_topic_tree_node(tree, "title")
}

fn read_yaml_or_json_tree_input(input: &Path) -> Result<TopicTreeInputDto, String> {
    let extension = input
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or_default();
    let content = fs::read_to_string(input)
        .map_err(|error| format!("Tree input could not be read: {error}"))?;

    match extension {
        "yaml" | "yml" => serde_yaml::from_str(&content)
            .map_err(|error| format!("Tree input YAML is invalid: {error}")),
        "json" => serde_json::from_str(&content)
            .map_err(|error| format!("Tree input JSON is invalid: {error}")),
        _ => Err("Tree input must use .yaml, .yml, or .json.".to_owned()),
    }
}

fn read_markdown_tree_input(
    input: &Path,
    markdown_mode: Option<MarkdownMode>,
) -> Result<TopicTreeInputDto, String> {
    let content = fs::read_to_string(input)
        .map_err(|error| format!("Markdown input could not be read: {error}"))?;
    let (frontmatter, body) = split_markdown_frontmatter(&content);
    let defaults = match frontmatter {
        Some(frontmatter) => serde_yaml::from_str::<TopicTreeDefaultsDto>(frontmatter)
            .map_err(|error| format!("Markdown frontmatter YAML is invalid: {error}"))?,
        None => TopicTreeDefaultsDto::default(),
    };
    reject_inline_metadata(body)?;
    reject_unsupported_markdown_mode_body(body, markdown_mode)?;

    if let Some(mut tree) = parse_markdown_outline(body)? {
        apply_topic_tree_defaults(&mut tree, defaults);
        return Ok(tree);
    }

    defaults.into_topic_tree().ok_or_else(|| {
        "Markdown input must include frontmatter title or a heading outline.".to_owned()
    })
}

fn split_markdown_frontmatter(content: &str) -> (Option<&str>, &str) {
    let Some(rest) = content.strip_prefix("---\n") else {
        return (None, content);
    };
    let Some(end) = rest.find("\n---") else {
        return (None, content);
    };
    let frontmatter = &rest[..end];
    let body = rest[end + "\n---".len()..]
        .strip_prefix('\n')
        .unwrap_or_default();
    (Some(frontmatter), body)
}

fn reject_inline_metadata(content: &str) -> Result<(), String> {
    if content
        .lines()
        .any(|line| line.contains('{') && line.contains('}'))
    {
        return Err("Inline metadata is not supported in Markdown input.".to_owned());
    }

    Ok(())
}

fn reject_unsupported_markdown_mode_body(
    content: &str,
    markdown_mode: Option<MarkdownMode>,
) -> Result<(), String> {
    if markdown_mode == Some(MarkdownMode::Heading)
        && content.lines().any(parse_markdown_list_item_line)
    {
        return Err("Markdown heading mode does not accept list items.".to_owned());
    }
    if markdown_mode == Some(MarkdownMode::List) && content.lines().any(parse_markdown_heading_line)
    {
        return Err("Markdown list mode does not accept headings.".to_owned());
    }

    Ok(())
}

fn parse_markdown_list_item_line(line: &str) -> bool {
    parse_markdown_list_item(line).is_some()
}

fn parse_markdown_heading_line(line: &str) -> bool {
    parse_markdown_heading(line).is_some()
}

fn parse_markdown_outline(content: &str) -> Result<Option<TopicTreeInputDto>, String> {
    let mut stack = Vec::<(usize, usize, TopicTreeInputDto)>::new();
    let mut roots = Vec::<(usize, TopicTreeInputDto)>::new();
    let mut current_heading_level = 0;

    for (line_index, line) in content.lines().enumerate() {
        let line_number = line_index + 1;
        validate_markdown_list_indent(line, line_index + 1)?;
        validate_markdown_list_title(line, line_index + 1)?;
        let item = if let Some((level, title)) = parse_markdown_heading(line) {
            if current_heading_level > 0 && level > current_heading_level + 1 {
                return Err(format!(
                    "Markdown heading levels cannot skip from {current_heading_level} to {level} at line {}.",
                    line_index + 1
                ));
            }
            current_heading_level = level;
            Some((level, TopicTreeInputDto::new(title)))
        } else if let Some((relative_level, node)) = parse_markdown_list_item(line) {
            Some((current_heading_level + relative_level, node))
        } else {
            if let Some((_, _, current)) = stack.last_mut() {
                append_markdown_note(current, line);
            }
            None
        };
        let Some((level, node)) = item else {
            continue;
        };

        while stack
            .last()
            .is_some_and(|(stack_level, _, _)| *stack_level >= level)
        {
            let (_, completed_line, completed) = stack.pop().expect("stack is not empty");
            if let Some((_, _, parent)) = stack.last_mut() {
                parent.children.push(completed);
            } else {
                roots.push((completed_line, completed));
            }
        }

        stack.push((level, line_number, node));
    }

    while let Some((_, completed_line, completed)) = stack.pop() {
        if let Some((_, _, parent)) = stack.last_mut() {
            parent.children.push(completed);
        } else {
            roots.push((completed_line, completed));
        }
    }

    match roots.len() {
        0 => Ok(None),
        1 => Ok(roots.pop().map(|(_, root)| root)),
        _ => Err(format!(
            "Markdown outline must contain one top-level root; second root starts at line {}.",
            roots[1].0
        )),
    }
}

fn validate_markdown_list_indent(line: &str, line_number: usize) -> Result<(), String> {
    let indent = line
        .chars()
        .take_while(|character| *character == ' ')
        .count();
    let trimmed = line.trim_start();
    if trimmed.starts_with("- ") && indent % 2 != 0 {
        return Err(format!(
            "Markdown unordered list indentation must use multiples of 2 spaces at line {line_number}."
        ));
    }

    if is_ordered_markdown_list_item(trimmed) && indent % 3 != 0 {
        return Err(format!(
            "Markdown ordered list indentation must use multiples of 3 spaces at line {line_number}."
        ));
    }

    Ok(())
}

fn validate_markdown_list_title(line: &str, line_number: usize) -> Result<(), String> {
    let trimmed = line.trim_start();
    let Some(title) = markdown_list_title_text(trimmed) else {
        return Ok(());
    };
    if strip_markdown_task_marker(title.trim()).trim().is_empty() {
        return Err(format!(
            "Markdown list item title is empty at line {line_number}."
        ));
    }

    Ok(())
}

fn markdown_list_title_text(trimmed: &str) -> Option<&str> {
    if let Some(title) = trimmed.strip_prefix("- ") {
        return Some(title);
    }

    let dot_index = trimmed.find(". ")?;
    if !is_ordered_markdown_list_item(trimmed) {
        return None;
    }

    Some(&trimmed[dot_index + ". ".len()..])
}

fn strip_markdown_task_marker(title: &str) -> &str {
    title
        .strip_prefix("[ ]")
        .or_else(|| title.strip_prefix("[x]"))
        .or_else(|| title.strip_prefix("[X]"))
        .unwrap_or(title)
}

fn is_ordered_markdown_list_item(trimmed: &str) -> bool {
    let Some(dot_index) = trimmed.find(". ") else {
        return false;
    };

    dot_index > 0
        && trimmed[..dot_index]
            .chars()
            .all(|character| character.is_ascii_digit())
}

fn append_markdown_note(topic: &mut TopicTreeInputDto, line: &str) {
    let line = line.trim();
    if line.is_empty() {
        return;
    }

    if let Some(note) = &mut topic.note {
        note.push('\n');
        note.push_str(line);
    } else {
        topic.note = Some(line.to_owned());
    }
}

fn parse_markdown_list_item(line: &str) -> Option<(usize, TopicTreeInputDto)> {
    let indent = line
        .chars()
        .take_while(|character| *character == ' ')
        .count();
    let trimmed = line.trim_start();
    if let Some(title) = trimmed.strip_prefix("- ") {
        let node = parse_markdown_list_topic(title.trim())?;
        return Some((indent / 2 + 1, node));
    }

    let dot_index = trimmed.find(". ")?;
    if dot_index == 0
        || !trimmed[..dot_index]
            .chars()
            .all(|character| character.is_ascii_digit())
    {
        return None;
    }
    let title = trimmed[dot_index + ". ".len()..].trim();
    let node = parse_markdown_list_topic(title)?;

    Some((indent / 3 + 1, node))
}

fn parse_markdown_list_topic(title: &str) -> Option<TopicTreeInputDto> {
    let (title, marker) = if let Some(title) = title.strip_prefix("[ ] ") {
        (title.trim(), Some("task-open"))
    } else if let Some(title) = title.strip_prefix("[x] ") {
        (title.trim(), Some("task-done"))
    } else if let Some(title) = title.strip_prefix("[X] ") {
        (title.trim(), Some("task-done"))
    } else {
        (title, None)
    };

    if title.is_empty() {
        return None;
    }

    let mut node = TopicTreeInputDto::new(title.to_owned());
    if let Some(marker) = marker {
        node.markers.push(marker.to_owned());
    }
    Some(node)
}

fn parse_markdown_heading(line: &str) -> Option<(usize, String)> {
    let trimmed = line.trim_start();
    let level = trimmed
        .chars()
        .take_while(|character| *character == '#')
        .count();
    if level == 0 || level > 6 {
        return None;
    }

    let title = trimmed[level..].trim();
    if title.is_empty() {
        return None;
    }

    Some((level, title.to_owned()))
}

fn apply_topic_tree_defaults(tree: &mut TopicTreeInputDto, defaults: TopicTreeDefaultsDto) {
    if tree.id.is_none() {
        tree.id = defaults.id;
    }
    if tree.note.is_none() {
        tree.note = defaults.note;
    }
    if tree.labels.is_empty() {
        tree.labels = defaults.labels;
    }
    if tree.markers.is_empty() {
        tree.markers = defaults.markers;
    }
    if tree.image.is_none() {
        tree.image = defaults.image;
    }
}

fn validate_topic_tree_node(
    tree: &TopicTreeInputDto,
    title_field_path: &str,
) -> Result<(), TreeInputValidationError> {
    if tree.title.trim().is_empty() {
        return Err(TreeInputValidationError {
            message: "Topic tree title must not be empty.".to_owned(),
            field_path: title_field_path.to_owned(),
        });
    }

    let child_prefix = title_field_path
        .strip_suffix(".title")
        .unwrap_or_default()
        .to_owned();
    for (index, child) in tree.children.iter().enumerate() {
        let child_title_path = if child_prefix.is_empty() {
            format!("children[{index}].title")
        } else {
            format!("{child_prefix}.children[{index}].title")
        };
        validate_topic_tree_node(child, &child_title_path)?;
    }

    Ok(())
}
