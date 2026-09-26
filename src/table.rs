use crate::{match_path, Params, Segment};

/// A parsed pattern paired with whatever the caller wants to associate
/// with it - a handler id, a closure, an enum variant. This crate never
/// inspects `data`; it only carries it alongside the pattern so callers
/// can build a route table without re-parsing on every lookup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Route<T> {
    pub pattern: Vec<Segment>,
    pub data: T,
}

impl<T> Route<T> {
    pub fn new(pattern: Vec<Segment>, data: T) -> Self {
        Route { pattern, data }
    }
}

/// Matches `path` against `routes` in order, returning the first route
/// whose pattern matches along with the params extracted from it.
///
/// "Prioritized" means the order of `routes` is the order of preference:
/// this function does no reordering and applies no notion of specificity
/// on its own. If two patterns could both match the same path (e.g.
/// `/users/:id` and `/users/me`), put the one that should win first in
/// the slice.
pub fn find_match<'a, T>(routes: &'a [Route<T>], path: &str) -> Option<(&'a T, Params)> {
    for route in routes {
        if let Some(params) = match_path(&route.pattern, path) {
            return Some((&route.data, params));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse;

    #[test]
    fn returns_first_matching_route_in_order() {
        let routes = vec![
            Route::new(parse("/users/:id").unwrap(), "get_user"),
            Route::new(parse("/users/me").unwrap(), "get_current_user"),
        ];
        // "/users/me" matches both patterns, but the param route is
        // listed first, so it wins - order is the caller's priority.
        let (data, params) = find_match(&routes, "/users/me").unwrap();
        assert_eq!(*data, "get_user");
        assert_eq!(params, vec![("id".to_string(), "me".to_string())]);
    }

    #[test]
    fn more_specific_route_wins_when_listed_first() {
        let routes = vec![
            Route::new(parse("/users/me").unwrap(), "get_current_user"),
            Route::new(parse("/users/:id").unwrap(), "get_user"),
        ];
        let (data, params) = find_match(&routes, "/users/me").unwrap();
        assert_eq!(*data, "get_current_user");
        assert_eq!(params, Vec::new());
    }

    #[test]
    fn returns_none_when_no_route_matches() {
        let routes = vec![Route::new(parse("/health").unwrap(), "health")];
        assert!(find_match(&routes, "/other").is_none());
    }

    #[test]
    fn empty_table_never_matches() {
        let routes: Vec<Route<&str>> = Vec::new();
        assert!(find_match(&routes, "/anything").is_none());
    }
}
