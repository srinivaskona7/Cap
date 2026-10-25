//! Dynamic IPTV-Org Playlist & Channel Engine for Cap Live TV
//!
//! Fetches, parses, and caches legal Free-to-Air live TV channels from
//! the official open-source `iptv-org/iptv` index by Country, Category, or custom M3U8.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct IptvChannel {
    pub id: String,
    pub name: String,
    pub logo: String,
    pub group: String,
    pub country: String,
    pub stream_url: String,
    pub resolution: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum IptvCategory {
    News,
    Sports,
    Documentary,
    Science,
    Music,
    Movies,
    Kids,
    Education,
    Weather,
}

impl IptvCategory {
    pub const ALL: [Self; 9] = [
        Self::News,
        Self::Sports,
        Self::Documentary,
        Self::Science,
        Self::Music,
        Self::Movies,
        Self::Kids,
        Self::Education,
        Self::Weather,
    ];

    pub fn slug(self) -> &'static str {
        match self {
            Self::News => "news",
            Self::Sports => "sports",
            Self::Documentary => "documentary",
            Self::Science => "science",
            Self::Music => "music",
            Self::Movies => "movies",
            Self::Kids => "kids",
            Self::Education => "education",
            Self::Weather => "weather",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::News => "📰 News",
            Self::Sports => "⚽ Sports",
            Self::Documentary => "🎬 Documentary",
            Self::Science => "🔬 Science",
            Self::Music => "🎵 Music",
            Self::Movies => "🍿 Movies",
            Self::Kids => "🧸 Kids",
            Self::Education => "📚 Education",
            Self::Weather => "⛈️ Weather",
        }
    }

    pub fn url(self) -> String {
        format!(
            "https://iptv-org.github.io/iptv/categories/{}.m3u",
            self.slug()
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum IptvLanguage {
    Telugu,
    Hindi,
    English,
    Tamil,
    Kannada,
    Malayalam,
    Spanish,
    Japanese,
    French,
    German,
}

impl IptvLanguage {
    pub const ALL: [Self; 10] = [
        Self::Telugu,
        Self::Hindi,
        Self::English,
        Self::Tamil,
        Self::Kannada,
        Self::Malayalam,
        Self::Spanish,
        Self::Japanese,
        Self::French,
        Self::German,
    ];

    pub fn code(self) -> &'static str {
        match self {
            Self::Telugu => "tel",
            Self::Hindi => "hin",
            Self::English => "eng",
            Self::Tamil => "tam",
            Self::Kannada => "kan",
            Self::Malayalam => "mal",
            Self::Spanish => "spa",
            Self::Japanese => "jpn",
            Self::French => "fra",
            Self::German => "deu",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Telugu => "తెలుగు (Telugu)",
            Self::Hindi => "हिन्दी (Hindi)",
            Self::English => "English",
            Self::Tamil => "தமிழ் (Tamil)",
            Self::Kannada => "ಕನ್ನಡ (Kannada)",
            Self::Malayalam => "മലയാളം (Malayalam)",
            Self::Spanish => "Español",
            Self::Japanese => "日本語 (Japanese)",
            Self::French => "Français",
            Self::German => "Deutsch",
        }
    }

    pub fn url(self) -> String {
        format!(
            "https://iptv-org.github.io/iptv/languages/{}.m3u",
            self.code()
        )
    }
}

pub struct IptvEngine;

impl IptvEngine {
    /// Cache directory for downloaded IPTV playlists (~/.cache/cap/iptv)
    pub fn cache_dir() -> PathBuf {
        dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("cap")
            .join("iptv")
    }

    /// Parses an M3U/M3U8 string into structured `IptvChannel` items.
    pub fn parse_m3u(content: &str) -> Vec<IptvChannel> {
        let mut channels = Vec::new();
        let mut current_id = String::new();
        let mut current_logo = String::new();
        let mut current_group = String::new();
        let mut current_name = String::new();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            if line.starts_with("#EXTINF:") {
                // Parse attributes
                current_id = Self::extract_attr(line, "tvg-id").unwrap_or_default();
                current_logo = Self::extract_attr(line, "tvg-logo").unwrap_or_default();
                current_group = Self::extract_attr(line, "group-title").unwrap_or_default();

                // Channel name is after the comma
                if let Some(comma_idx) = line.rfind(',') {
                    current_name = line[comma_idx + 1..].trim().to_string();
                } else {
                    current_name = "Live Stream".to_string();
                }
            } else if !line.starts_with('#') {
                if !current_name.is_empty() {
                    let resolution = Self::detect_resolution(&current_name);
                    let country = Self::detect_country(&current_id);

                    channels.push(IptvChannel {
                        id: current_id.clone(),
                        name: current_name.clone(),
                        logo: current_logo.clone(),
                        group: current_group.clone(),
                        country,
                        stream_url: line.to_string(),
                        resolution,
                    });

                    current_name.clear();
                    current_id.clear();
                    current_logo.clear();
                    current_group.clear();
                }
            }
        }

        channels
    }

    fn extract_attr(line: &str, attr: &str) -> Option<String> {
        let pattern = format!("{}=\"", attr);
        if let Some(start) = line.find(&pattern) {
            let val_start = start + pattern.len();
            if let Some(end) = line[val_start..].find('"') {
                return Some(line[val_start..val_start + end].to_string());
            }
        }
        None
    }

    fn detect_resolution(name: &str) -> String {
        if name.contains("4K") || name.contains("2160p") || name.contains("UHD") {
            "4K UHD".to_string()
        } else if name.contains("1080p") || name.contains("FHD") {
            "1080p HD".to_string()
        } else if name.contains("720p") || name.contains("HD") {
            "720p HD".to_string()
        } else if name.contains("576p") || name.contains("SD") {
            "576p SD".to_string()
        } else {
            "HD".to_string()
        }
    }

    fn detect_country(tvg_id: &str) -> String {
        // e.g. "NASA.us@SD" -> "US", "BBC.uk@HD" -> "UK"
        if let Some(at_idx) = tvg_id.find('@') {
            let prefix = &tvg_id[..at_idx];
            if let Some(dot_idx) = prefix.rfind('.') {
                return prefix[dot_idx + 1..].to_uppercase();
            }
        }
        if let Some(dot_idx) = tvg_id.rfind('.') {
            return tvg_id[dot_idx + 1..].to_uppercase();
        }
        "GLOBAL".to_string()
    }

    /// Fetches playlist incrementally: returns cached version if fresh (< 24h),
    /// or fetches from remote and caches to disk.
    pub async fn fetch_playlist_incremental(
        url: &str,
        cache_key: &str,
    ) -> Result<Vec<IptvChannel>, String> {
        let dir = Self::cache_dir();
        let _ = std::fs::create_dir_all(&dir);
        let cache_file = dir.join(format!("{}.m3u", cache_key));

        // Check if cached file exists and is less than 24 hours old
        if let Ok(metadata) = std::fs::metadata(&cache_file) {
            if let Ok(modified) = metadata.modified() {
                if let Ok(elapsed) = modified.elapsed() {
                    if elapsed.as_secs() < 86400 {
                        if let Ok(cached_body) = std::fs::read_to_string(&cache_file) {
                            return Ok(Self::parse_m3u(&cached_body));
                        }
                    }
                }
            }
        }

        // Fetch remotely
        let channels = Self::fetch_playlist_direct(url).await?;

        // Save fresh content to cache
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .map_err(|e| e.to_string())?;

        if let Ok(res) = client
            .get(url)
            .header("User-Agent", "CapStudioLiveTV/0.6.8")
            .send()
            .await
        {
            if let Ok(body) = res.text().await {
                let _ = std::fs::write(&cache_file, body);
            }
        }

        Ok(channels)
    }

    /// Direct remote fetch without cache check
    pub async fn fetch_playlist_direct(url: &str) -> Result<Vec<IptvChannel>, String> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .map_err(|e| e.to_string())?;

        let res = client
            .get(url)
            .header("User-Agent", "CapStudioLiveTV/0.6.8")
            .send()
            .await
            .map_err(|e| format!("Failed to fetch IPTV playlist: {e}"))?;

        if !res.status().is_success() {
            return Err(format!("HTTP Error {}", res.status()));
        }

        let body = res.text().await.map_err(|e| e.to_string())?;
        Ok(Self::parse_m3u(&body))
    }

    /// Fetch channels by language with incremental caching
    pub async fn fetch_by_language(lang: IptvLanguage) -> Result<Vec<IptvChannel>, String> {
        Self::fetch_playlist_incremental(&lang.url(), &format!("lang_{}", lang.code())).await
    }

    /// Fetch channels by category with incremental caching
    pub async fn fetch_by_category(cat: IptvCategory) -> Result<Vec<IptvChannel>, String> {
        Self::fetch_playlist_incremental(&cat.url(), &format!("cat_{}", cat.slug())).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_m3u_stream() {
        let sample = r#"#EXTM3U x-tvg-url="https://epg.test.xml"
#EXTINF:-1 tvg-id="NASA.us@UHD" tvg-logo="https://nasa.gov/logo.png" group-title="Science",NASA TV (4K)
https://ntv1.akamaized.net/live/master.m3u8
#EXTINF:-1 tvg-id="Bloomberg.us@HD" tvg-logo="https://bloomberg.com/logo.png" group-title="News",Bloomberg Global Live (1080p)
https://live.bloomberg.com/live.m3u8
"#;
        let channels = IptvEngine::parse_m3u(sample);
        assert_eq!(channels.len(), 2);
        assert_eq!(channels[0].name, "NASA TV (4K)");
        assert_eq!(channels[0].country, "US");
        assert_eq!(channels[0].resolution, "4K UHD");
        assert_eq!(
            channels[0].stream_url,
            "https://ntv1.akamaized.net/live/master.m3u8"
        );

        assert_eq!(channels[1].name, "Bloomberg Global Live (1080p)");
        assert_eq!(channels[1].resolution, "1080p HD");
    }

    #[test]
    fn test_language_urls() {
        assert_eq!(IptvLanguage::Telugu.code(), "tel");
        assert_eq!(
            IptvLanguage::Telugu.url(),
            "https://iptv-org.github.io/iptv/languages/tel.m3u"
        );
        assert_eq!(IptvLanguage::Hindi.code(), "hin");
        assert_eq!(
            IptvLanguage::Hindi.url(),
            "https://iptv-org.github.io/iptv/languages/hin.m3u"
        );
    }
}
