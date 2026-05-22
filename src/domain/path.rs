#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TopicPath {
    segments: Vec<String>,
}

impl TopicPath {
    pub fn root() -> Self {
        Self {
            segments: Vec::new(),
        }
    }

    pub fn from_segments(segments: Vec<String>) -> Self {
        Self { segments }
    }

    pub fn parse_selector_value(input: &str) -> Result<Self, PathError> {
        if input == "/" {
            return Ok(Self::root());
        }

        let Some(rest) = input.strip_prefix('/') else {
            return Err(PathError::MustBeAbsolute);
        };

        if rest.is_empty() {
            return Ok(Self::root());
        }

        parse_segments(rest).map(Self::from_segments)
    }

    pub fn to_selector_value(&self) -> String {
        if self.segments.is_empty() {
            "/".to_owned()
        } else {
            format!(
                "/{}",
                self.segments
                    .iter()
                    .map(|segment| escape_segment(segment))
                    .collect::<Vec<_>>()
                    .join("/")
            )
        }
    }

    pub fn is_root(&self) -> bool {
        self.segments.is_empty()
    }

    pub fn segments(&self) -> &[String] {
        &self.segments
    }

    pub fn join(&self, segment: impl Into<String>) -> Self {
        let mut segments = self.segments.clone();
        segments.push(segment.into());
        Self { segments }
    }
}

#[derive(Debug, Eq, PartialEq, thiserror::Error)]
pub enum PathError {
    #[error("topic path must be absolute")]
    MustBeAbsolute,

    #[error("topic path contains an incomplete escape sequence")]
    IncompleteEscape,
}

fn parse_segments(input: &str) -> Result<Vec<String>, PathError> {
    let mut segments = Vec::new();
    let mut current = String::new();
    let mut escaped = false;

    for character in input.chars() {
        if escaped {
            current.push(character);
            escaped = false;
            continue;
        }

        match character {
            '\\' => escaped = true,
            '/' => {
                segments.push(current);
                current = String::new();
            }
            _ => current.push(character),
        }
    }

    if escaped {
        return Err(PathError::IncompleteEscape);
    }

    segments.push(current);
    Ok(segments)
}

fn escape_segment(segment: &str) -> String {
    segment.replace('\\', "\\\\").replace('/', "\\/")
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::TopicPath;

    #[test]
    fn root_path_round_trips() {
        let path = TopicPath::parse_selector_value("/").expect("root path parses");

        assert!(path.is_root());
        assert_eq!(path.to_selector_value(), "/");
        assert_eq!(path.segments(), &[] as &[String]);
    }

    #[test]
    fn ordinary_path_round_trips_without_root_title() {
        let path = TopicPath::parse_selector_value("/Q2/Payment").expect("ordinary path parses");

        assert!(!path.is_root());
        assert_eq!(path.segments(), &["Q2".to_owned(), "Payment".to_owned()]);
        assert_eq!(path.to_selector_value(), "/Q2/Payment");
    }

    #[test]
    fn literal_slash_in_title_segment_round_trips() {
        let path = TopicPath::parse_selector_value(r"/API\/SDK").expect("escaped slash parses");

        assert_eq!(path.segments(), &["API/SDK".to_owned()]);
        assert_eq!(path.to_selector_value(), r"/API\/SDK");
    }

    #[test]
    fn literal_backslash_in_title_segment_round_trips() {
        let path =
            TopicPath::parse_selector_value(r"/Windows\\Path").expect("escaped backslash parses");

        assert_eq!(path.segments(), &["Windows\\Path".to_owned()]);
        assert_eq!(path.to_selector_value(), r"/Windows\\Path");
    }

    #[test]
    fn title_that_is_a_slash_is_not_root() {
        let path = TopicPath::parse_selector_value(r"/\/").expect("slash title parses");

        assert!(!path.is_root());
        assert_eq!(path.segments(), &["/".to_owned()]);
        assert_eq!(path.to_selector_value(), r"/\/");
    }

    proptest! {
        #[test]
        fn path_segments_round_trip(segments in proptest::collection::vec(".{1,8}", 0..6)) {
            let path = TopicPath::from_segments(segments);
            let rendered = path.to_selector_value();
            let parsed = TopicPath::parse_selector_value(&rendered).expect("rendered path parses");

            prop_assert_eq!(parsed, path);
        }
    }
}
