#[derive(Debug)]
enum Segment {
    Static(String),
    Param(String),
    Wildcard(Option<String>),
}

#[derive(Debug)]
pub struct Route {
    pub name: String,
    segments: Vec<Segment>,
}

impl Route {
    pub fn parse(name: &str, pattern: &str) -> Result<Route, String> {
        let trimmed = pattern.trim().trim_matches('/');
        let mut segments = Vec::new();

        if !trimmed.is_empty() {
            for part in trimmed.split('/') {
                if part.is_empty() {
                    return Err(format!("pattern '{}' has an empty segment", pattern));
                }
                let segment = if let Some(rest) = part.strip_prefix(':') {
                    if rest.is_empty() {
                        return Err(format!("pattern '{}' has a nameless param", pattern));
                    }
                    Segment::Param(rest.to_string())
                } else if part == "*" {
                    Segment::Wildcard(None)
                } else if let Some(rest) = part.strip_prefix('*') {
                    Segment::Wildcard(Some(rest.to_string()))
                } else {
                    Segment::Static(part.to_string())
                };
                segments.push(segment);
            }
        }

        // a wildcard swallows everything after it, so it only makes sense
        // as the last segment; anything past it could never be reached.
        if let Some(pos) = segments
            .iter()
            .position(|s| matches!(s, Segment::Wildcard(_)))
        {
            if pos != segments.len() - 1 {
                return Err(format!(
                    "pattern '{}' has a wildcard before the end",
                    pattern
                ));
            }
        }

        Ok(Route {
            name: name.to_string(),
            segments,
        })
    }

    /// Returns the extracted params on a match, keyed in pattern order.
    pub fn matches(&self, path: &str) -> Option<Vec<(String, String)>> {
        let parts: Vec<&str> = path
            .trim_matches('/')
            .split('/')
            .filter(|s| !s.is_empty())
            .collect();

        let mut params = Vec::new();
        let mut i = 0;

        for segment in &self.segments {
            match segment {
                Segment::Wildcard(name) => {
                    if let Some(n) = name {
                        params.push((n.clone(), parts[i..].join("/")));
                    }
                    return Some(params);
                }
                Segment::Static(expected) => {
                    if parts.get(i) != Some(&expected.as_str()) {
                        return None;
                    }
                    i += 1;
                }
                Segment::Param(name) => match parts.get(i) {
                    Some(value) => {
                        params.push((name.clone(), value.to_string()));
                        i += 1;
                    }
                    None => return None,
                },
            }
        }

        if i == parts.len() {
            Some(params)
        } else {
            None
        }
    }
}

/// Pulls the path component out of a full URL or a bare path, dropping
/// scheme, host, query string and fragment.
pub fn extract_path(raw: &str) -> &str {
    let s = raw.trim();

    let after_authority = match s.find("://") {
        Some(scheme_end) => {
            let rest = &s[scheme_end + 3..];
            match rest.find('/') {
                Some(slash) => &rest[slash..],
                None => "/",
            }
        }
        None => s,
    };

    let end = after_authority
        .find(['?', '#'])
        .unwrap_or(after_authority.len());
    &after_authority[..end]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn static_route_matches_exactly() {
        let r = Route::parse("health", "/health").unwrap();
        assert!(r.matches("/health").is_some());
        assert!(r.matches("/health/check").is_none());
    }

    #[test]
    fn param_route_captures_values() {
        let r = Route::parse("user_posts", "/users/:id/posts/:post_id").unwrap();
        let params = r.matches("/users/42/posts/7").unwrap();
        assert_eq!(
            params,
            vec![
                ("id".to_string(), "42".to_string()),
                ("post_id".to_string(), "7".to_string()),
            ]
        );
    }

    #[test]
    fn wildcard_captures_remainder() {
        let r = Route::parse("assets", "/assets/*path").unwrap();
        let params = r.matches("/assets/css/site.css").unwrap();
        assert_eq!(params, vec![("path".to_string(), "css/site.css".to_string())]);
    }

    #[test]
    fn wildcard_before_end_is_rejected() {
        assert!(Route::parse("bad", "/a/*rest/b").is_err());
    }

    #[test]
    fn extract_path_strips_scheme_host_and_query() {
        assert_eq!(
            extract_path("https://example.com/users/1?verbose=true#top"),
            "/users/1"
        );
        assert_eq!(extract_path("/users/1"), "/users/1");
    }
}
