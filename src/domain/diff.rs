#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Diff {
    events: Vec<DiffEvent>,
}

impl Diff {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.events.len()
    }

    #[allow(dead_code)]
    pub fn events(&self) -> &[DiffEvent] {
        &self.events
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DiffEvent {}

#[cfg(test)]
mod tests {
    use super::Diff;

    #[test]
    fn new_diff_starts_without_events() {
        let diff = Diff::new();

        assert!(diff.is_empty());
        assert_eq!(diff.len(), 0);
        assert!(diff.events().is_empty());
    }
}
