//! Validate without contacting a server; hand off only the exact reviewed URL.
use serde::Serialize;
use std::time::{Duration, Instant};
use tauri::Url;

const INVALID: &str = "このURLは開けません。HTTP／HTTPSの接続先を確認してください。";
const EXPIRED: &str = "確認の有効期限が切れました。リンクを押し直してください。";
const MAX_URL: usize = 8192;
#[derive(Clone, Debug, Serialize)]
pub struct Review {
    pub ticket: String,
    pub url: String,
    pub host: String,
    pub insecure: bool,
}
struct Pending {
    review: Review,
    expires: Instant,
}
#[derive(Default)]
pub struct ExternalLinks {
    next: u64,
    pending: Option<Pending>,
}
fn hidden(c: char) -> bool {
    c.is_control()
        || matches!(c, '\u{061c}' | '\u{200b}'..='\u{200f}' | '\u{202a}'..='\u{202e}' | '\u{2060}'..='\u{206f}' | '\u{feff}')
}
pub fn validate(input: &str) -> Result<Review, String> {
    if input.len() > MAX_URL
        || input.chars().any(|c| c.is_whitespace() || hidden(c))
        || input.contains('\\')
    {
        return Err(INVALID.into());
    }
    let (scheme, rest) = input.split_once("://").ok_or(INVALID)?;
    if !scheme.eq_ignore_ascii_case("https") && !scheme.eq_ignore_ascii_case("http") {
        return Err(INVALID.into());
    }
    if rest.split(['/', '?', '#']).next().unwrap_or("").is_empty() {
        return Err(INVALID.into());
    }
    if rest
        .split(['/', '?', '#'])
        .next()
        .unwrap_or("")
        .contains('@')
    {
        return Err(INVALID.into());
    }
    let bytes = input.as_bytes();
    for (i, byte) in bytes.iter().enumerate() {
        if *byte == b'%'
            && (i + 2 >= bytes.len()
                || !bytes[i + 1].is_ascii_hexdigit()
                || !bytes[i + 2].is_ascii_hexdigit())
        {
            return Err(INVALID.into());
        }
    }
    let decoded = percent_encoding::percent_decode_str(input)
        .decode_utf8()
        .map_err(|_| INVALID)?;
    if decoded.chars().any(hidden) {
        return Err(INVALID.into());
    }
    let parsed = Url::parse(input).map_err(|_| INVALID)?;
    if !matches!(parsed.scheme(), "http" | "https")
        || !parsed.username().is_empty()
        || parsed.password().is_some()
    {
        return Err(INVALID.into());
    }
    let host = parsed
        .host_str()
        .filter(|host| !host.is_empty())
        .ok_or(INVALID)?
        .to_owned();
    let url = parsed.to_string();
    if url.len() > MAX_URL {
        return Err(INVALID.into());
    }
    Ok(Review {
        ticket: String::new(),
        url,
        host,
        insecure: parsed.scheme() == "http",
    })
}
impl ExternalLinks {
    pub fn prepare(&mut self, input: &str) -> Result<Review, String> {
        // Even an invalid replacement invalidates an earlier confirmation.
        self.pending = None;
        let mut review = validate(input)?;
        self.next = self.next.checked_add(1).ok_or(INVALID)?;
        review.ticket = self.next.to_string();
        self.pending = Some(Pending {
            review: review.clone(),
            expires: Instant::now() + Duration::from_secs(120),
        });
        Ok(review)
    }
    pub fn cancel(&mut self, ticket: &str) {
        if self
            .pending
            .as_ref()
            .is_some_and(|p| p.review.ticket == ticket)
        {
            self.pending = None;
        }
    }
    pub fn take_confirmed(&mut self, ticket: &str) -> Result<String, String> {
        if self
            .pending
            .as_ref()
            .is_none_or(|p| p.review.ticket != ticket)
        {
            return Err(EXPIRED.into());
        }
        let pending = self.pending.take().ok_or(EXPIRED)?;
        if Instant::now() >= pending.expires {
            return Err(EXPIRED.into());
        }
        Ok(pending.review.url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn displays_the_exact_canonical_destination() {
        let r = validate("HTTPS://例え.テスト:8443/資料?q=a%20b&x=1#節").unwrap();
        assert_eq!(r.host, "xn--r8jz45g.xn--zckzah");
        assert_eq!(
            r.url,
            "https://xn--r8jz45g.xn--zckzah:8443/%E8%B3%87%E6%96%99?q=a%20b&x=1#%E7%AF%80"
        );
        assert!(!r.insecure);
        assert!(validate("http://127.0.0.1:8080/a").unwrap().insecure);
        assert!(validate("https://[::1]/").is_ok());
        assert!(validate("https://example.com/?x=%22%26%25").is_ok());
    }
    #[test]
    fn rejects_non_web_ambiguous_and_hidden_destinations() {
        for value in [
            "javascript:alert(1)",
            "file:///tmp/a",
            "mailto:a@example.com",
            "data:text/html,x",
            "//example.com",
            "https:example.com",
            "https:///example.com",
            "https://",
            "https://user:pw@example.com",
            "https://good.com@evil.com",
            "https://@example.com",
            " https://example.com",
            "https://example.com/a b",
            "https://example.com/\n",
            "https://example.com/%0a",
            "https://example.com/%00",
            "https://example.com/%E2%80%AEevil",
            "https://example.com/\u{202e}evil",
            "https://example.com\\evil",
            "https://example.com/%ZZ",
            "https://example.com/%ff",
        ] {
            assert!(validate(value).is_err(), "accepted {value:?}");
        }
        assert!(validate(&format!("https://example.com/{}", "x".repeat(MAX_URL))).is_err());
    }
    #[test]
    fn approval_is_exact_single_use_cancelable_and_expiring() {
        let mut links = ExternalLinks::default();
        let a = links.prepare("https://example.com/a").unwrap();
        let b = links.prepare("https://example.com/b").unwrap();
        assert!(links.take_confirmed(&a.ticket).is_err());
        assert_eq!(links.take_confirmed(&b.ticket).unwrap(), b.url);
        assert!(links.take_confirmed(&b.ticket).is_err());
        let c = links.prepare("https://example.com/c").unwrap();
        links.cancel(&c.ticket);
        assert!(links.take_confirmed(&c.ticket).is_err());
        let d = links.prepare("https://example.com/d").unwrap();
        links.pending.as_mut().unwrap().expires = Instant::now() - Duration::from_secs(1);
        assert!(links.take_confirmed(&d.ticket).is_err());
        let e = links.prepare("https://example.com/e").unwrap();
        assert!(links.prepare("file:///no").is_err());
        assert!(links.take_confirmed(&e.ticket).is_err());
    }
}
