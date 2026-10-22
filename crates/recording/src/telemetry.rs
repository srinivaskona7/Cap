use serde::{Deserialize, Serialize};
use std::{
    path::Path,
    sync::{Arc, Mutex},
};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct TelemetryRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl TelemetryRect {
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x: x.clamp(0.0, 1.0),
            y: y.clamp(0.0, 1.0),
            width: width.clamp(0.01, 1.0),
            height: height.clamp(0.01, 1.0),
        }
    }

    pub fn center(&self) -> (f64, f64) {
        (
            (self.x + self.width * 0.5).clamp(0.0, 1.0),
            (self.y + self.height * 0.5).clamp(0.0, 1.0),
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SemanticElementKind {
    TerminalError,
    ModalDialog,
    IdeBreakpoint,
    NotificationBanner,
    FocusedInput,
    MobileElement,
}

impl SemanticElementKind {
    pub fn suggested_zoom(self) -> f64 {
        match self {
            Self::TerminalError => 1.85,
            Self::ModalDialog => 1.65,
            Self::IdeBreakpoint => 1.75,
            Self::NotificationBanner => 1.55,
            Self::FocusedInput => 1.5,
            Self::MobileElement => 1.6,
        }
    }

    pub fn priority(self) -> u8 {
        match self {
            Self::TerminalError => 100,
            Self::ModalDialog => 90,
            Self::IdeBreakpoint => 85,
            Self::NotificationBanner => 75,
            Self::MobileElement => 70,
            Self::FocusedInput => 60,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SemanticTelemetryPacket {
    pub ts_ms: u64,
    pub element_type: SemanticElementKind,
    pub bounds: TelemetryRect,
    pub text_summary: String,
}

#[derive(Clone, Default)]
pub struct SemanticTelemetryRecorder {
    packets: Arc<Mutex<Vec<SemanticTelemetryPacket>>>,
}

impl SemanticTelemetryRecorder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_packet(&self, packet: SemanticTelemetryPacket) {
        if let Ok(mut guard) = self.packets.lock() {
            guard.push(packet);
        }
    }

    pub fn snapshot(&self) -> Vec<SemanticTelemetryPacket> {
        self.packets.lock().map(|g| g.clone()).unwrap_or_default()
    }

    pub fn save_to_project(&self, project_dir: &Path) -> std::io::Result<()> {
        let packets = self.snapshot();
        let json = serde_json::to_string_pretty(&packets).map_err(std::io::Error::other)?;
        std::fs::write(project_dir.join("semantic-telemetry.json"), json)
    }

    pub fn load_from_project(project_dir: &Path) -> Vec<SemanticTelemetryPacket> {
        let path = project_dir.join("semantic-telemetry.json");
        let Ok(raw) = std::fs::read_to_string(path) else {
            return Vec::new();
        };
        serde_json::from_str(&raw).unwrap_or_default()
    }
}

pub fn classify_accessibility_event(role: &str, title_or_value: &str) -> SemanticElementKind {
    let lower_role = role.to_ascii_lowercase();
    let lower_text = title_or_value.to_ascii_lowercase();
    if lower_text.contains("error")
        || lower_text.contains("panic")
        || lower_text.contains("exception")
        || lower_text.contains("failed")
    {
        SemanticElementKind::TerminalError
    } else if lower_role.contains("dialog")
        || lower_role.contains("sheet")
        || lower_role.contains("alert")
        || lower_text.contains("confirm")
    {
        SemanticElementKind::ModalDialog
    } else if lower_text.contains("breakpoint") || lower_role.contains("debugger") {
        SemanticElementKind::IdeBreakpoint
    } else if lower_role.contains("notification") || lower_role.contains("banner") {
        SemanticElementKind::NotificationBanner
    } else {
        SemanticElementKind::FocusedInput
    }
}

pub fn parse_android_accessibility_bounds(
    bounds_str: &str,
    screen_width: f64,
    screen_height: f64,
) -> Option<TelemetryRect> {
    let sw = screen_width.max(1.0);
    let sh = screen_height.max(1.0);
    let cleaned = bounds_str.trim();
    let (first, second) = cleaned.split_once("][")?;
    let first = first.trim_start_matches('[');
    let second = second.trim_end_matches(']');
    let (x1s, y1s) = first.split_once(',')?;
    let (x2s, y2s) = second.split_once(',')?;
    let x1: f64 = x1s.parse().ok()?;
    let y1: f64 = y1s.parse().ok()?;
    let x2: f64 = x2s.parse().ok()?;
    let y2: f64 = y2s.parse().ok()?;
    if x2 <= x1 || y2 <= y1 {
        return None;
    }
    Some(TelemetryRect::new(
        x1 / sw,
        y1 / sh,
        (x2 - x1) / sw,
        (y2 - y1) / sh,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_terminal_errors_and_dialogs_accurately() {
        assert_eq!(
            classify_accessibility_event("AXStaticText", "error[E0599]: method not found"),
            SemanticElementKind::TerminalError
        );
        assert_eq!(
            classify_accessibility_event("AXDialog", "Confirm Merge Pull Request"),
            SemanticElementKind::ModalDialog
        );
    }

    #[test]
    fn parses_android_accessibility_node_bounds_into_normalized_rect() {
        let rect =
            parse_android_accessibility_bounds("[108,480][972,960]", 1080.0, 2400.0).unwrap();
        assert!((rect.x - 0.1).abs() < 1e-6);
        assert!((rect.y - 0.2).abs() < 1e-6);
        let (cx, cy) = rect.center();
        assert!((cx - 0.5).abs() < 1e-6);
        assert!((cy - 0.3).abs() < 1e-6);
    }
}
