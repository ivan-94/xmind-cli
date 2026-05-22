use super::path::TopicPath;
use super::topic::Topic;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum QueryExpr {
    Comparison(QueryComparison),
    And(Box<QueryExpr>, Box<QueryExpr>),
    Or(Box<QueryExpr>, Box<QueryExpr>),
    Not(Box<QueryExpr>),
}

impl QueryExpr {
    pub fn parse(input: &str) -> Result<Self, QueryParseError> {
        let mut parser = QueryParser::new(input);
        let expr = parser.parse_or_expr()?;
        parser.expect_end()?;
        Ok(expr)
    }

    pub fn matches_topic(&self, topic: &Topic, path: &TopicPath, depth: usize) -> bool {
        match self {
            Self::Comparison(comparison) => comparison.matches_topic(topic, path, depth),
            Self::And(left, right) => {
                left.matches_topic(topic, path, depth) && right.matches_topic(topic, path, depth)
            }
            Self::Or(left, right) => {
                left.matches_topic(topic, path, depth) || right.matches_topic(topic, path, depth)
            }
            Self::Not(expr) => !expr.matches_topic(topic, path, depth),
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
    fn matches_topic(&self, topic: &Topic, path: &TopicPath, depth: usize) -> bool {
        match (&self.field, &self.operator, &self.value) {
            (QueryField::Id, QueryOperator::Eq, QueryValue::String(expected)) => {
                topic.id.0 == *expected
            }
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
            (QueryField::Title, QueryOperator::EndsWith, QueryValue::String(suffix)) => {
                topic.title.ends_with(suffix)
            }
            (QueryField::Title, QueryOperator::In, QueryValue::StringList(expected)) => {
                expected.iter().any(|value| topic.title == *value)
            }
            (QueryField::Path, QueryOperator::Eq, QueryValue::String(expected)) => {
                path.to_selector_value() == *expected
            }
            (QueryField::Path, QueryOperator::Ne, QueryValue::String(expected)) => {
                path.to_selector_value() != *expected
            }
            (QueryField::Path, QueryOperator::Contains, QueryValue::String(needle)) => {
                path.to_selector_value().contains(needle)
            }
            (QueryField::Path, QueryOperator::StartsWith, QueryValue::String(prefix)) => {
                path.to_selector_value().starts_with(prefix)
            }
            (QueryField::Path, QueryOperator::In, QueryValue::StringList(expected)) => {
                let rendered_path = path.to_selector_value();
                expected.iter().any(|value| rendered_path == *value)
            }
            (QueryField::Note, QueryOperator::Eq, QueryValue::String(expected)) => {
                topic.note.as_deref() == Some(expected.as_str())
            }
            (QueryField::Note, QueryOperator::Ne, QueryValue::String(expected)) => {
                topic.note.as_deref() != Some(expected.as_str())
            }
            (QueryField::Note, QueryOperator::Contains, QueryValue::String(needle)) => topic
                .note
                .as_deref()
                .is_some_and(|note| note.contains(needle)),
            (QueryField::Note, QueryOperator::Exists, QueryValue::None) => topic.note.is_some(),
            (QueryField::Depth, QueryOperator::Gt, QueryValue::Number(expected)) => {
                (depth as i64) > *expected
            }
            (QueryField::Depth, QueryOperator::Eq, QueryValue::Number(expected)) => {
                (depth as i64) == *expected
            }
            (QueryField::Depth, QueryOperator::Gte, QueryValue::Number(expected)) => {
                (depth as i64) >= *expected
            }
            (QueryField::Depth, QueryOperator::Lt, QueryValue::Number(expected)) => {
                (depth as i64) < *expected
            }
            (QueryField::Depth, QueryOperator::Lte, QueryValue::Number(expected)) => {
                (depth as i64) <= *expected
            }
            _ => false,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum QueryField {
    Id,
    Title,
    Path,
    Note,
    Depth,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum QueryOperator {
    Eq,
    Ne,
    Contains,
    StartsWith,
    EndsWith,
    In,
    Exists,
    Gt,
    Gte,
    Lt,
    Lte,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum QueryValue {
    String(String),
    StringList(Vec<String>),
    Number(i64),
    None,
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

    #[error("query expression is missing a closing parenthesis")]
    ExpectedClosingParenthesis,

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

    fn parse_or_expr(&mut self) -> Result<QueryExpr, QueryParseError> {
        let mut expr = self.parse_and_expr()?;

        while self.consume_keyword("or") {
            let right = self.parse_and_expr()?;
            expr = QueryExpr::Or(Box::new(expr), Box::new(right));
        }

        Ok(expr)
    }

    fn parse_and_expr(&mut self) -> Result<QueryExpr, QueryParseError> {
        let mut expr = self.parse_not_expr()?;

        while self.consume_keyword("and") {
            let right = self.parse_not_expr()?;
            expr = QueryExpr::And(Box::new(expr), Box::new(right));
        }

        Ok(expr)
    }

    fn parse_not_expr(&mut self) -> Result<QueryExpr, QueryParseError> {
        if self.consume_keyword("not") {
            return Ok(QueryExpr::Not(Box::new(self.parse_not_expr()?)));
        }

        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<QueryExpr, QueryParseError> {
        self.skip_whitespace();
        if self.consume_char('(') {
            let expr = self.parse_or_expr()?;
            self.skip_whitespace();
            if !self.consume_char(')') {
                return Err(QueryParseError::ExpectedClosingParenthesis);
            }
            return Ok(expr);
        }

        Ok(QueryExpr::Comparison(self.parse_comparison()?))
    }

    fn parse_comparison(&mut self) -> Result<QueryComparison, QueryParseError> {
        let field = self.parse_field()?;
        let operator = self.parse_operator()?;
        let value = if operator == QueryOperator::Exists {
            QueryValue::None
        } else if operator == QueryOperator::In {
            self.parse_string_list_value()?
        } else if field == QueryField::Depth {
            self.parse_number_value()?
        } else {
            self.parse_string_value()?
        };

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
            "id" => Ok(QueryField::Id),
            "title" => Ok(QueryField::Title),
            "path" => Ok(QueryField::Path),
            "note" => Ok(QueryField::Note),
            "depth" => Ok(QueryField::Depth),
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
            "ends_with" => Ok(QueryOperator::EndsWith),
            "in" => Ok(QueryOperator::In),
            "exists" => Ok(QueryOperator::Exists),
            ">" => Ok(QueryOperator::Gt),
            ">=" => Ok(QueryOperator::Gte),
            "<" => Ok(QueryOperator::Lt),
            "<=" => Ok(QueryOperator::Lte),
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

    fn parse_string_list_value(&mut self) -> Result<QueryValue, QueryParseError> {
        self.skip_whitespace();

        if !self.consume_char('[') {
            return Err(QueryParseError::ExpectedStringValue);
        }

        let mut values = Vec::new();

        loop {
            self.skip_whitespace();
            if self.consume_char(']') {
                return Ok(QueryValue::StringList(values));
            }

            let QueryValue::String(value) = self.parse_string_value()? else {
                unreachable!("parse_string_value returns a string value");
            };
            values.push(value);

            self.skip_whitespace();
            if self.consume_char(',') {
                continue;
            }

            if self.consume_char(']') {
                return Ok(QueryValue::StringList(values));
            }

            return Err(QueryParseError::ExpectedStringValue);
        }
    }

    fn parse_number_value(&mut self) -> Result<QueryValue, QueryParseError> {
        self.skip_whitespace();
        let start = self.position;

        while let Some(character) = self.peek_char() {
            if character.is_ascii_digit() {
                self.position += character.len_utf8();
            } else {
                break;
            }
        }

        if self.position == start {
            return Err(QueryParseError::ExpectedStringValue);
        }

        let value = self.input[start..self.position]
            .parse::<i64>()
            .map_err(|_| QueryParseError::ExpectedStringValue)?;
        Ok(QueryValue::Number(value))
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

    fn consume_keyword(&mut self, expected: &str) -> bool {
        self.skip_whitespace();

        if !self.input[self.position..].starts_with(expected) {
            return false;
        }

        let end = self.position + expected.len();
        let next_is_boundary = self.input[end..]
            .chars()
            .next()
            .map_or(true, |character| !is_identifier_character(character));

        if next_is_boundary {
            self.position = end;
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

fn is_identifier_character(character: char) -> bool {
    character.is_ascii_alphanumeric() || character == '_'
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

    #[test]
    fn title_ends_with_matches_suffix() {
        let expr = QueryExpr::parse(r#"title ends_with "ment""#).expect("query parses");
        let topic = Topic {
            id: TopicId("topic-payment".to_owned()),
            title: "Payment".to_owned(),
            note: None,
            children: Vec::new(),
        };

        assert!(expr.matches_topic(&topic, &TopicPath::root(), 0));
    }

    #[test]
    fn title_in_matches_list_member() {
        let expr = QueryExpr::parse(r#"title in ["Payment", "Other"]"#).expect("query parses");
        let topic = Topic {
            id: TopicId("topic-payment".to_owned()),
            title: "Payment".to_owned(),
            note: None,
            children: Vec::new(),
        };

        assert!(expr.matches_topic(&topic, &TopicPath::root(), 0));
    }

    #[test]
    fn note_exists_matches_present_note() {
        let expr = QueryExpr::parse("note exists").expect("query parses");
        let topic = Topic {
            id: TopicId("topic-payment".to_owned()),
            title: "Payment".to_owned(),
            note: Some("Refund details".to_owned()),
            children: Vec::new(),
        };

        assert!(expr.matches_topic(&topic, &TopicPath::root(), 0));
    }

    #[test]
    fn depth_greater_than_matches_deeper_topic() {
        let expr = QueryExpr::parse("depth > 1").expect("query parses");
        let topic = Topic {
            id: TopicId("topic-payment".to_owned()),
            title: "Payment".to_owned(),
            note: None,
            children: Vec::new(),
        };

        assert!(expr.matches_topic(&topic, &TopicPath::root(), 2));
    }

    #[test]
    fn depth_equality_matches_same_depth_topic() {
        let expr = QueryExpr::parse("depth = 2").expect("query parses");
        let topic = Topic {
            id: TopicId("topic-payment".to_owned()),
            title: "Payment".to_owned(),
            note: None,
            children: Vec::new(),
        };

        assert!(expr.matches_topic(&topic, &TopicPath::root(), 2));
    }

    #[test]
    fn and_requires_both_sides_to_match() {
        let expr = QueryExpr::parse(r#"title = "Payment" and depth = 2"#).expect("query parses");
        let topic = Topic {
            id: TopicId("topic-payment".to_owned()),
            title: "Payment".to_owned(),
            note: None,
            children: Vec::new(),
        };

        assert!(expr.matches_topic(&topic, &TopicPath::root(), 2));
        assert!(!expr.matches_topic(&topic, &TopicPath::root(), 1));
    }

    #[test]
    fn or_accepts_either_side_matching() {
        let expr =
            QueryExpr::parse(r#"title = "Payment" or title = "Missing""#).expect("query parses");
        let topic = Topic {
            id: TopicId("topic-payment".to_owned()),
            title: "Payment".to_owned(),
            note: None,
            children: Vec::new(),
        };

        assert!(expr.matches_topic(&topic, &TopicPath::root(), 0));
    }

    #[test]
    fn not_inverts_matching_result() {
        let expr = QueryExpr::parse(r#"not title = "Payment""#).expect("query parses");
        let topic = Topic {
            id: TopicId("topic-payment".to_owned()),
            title: "Payment".to_owned(),
            note: None,
            children: Vec::new(),
        };

        assert!(!expr.matches_topic(&topic, &TopicPath::root(), 0));
    }

    #[test]
    fn parentheses_group_nested_expression() {
        let expr = QueryExpr::parse(r#"title = "Payment" and (depth = 1 or depth = 2)"#)
            .expect("query parses");
        let topic = Topic {
            id: TopicId("topic-payment".to_owned()),
            title: "Payment".to_owned(),
            note: None,
            children: Vec::new(),
        };

        assert!(expr.matches_topic(&topic, &TopicPath::root(), 2));
    }

    #[test]
    fn depth_greater_or_equal_matches_same_depth_topic() {
        let expr = QueryExpr::parse("depth >= 2").expect("query parses");
        let topic = Topic {
            id: TopicId("topic-payment".to_owned()),
            title: "Payment".to_owned(),
            note: None,
            children: Vec::new(),
        };

        assert!(expr.matches_topic(&topic, &TopicPath::root(), 2));
    }

    #[test]
    fn depth_less_than_matches_shallower_topic() {
        let expr = QueryExpr::parse("depth < 1").expect("query parses");
        let topic = Topic {
            id: TopicId("topic-root".to_owned()),
            title: "Roadmap".to_owned(),
            note: None,
            children: Vec::new(),
        };

        assert!(expr.matches_topic(&topic, &TopicPath::root(), 0));
    }

    #[test]
    fn depth_less_or_equal_matches_same_depth_topic() {
        let expr = QueryExpr::parse("depth <= 0").expect("query parses");
        let topic = Topic {
            id: TopicId("topic-root".to_owned()),
            title: "Roadmap".to_owned(),
            note: None,
            children: Vec::new(),
        };

        assert!(expr.matches_topic(&topic, &TopicPath::root(), 0));
    }
}
