//! Matches URL paths against route patterns without owning any routing
//! table, dispatch logic, or I/O. Every function here takes its inputs as
//! arguments and returns a value - nothing is read from or written to
//! shared state, so a caller can build whatever router shape they want
//! (a Vec of routes, a trie, a HashMap) on top of these primitives and
//! test that shape independently of the matching logic itself.

mod pattern;
mod table;

pub use pattern::{parse, split_path, PatternError, Segment};
pub use table::{find_match, Route};

/// Extracted parameter names and values, in the order they appear in the
/// pattern. A `Vec` rather than a `HashMap` because route patterns rarely
/// have more than a handful of params and callers usually want the order
/// preserved for error messages or logging.
pub type Params = Vec<(String, String)>;

/// Matches a path against an already-parsed pattern, returning the
/// extracted params on success.
///
/// Static segments must match exactly. Param segments capture exactly one
/// path component. A wildcard segment, if present, must be the last
/// pattern segment and captures every remaining path component joined
/// with '/', including zero of them (an empty string).
pub fn match_path(pattern: &[Segment], path: &str) -> Option<Params> {
    let path_segments = split_path(path);
    let mut params = Params::new();
    let mut cursor = 0;

    for segment in pattern {
        match segment {
            Segment::Wildcard(name) => {
                let rest = &path_segments[cursor..];
                params.push((name.clone(), rest.join("/")));
                cursor = path_segments.len();
            }
            Segment::Param(name) => {
                let value = path_segments.get(cursor)?;
                params.push((name.clone(), value.to_string()));
                cursor += 1;
            }
            Segment::Static(expected) => {
                let value = path_segments.get(cursor)?;
                if value != expected {
                    return None;
                }
                cursor += 1;
            }
        }
    }

    if cursor < path_segments.len() {
        return None;
    }

    Some(params)
}

/// Parses `pattern` and matches `path` against it in one call. Prefer
/// [`parse`] plus [`match_path`] when the same pattern will be matched
/// against many paths, so the parse cost is paid only once.
pub fn matches(pattern: &str, path: &str) -> Result<Option<Params>, PatternError> {
    let segments = parse(pattern)?;
    Ok(match_path(&segments, path))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_static_route() {
        let segments = parse("/health").unwrap();
        assert_eq!(match_path(&segments, "/health"), Some(vec![]));
        assert_eq!(match_path(&segments, "/health/check"), None);
    }

    #[test]
    fn captures_params() {
        let segments = parse("/users/:id/posts/:post_id").unwrap();
        assert_eq!(
            match_path(&segments, "/users/42/posts/7"),
            Some(vec![("id".to_string(), "42".to_string()), ("post_id".to_string(), "7".to_string())])
        );
    }

    #[test]
    fn wildcard_captures_rest_including_empty() {
        let segments = parse("/files/*path").unwrap();
        assert_eq!(
            match_path(&segments, "/files/a/b/c"),
            Some(vec![("path".to_string(), "a/b/c".to_string())])
        );
        assert_eq!(match_path(&segments, "/files"), Some(vec![("path".to_string(), String::new())]));
    }

    #[test]
    fn root_pattern_matches_only_root() {
        let segments = parse("/").unwrap();
        assert_eq!(match_path(&segments, "/"), Some(vec![]));
        assert_eq!(match_path(&segments, "/anything"), None);
    }

    #[test]
    fn matches_convenience_function() {
        assert_eq!(
            matches("/users/:id", "/users/9").unwrap(),
            Some(vec![("id".to_string(), "9".to_string())])
        );
    }
}
