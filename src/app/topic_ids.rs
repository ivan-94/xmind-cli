use std::collections::BTreeSet;

use crate::domain::topic::Topic;
use crate::domain::workbook::Workbook;

use super::tree_input::TopicTreeInputDto;

#[derive(Debug, Default)]
pub(super) struct TopicIdAllocator {
    used_ids: BTreeSet<String>,
}

impl TopicIdAllocator {
    pub(super) fn new() -> Self {
        Self::default()
    }

    pub(super) fn from_root(root: &Topic) -> Self {
        let mut allocator = Self::new();
        allocator.collect_topic(root);
        allocator
    }

    pub(super) fn from_workbook(workbook: &Workbook) -> Self {
        let mut allocator = Self::new();
        for sheet in &workbook.sheets {
            allocator.collect_topic(&sheet.root);
        }
        allocator
    }

    pub(super) fn allocate(&mut self, title: &str) -> String {
        let base = generated_topic_id(title);
        if self.used_ids.insert(base.clone()) {
            return base;
        }

        for suffix in 2.. {
            let candidate = format!("{base}-{suffix}");
            if self.used_ids.insert(candidate.clone()) {
                return candidate;
            }
        }

        unreachable!("unbounded suffix search should always find a unique generated topic id")
    }

    pub(super) fn allocate_for_tree(&mut self, tree: &TopicTreeInputDto) -> String {
        match &tree.id {
            Some(id) => {
                self.used_ids.insert(id.clone());
                id.clone()
            }
            None => self.allocate(&tree.title),
        }
    }

    fn collect_topic(&mut self, topic: &Topic) {
        self.used_ids.insert(topic.id.0.clone());
        for child in &topic.children {
            self.collect_topic(child);
        }
    }
}

fn generated_topic_id(title: &str) -> String {
    let slug = title
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect::<String>();

    if slug.is_empty() {
        "topic-new".to_owned()
    } else {
        format!("topic-{slug}")
    }
}
