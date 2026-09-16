use std::fmt;

/// A single piece of a parsed route pattern.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Segment {
    /// A literal segment that must match exactly, e.g. "users".
    Static(String),
    /// A named segment that matches any single path component, e.g. ":id".
    Param(String),
    /// A named segment that matches everything remaining, e.g. "*rest".
    /// Only valid as the final segment of a pattern.
    Wildcard(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PatternError {
    EmptyParamName,
    EmptyWildcardName,
    InvalidName(String),
    WildcardNotLast,
}

impl fmt::Display for PatternError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PatternError::EmptyParamName => write!(f, "param segment has no name after ':'"),
            PatternError::EmptyWildcardName => {
                write!(f, "wildcard segment has no name after '*'")
            }
            PatternError::InvalidName(name) => {
                write!(f, "segment name {name:?} must be alphanumeric or '_'")
            }
            PatternError::WildcardNotLast => {
                write!(f, "wildcard segment must be the last segment in the pattern")
            }
        }
    }
}

impl std::error::Error for PatternError {}

/// Splits a path or pattern string into its non-empty components.
///
/// Leading, trailing, and repeated slashes are ignored so that "/a/b/",
/// "a/b", and "a//b" all split the same way.
pub fn split_path(path: &str) -> Vec<&str> {
    path.split('/').filter(|segment| !segment.is_empty()).collect()
}

/// Parses a route pattern such as "/users/:id/posts/*rest" into segments.
///
/// This never inspects any input beyond the string passed in, and never
/// allocates state outside of its return value.
pub fn parse(pattern: &str) -> Result<Vec<Segment>, PatternError> {
    let raw = split_path(pattern);
    if raw.is_empty() {
        return Ok(Vec::new());
    }

    let last_index = raw.len() - 1;
    let mut segments = Vec::with_capacity(raw.len());

    for (index, piece) in raw.iter().enumerate() {
        let segment = if let Some(name) = piece.strip_prefix(':') {
            if name.is_empty() {
                return Err(PatternError::EmptyParamName);
            }
            validate_name(name)?;
            Segment::Param(name.to_string())
        } else if let Some(name) = piece.strip_prefix('*') {
            if name.is_empty() {
                return Err(PatternError::EmptyWildcardName);
            }
            validate_name(name)?;
            if index != last_index {
                return Err(PatternError::WildcardNotLast);
            }
            Segment::Wildcard(name.to_string())
        } else {
            Segment::Static(piece.to_string())
        };
        segments.push(segment);
    }

    Ok(segments)
}

fn validate_name(name: &str) -> Result<(), PatternError> {
    if name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        Ok(())
    } else {
        Err(PatternError::InvalidName(name.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_static_segments() {
        assert_eq!(
            parse("/users/all").unwrap(),
            vec![Segment::Static("users".into()), Segment::Static("all".into())]
        );
    }

    #[test]
    fn parses_param_and_wildcard() {
        assert_eq!(
            parse("/users/:id/*rest").unwrap(),
            vec![
                Segment::Static("users".into()),
                Segment::Param("id".into()),
                Segment::Wildcard("rest".into()),
            ]
        );
    }

    #[test]
    fn root_pattern_has_no_segments() {
        assert_eq!(parse("/").unwrap(), Vec::new());
        assert_eq!(parse("").unwrap(), Vec::new());
    }

    #[test]
    fn rejects_wildcard_not_last() {
        assert_eq!(parse("/*rest/more"), Err(PatternError::WildcardNotLast));
    }

    #[test]
    fn rejects_empty_names() {
        assert_eq!(parse("/users/:"), Err(PatternError::EmptyParamName));
        assert_eq!(parse("/users/*"), Err(PatternError::EmptyWildcardName));
    }

    #[test]
    fn rejects_invalid_names() {
        assert_eq!(
            parse("/users/:user-id"),
            Err(PatternError::InvalidName("user-id".into()))
        );
    }
}
