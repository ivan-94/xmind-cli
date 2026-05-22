use crate::domain::path::{PathError, TopicPath};
use crate::domain::query::{QueryExpr, QueryParseError};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Selector {
    Root,
    Id(String),
    Path(TopicPath),
    Title(String),
    Query { expr: QueryExpr, source: String },
}

impl Selector {
    pub fn parse(input: &str) -> Result<Self, SelectorParseError> {
        if input == "root" {
            return Ok(Self::Root);
        }

        if let Some(id) = input.strip_prefix("id:") {
            return Ok(Self::Id(id.to_owned()));
        }

        if let Some(path) = input.strip_prefix("path:") {
            return TopicPath::parse_selector_value(path)
                .map(Self::Path)
                .map_err(SelectorParseError::InvalidPath);
        }

        if let Some(title) = input.strip_prefix("title:") {
            return Ok(Self::Title(title.to_owned()));
        }

        if let Some(query) = input.strip_prefix("query:") {
            return QueryExpr::parse(query)
                .map(|expr| Self::Query {
                    expr,
                    source: query.to_owned(),
                })
                .map_err(SelectorParseError::InvalidQuery);
        }

        Err(SelectorParseError::UnknownSelector)
    }

    pub fn render(&self) -> String {
        match self {
            Self::Root => "root".to_owned(),
            Self::Id(id) => format!("id:{id}"),
            Self::Path(path) => format!("path:{}", path.to_selector_value()),
            Self::Title(title) => format!("title:{title}"),
            Self::Query { source, .. } => format!("query:{source}"),
        }
    }
}

#[derive(Debug, Eq, PartialEq, thiserror::Error)]
pub enum SelectorParseError {
    #[error("selector type is not supported")]
    UnknownSelector,

    #[error("path selector is invalid: {0}")]
    InvalidPath(PathError),

    #[error("query selector is invalid: {0}")]
    InvalidQuery(QueryParseError),
}

#[cfg(test)]
mod tests {
    use super::Selector;

    #[test]
    fn parses_root_selector() {
        assert_eq!(
            Selector::parse("root").expect("root parses"),
            Selector::Root
        );
    }

    #[test]
    fn parses_id_selector() {
        assert_eq!(
            Selector::parse("id:topic-123").expect("id parses"),
            Selector::Id("topic-123".to_owned())
        );
    }

    #[test]
    fn parses_path_selector_with_escaping() {
        let selector = Selector::parse(r"path:/API\/SDK").expect("path parses");

        assert_eq!(selector.render(), r"path:/API\/SDK");
    }

    #[test]
    fn parses_title_selector() {
        assert_eq!(
            Selector::parse("title:Payment").expect("title parses"),
            Selector::Title("Payment".to_owned())
        );
    }

    #[test]
    fn parses_query_selector() {
        let selector = Selector::parse(r#"query:title = "Payment""#).expect("query parses");

        assert_eq!(selector.render(), r#"query:title = "Payment""#);
    }
}
