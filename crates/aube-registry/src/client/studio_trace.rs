//! Optional Studio correlation for Pkgst traffic. Never a credential or cache key.
use reqwest::{RequestBuilder, Url};

pub(super) fn pkgst_origin(url: &Url) -> bool {
    url.scheme() == "https"
        && url.host_str() == Some("npm.pkg.st")
        && url.port_or_known_default() == Some(443)
}

fn session_id(value: Option<&str>) -> Option<&str> {
    value.filter(|s| {
        !s.is_empty()
            && s.len() <= 128
            && s.bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
    })
}

pub(super) trait StudioTrace {
    fn studio_trace(self, url: &str) -> Self;
}

impl StudioTrace for RequestBuilder {
    fn studio_trace(self, url: &str) -> Self {
        let value = std::env::var("ELIDE_STUDIO_SESSION_ID").ok();
        attach(self, url, value.as_deref())
    }
}

fn attach(request: RequestBuilder, url: &str, value: Option<&str>) -> RequestBuilder {
    if Url::parse(url).is_ok_and(|url| pkgst_origin(&url))
        && let Some(id) = session_id(value) {
            return request.header("x-elide-studio-session-id", id);
        }
    request
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_metadata_is_bounded_and_only_reaches_the_pkgst_origin() {
        let client = reqwest::Client::new();
        for (url, expected) in [
            ("https://npm.pkg.st/package", true),
            ("https://npm.pkg.st:443/package", true),
            ("http://npm.pkg.st/package", false),
            ("https://npm.pkg.st:8443/package", false),
            ("https://npm.pkg.st.evil.test/package", false),
            ("https://registry.npmjs.org/package", false),
        ] {
            let request = attach(client.get(url), url, Some("studio-session_1"))
                .build()
                .unwrap();
            assert_eq!(
                request.headers().contains_key("x-elide-studio-session-id"),
                expected
            );
        }
        for value in [
            None,
            Some(""),
            Some("bad\r\nheader"),
            Some("bad/value"),
            Some("☃"),
        ] {
            let url = "https://npm.pkg.st/package";
            assert!(
                !attach(client.get(url), url, value)
                    .build()
                    .unwrap()
                    .headers()
                    .contains_key("x-elide-studio-session-id")
            );
        }
        assert!(session_id(Some(&"a".repeat(129))).is_none());
    }
}
