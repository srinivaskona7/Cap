use std::process::Command;

pub fn normalize_launch_url(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed.contains(char::is_whitespace) {
        return None;
    }
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") || trimmed.contains("://")
    {
        return Some(trimmed.to_string());
    }
    if trimmed.contains('.') || trimmed.starts_with("localhost") {
        return Some(format!("https://{trimmed}"));
    }
    None
}

fn find_adb() -> String {
    for candidate in [
        "/opt/homebrew/bin/adb",
        "/usr/local/bin/adb",
        "/Users/Shared/Android/sdk/platform-tools/adb",
    ] {
        if std::path::Path::new(candidate).exists() {
            return candidate.to_string();
        }
    }
    if let Some(home) = dirs::home_dir() {
        let sdk = home.join("Library/Android/sdk/platform-tools/adb");
        if sdk.exists() {
            return sdk.to_string_lossy().to_string();
        }
    }
    "adb".to_string()
}

pub fn sync_mac_clipboard_to_device(serial: &str, text: &str) -> bool {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return false;
    }
    if let Some(udid) = serial.strip_prefix("sim:") {
        let Ok(mut child) = Command::new("xcrun")
            .args(["simctl", "pbcopy", udid])
            .stdin(std::process::Stdio::piped())
            .spawn()
        else {
            return false;
        };
        if let Some(mut stdin) = child.stdin.take() {
            let _ = std::io::Write::write_all(&mut stdin, trimmed.as_bytes());
        }
        return child.wait().map(|s| s.success()).unwrap_or(false);
    }
    let adb = find_adb();
    let escaped = trimmed.replace(' ', "%s").replace(['"', '\''], "");
    Command::new(adb)
        .args(["-s", serial, "shell", "input", "text", &escaped])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

pub fn pull_device_clipboard_to_mac(serial: &str) -> Option<String> {
    if let Some(udid) = serial.strip_prefix("sim:") {
        let out = Command::new("xcrun")
            .args(["simctl", "pbpaste", udid])
            .output()
            .ok()?;
        let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
        return (!s.is_empty()).then_some(s);
    }
    let adb = find_adb();
    let out = Command::new(adb)
        .args(["-s", serial, "shell", "cmd", "clipboard", "get-primary"])
        .output()
        .ok()?;
    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
    (!s.is_empty() && !s.contains("Error") && !s.contains("Unknown")).then_some(s)
}

pub fn open_url_on_mirrored_device(serial: &str, raw_url: &str) -> bool {
    let Some(url) = normalize_launch_url(raw_url) else {
        return false;
    };
    if let Some(udid) = serial.strip_prefix("sim:") {
        return Command::new("xcrun")
            .args(["simctl", "openurl", udid, &url])
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
    }
    let adb = find_adb();
    Command::new(adb)
        .args([
            "-s",
            serial,
            "shell",
            "am",
            "start",
            "-a",
            "android.intent.action.VIEW",
            "-d",
            &url,
        ])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_http_custom_scheme_and_bare_domains() {
        assert_eq!(
            normalize_launch_url("https://github.com/srinivaskona7/Cap"),
            Some("https://github.com/srinivaskona7/Cap".to_string())
        );
        assert_eq!(
            normalize_launch_url("sricap.dev/demo"),
            Some("https://sricap.dev/demo".to_string())
        );
        assert_eq!(
            normalize_launch_url("myapp://checkout/42"),
            Some("myapp://checkout/42".to_string())
        );
        assert_eq!(normalize_launch_url("not a url"), None);
    }
}
