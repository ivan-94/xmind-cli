use super::path::TopicPath;
use super::topic::Topic;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum QueryExpr {
    Comparison(QueryComparison),
}

impl QueryExpr {
    pub fn parse(input: &str) -> Result<Self, QueryParseError> {
        let mut parser = QueryParser::new(input);
        let expr = parser.parse_comparison()?;
        parser.expect_end()?;
        Ok(Self::Comparison(expr))
    }

    pub fn matches_topic(&self, topic: &Topic, path: &TopicPath, depth: usize) -> bool {
        match self {
            Self::Comparison(comparison) => comparison.matches_topic(topic, path, depth),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueryComparison {
    field: QueryField,
    operator: QueryOperator,
    value: QueryValue,
}

impl QueryComparison {
    fn matches_topic(&self, topic: &Topic, _path: &TopicPath, _depth: usize) -> bool {
        match (&self.field, &self.operator, &self.value) {
            (QueryField::Title, QueryOperator::Eq, QueryValue::String(expected)) => {
                topic.title == *expected
            }
            (QueryField::Title, QueryOperator::Ne, QueryValue::String(expected)) => {
                topic.title != *expected
            }
            (QueryField::Title, QueryOperator::Contains, QueryValue::String(needle)) => {
                topic.title.contains(needle)
            }
            (QueryField::Title, QueryOperator::StartsWith, QueryValue::String(prefix)) => {
                topic.title.starts_with(prefix)
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum QueryField {
    Title,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum QueryOperator {
    Eq,
    Ne,
    Contains,
    StartsWith,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum QueryValue {
    String(String),
}

#[derive(Debug, Eq, PartialEq, thiserror::Error)]
pub enum QueryParseError {
    #[error("expected field name")]
    ExpectedField,

    #[error("unsupported query field: {0}")]
    UnsupportedField(String),

    #[error("expected operator")]
    ExpectedOperator,

    #[error("unsupported operator for this query slice: {0}")]
    UnsupportedOperator(String),

    #[error("expected quoted string value")]
    ExpectedStringValue,

    #[error("query string contains an incomplete escape sequence")]
    IncompleteEscape,

    #[error("query string is missing a closing quote")]
    UnterminatedString,

    #[error("unexpected trailing query input: {0}")]
    TrailingInput(String),
}

struct QueryParser<'a> {
    input: &'a str,
    position: usize,
}

impl<'a> QueryParser<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, position: 0 }
    }

    fn parse_comparison(&mut self) -> Result<QueryComparison, QueryParseError> {
        let field = self.parse_field()?;
        let operator = self.parse_operator()?;
        let value = self.parse_string_value()?;

        Ok(QueryComparison {
            field,
            operator,
            value,
        })
    }

    fn parse_field(&mut self) -> Result<QueryField, QueryParseError> {
        let Some(identifier) = self.parse_identifier() else {
            return Err(QueryParseError::ExpectedField);
        };

        match identifier {
            "title" => Ok(QueryField::Title),
            other => Err(QueryParseError::UnsupportedField(other.to_owned())),
        }
    }

    fn parse_operator(&mut self) -> Result<QueryOperator, QueryParseError> {
        self.skip_whitespace();

        let Some(operator) = self.parse_operator_token() else {
            return Err(QueryParseError::ExpectedOperator);
        };

        match operator {
            "=" => Ok(QueryOperator::Eq),
            "!=" => Ok(QueryOperator::Ne),
            "contains" => Ok(QueryOperator::Contains),
            "starts_with" => Ok(QueryOperator::StartsWith),
            other => Err(QueryParseError::UnsupportedOperator(other.to_owned())),
        }
    }

    fn parse_string_value(&mut self) -> Result<QueryValue, QueryParseError> {
        self.skip_whitespace();

        if !self.consume_char('"') {
            return Err(QueryParseError::ExpectedStringValue);
        }

        let mut value = String::new();
        let mut escaped = false;

        while let Some(character) = self.next_char() {
            if escaped {
                value.push(character);
                escaped = false;
                continue;
            }

            match character {
                '\\' => escaped = true,
                '"' => return Ok(QueryValue::String(value)),
                _ => value.push(character),
            }
        }

        if escaped {
            Err(QueryParseError::IncompleteEscape)
        } else {
            Err(QueryParseError::UnterminatedString)
        }
    }

    fn expect_end(&mut self) -> Result<(), QueryParseError> {
        self.skip_whitespace();

        if self.position == self.input.len() {
            Ok(())
        } else {
            Err(QueryParseError::TrailingInput(
                self.input[self.position..].to_owned(),
            ))
        }
    }

    fn parse_identifier(&mut self) -> Option<&'a str> {
        self.skip_whitespace();
        let start = self.position;

        while let Some(character) = self.peek_char() {
            if character.is_ascii_alphanumeric() || character == '_' {
                self.position += character.len_utf8();
            } else {
                break;
            }
        }

        (self.position > start).then_some(&self.input[start..self.position])
    }

    fn parse_operator_token(&mut self) -> Option<&'a str> {
        for operator in ["!=", ">=", "<=", "=", ">", "<"] {
            if self.input[self.position..].starts_with(operator) {
                self.position += operator.len();
                return Some(operator);
            }
        }

        self.parse_identifier()
    }

    fn skip_whitespace(&mut self) {
        while let Some(character) = self.peek_char() {
            if character.is_whitespace() {
                self.position += character.len_utf8();
            } else {
                break;
            }
        }
    }

    fn consume_char(&mut self, expected: char) -> bool {
        if self.peek_char() == Some(expected) {
            self.position += expected.len_utf8();
            true
        } else {
            false
        }
    }

    fn peek_char(&self) -> Option<char> {
        self.input[self.position..].chars().next()
    }

    fn next_char(&mut self) -> Option<char> {
        let character = self.peek_char()?;
        self.position += character.len_utf8();
        Some(character)
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::path::TopicPath;
    use crate::domain::topic::{Topic, TopicId};

    use super::QueryExpr;

    #[test]
    fn title_equality_matches_topic_title() {
        let expr = QueryExpr::parse(r#"title = "Payment""#).expect("query parses");
        let topic = Topic {
            id: TopicId("topic-payment".to_owned()),
            title: "Payment".to_owned(),
            note: None,
            children: Vec::new(),
        };

        assert!(expr.matches_topic(&topic, &TopicPath::root(), 0));
    }

    #[test]
    fn title_inequality_rejects_matching_topic_title() {
        let expr = QueryExpr::parse(r#"title != "Payment""#).expect("query parses");
        let topic = Topic {
            id: TopicId("topic-payment".to_owned()),
            title: "Payment".to_owned(),
            note: None,
            children: Vec::new(),
        };

        assert!(!expr.matches_topic(&topic, &TopicPath::root(), 0));
    }

    #[test]
    fn title_contains_matches_substring() {
        let expr = QueryExpr::parse(r#"title contains "Pay""#).expect("query parses");
        let topic = Topic {
            id: TopicId("topic-payment".to_owned()),
            title: "Payment".to_owned(),
            note: None,
            children: Vec::new(),
        };

        assert!(expr.matches_topic(&topic, &TopicPath::root(), 0));
    }

    #[test]
    fn title_starts_with_matches_prefix() {
        let expr = QueryExpr::parse(r#"title starts_with "Pay""#).expect("query parses");
        let topic = Topic {
            id: TopicId("topic-payment".to_owned()),
            title: "Payment".to_owned(),
            note: None,
            children: Vec::new(),
        };

        assert!(expr.matches_topic(&topic, &TopicPath::root(), 0));
    }
}
