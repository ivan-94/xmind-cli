#![allow(dead_code)]

use crate::domain::diff::{Diff, DiffEvent};

pub fn render_human_outline(diff: &Diff) -> String {
    diff.events()
        .iter()
        .map(render_event)
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_event(event: &DiffEvent) -> String {
    match event {
        DiffEvent::Added { path } => format!("+ {}", path.to_selector_value()),
        DiffEvent::Removed { path } => format!("- {}", path.to_selector_value()),
        DiffEvent::Updated { path, fields } => {
            let field_list = fields
                .iter()
                .map(|field| field.field())
                .collect::<Vec<_>>()
                .join(",");
            format!("~ {} {}", path.to_selector_value(), field_list)
        }
        DiffEvent::Moved { from, to } => {
            format!(
                "> {} -> {}",
                from.to_selector_value(),
                to.to_selector_value()
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::diff::{Diff, DiffEvent, FieldChange};
    use crate::domain::path::TopicPath;

    use super::render_human_outline;

    #[test]
    fn human_outline_diff_renders_event_lines() {
        let diff = Diff::from_events(vec![
            DiffEvent::Added {
                path: TopicPath::parse_selector_value("/Q2/New").expect("path parses"),
            },
            DiffEvent::Removed {
                path: TopicPath::parse_selector_value("/Q2/Old").expect("path parses"),
            },
            DiffEvent::Updated {
                path: TopicPath::parse_selector_value("/Q2/Payment").expect("path parses"),
                fields: vec![FieldChange::new("title")],
            },
            DiffEvent::Moved {
                from: TopicPath::parse_selector_value("/Q2/Risk").expect("path parses"),
                to: TopicPath::parse_selector_value("/Q3/Risk").expect("path parses"),
            },
        ]);

        assert_eq!(
            render_human_outline(&diff),
            "+ /Q2/New\n- /Q2/Old\n~ /Q2/Payment title\n> /Q2/Risk -> /Q3/Risk"
        );
    }
}
