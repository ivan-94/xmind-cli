#![allow(dead_code)]

use crate::domain::path::TopicPath;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Diff {
    events: Vec<DiffEvent>,
}

impl Diff {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_events(events: Vec<DiffEvent>) -> Self {
        Self { events }
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn events(&self) -> &[DiffEvent] {
        &self.events
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DiffEvent {
    Added {
        path: TopicPath,
    },
    Removed {
        path: TopicPath,
    },
    Updated {
        path: TopicPath,
        fields: Vec<FieldChange>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FieldChange {
    field: String,
}

impl FieldChange {
    pub fn new(field: impl Into<String>) -> Self {
        Self {
            field: field.into(),
        }
    }

    pub fn field(&self) -> &str {
        &self.field
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::path::TopicPath;

    use super::{Diff, DiffEvent, FieldChange};

    #[test]
    fn new_diff_starts_without_events() {
        let diff = Diff::new();

        assert!(diff.is_empty());
        assert_eq!(diff.len(), 0);
        assert!(diff.events().is_empty());
    }

    #[test]
    fn added_event_carries_topic_path() {
        let path = TopicPath::parse_selector_value("/Q2/Payment").expect("path parses");
        let diff = Diff::from_events(vec![DiffEvent::Added { path: path.clone() }]);

        assert!(!diff.is_empty());
        assert_eq!(diff.len(), 1);
        assert_eq!(diff.events(), &[DiffEvent::Added { path }]);
    }

    #[test]
    fn removed_event_carries_topic_path() {
        let path = TopicPath::parse_selector_value("/Q2/Legacy").expect("path parses");
        let diff = Diff::from_events(vec![DiffEvent::Removed { path: path.clone() }]);

        assert_eq!(diff.events(), &[DiffEvent::Removed { path }]);
    }

    #[test]
    fn updated_event_carries_topic_path_and_changed_fields() {
        let path = TopicPath::parse_selector_value("/Q2/Payment").expect("path parses");
        let field = FieldChange::new("title");
        let diff = Diff::from_events(vec![DiffEvent::Updated {
            path: path.clone(),
            fields: vec![field.clone()],
        }]);

        assert_eq!(
            diff.events(),
            &[DiffEvent::Updated {
                path,
                fields: vec![field]
            }]
        );
    }
}
