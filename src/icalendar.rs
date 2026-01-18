use crate::client::AnyListClient;
use crate::error::{AnyListError, Result};
use prost::Message;
use regex::Regex;
use serde_derive::{Deserialize, Serialize};

/// iCalendar export info for meal planning calendar
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ICalendarInfo {
    pub(crate) enabled: bool,
    pub(crate) url: Option<String>,
    pub(crate) token: Option<String>,
}

impl ICalendarInfo {
    /// Whether iCalendar export is enabled
    pub fn enabled(&self) -> bool {
        self.enabled
    }

    /// The iCalendar URL for subscribing to the meal planning calendar
    pub fn url(&self) -> Option<&str> {
        self.url.as_deref()
    }

    /// The iCalendar token (used to construct the URL)
    pub fn token(&self) -> Option<&str> {
        self.token.as_deref()
    }
}

/// Request message for enabling iCalendar
#[derive(Clone, PartialEq, Message)]
struct ICalendarRequest {
    #[prost(bool, optional, tag = "1")]
    enabled: Option<bool>,
}

impl AnyListClient {
    /// Enable iCalendar export for meal planning calendar
    ///
    /// This enables the public iCalendar URL that can be subscribed to
    /// by external calendar applications like Home Assistant, Google Calendar, etc.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anylist_rs::AnyListClient;
    /// # #[tokio::main]
    /// # async fn main() {
    /// let client = AnyListClient::login("user@example.com", "password")
    ///     .await
    ///     .expect("Failed to authenticate");
    ///
    /// let info = client.enable_icalendar().await.expect("Failed to enable");
    /// if let Some(url) = info.url() {
    ///     println!("iCalendar URL: {}", url);
    /// }
    /// # }
    /// ```
    pub async fn enable_icalendar(&self) -> Result<ICalendarInfo> {
        let request = ICalendarRequest {
            enabled: Some(true),
        };

        let mut body = Vec::new();
        request.encode(&mut body).map_err(|e| {
            AnyListError::ProtobufError(format!("Failed to encode iCalendar request: {}", e))
        })?;

        let response = self
            .post_multipart(
                "/data/meal-planning-calendar/set-icalendar-enabled",
                "icalendar_request",
                body,
            )
            .await?;

        // Extract token from response - it's a 32-char hex string
        let token = extract_icalendar_token(&response);

        Ok(ICalendarInfo {
            enabled: true,
            url: token.as_ref().map(|t| format!("https://icalendar.anylist.com/{}.ics", t)),
            token,
        })
    }

    /// Disable iCalendar export for meal planning calendar
    pub async fn disable_icalendar(&self) -> Result<()> {
        let request = ICalendarRequest {
            enabled: Some(false),
        };

        let mut body = Vec::new();
        request.encode(&mut body).map_err(|e| {
            AnyListError::ProtobufError(format!("Failed to encode iCalendar request: {}", e))
        })?;

        self.post_multipart(
            "/data/meal-planning-calendar/set-icalendar-enabled",
            "icalendar_request",
            body,
        )
        .await?;

        Ok(())
    }

    /// Get the iCalendar URL if already enabled
    ///
    /// This fetches account info and extracts the iCalendar token if available.
    pub async fn get_icalendar_url(&self) -> Result<Option<String>> {
        let response = self.post("data/account/info", vec![]).await?;

        let token = extract_icalendar_token(&response);

        Ok(token.map(|t| format!("https://icalendar.anylist.com/{}.ics", t)))
    }
}

/// Extract iCalendar token from response bytes
///
/// The token is a 32-character hex string (UUID without dashes)
fn extract_icalendar_token(data: &[u8]) -> Option<String> {
    // Convert to string, looking for 32-char hex pattern
    let text = String::from_utf8_lossy(data);

    // Pattern: 32 hex characters that look like a UUID without dashes
    let re = Regex::new(r"[a-f0-9]{32}").ok()?;

    // Find matches and return the one that looks like an iCalendar token
    // (typically the last one or one that's not a user ID)
    for cap in re.find_iter(&text) {
        let token = cap.as_str();
        // Skip if it looks like a known field (user IDs typically start with specific patterns)
        // The iCalendar token is usually different
        if !token.starts_with("9540") {
            // Skip user ID patterns
            return Some(token.to_string());
        }
    }

    // Fallback: return the last match if any
    re.find_iter(&text).last().map(|m| m.as_str().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_token() {
        let data = b"some data b2349cc3e7bc49ef85226d199916d1b6 more data";
        let token = extract_icalendar_token(data);
        assert_eq!(token, Some("b2349cc3e7bc49ef85226d199916d1b6".to_string()));
    }

    #[test]
    fn test_extract_token_skips_user_id() {
        // Simulated response with user ID and calendar token
        let data = b"9540e2d686024900b0c28f4498db8165 other b2349cc3e7bc49ef85226d199916d1b6";
        let token = extract_icalendar_token(data);
        assert_eq!(token, Some("b2349cc3e7bc49ef85226d199916d1b6".to_string()));
    }
}
