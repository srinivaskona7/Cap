use std::{
    io::Write,
    net::TcpStream,
    process::Command,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use gpui::{
    App, AppContext as _, Bounds, Context, FontWeight, Hsla, InteractiveElement, IntoElement,
    KeyDownEvent, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, ParentElement, Render,
    SharedString, StatefulInteractiveElement, Styled, StyledImage, Task, Window, WindowBounds,
    WindowHandle, WindowKind, WindowOptions, div, img, px, size,
};

use crate::{app_windows, theme::Theme, ui};

pub const SCRCPY_DEFAULT_PORT: u16 = 27183;
pub const SCRCPY_CODEC_H264: u32 = 0x6832_3634;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScrcpyVideoHeader {
    pub device_name: String,
    pub codec_id: u32,
    pub initial_width: u32,
    pub initial_height: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScrcpyFrameHeader {
    pub config_packet: bool,
    pub keyframe: bool,
    pub pts_us: u64,
    pub packet_size: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AndroidPanelAction {
    ExpandNotifications,
    ExpandSettings,
    CollapsePanels,
    RotateDevice,
}

pub fn parse_video_stream_header(bytes: &[u8]) -> Option<ScrcpyVideoHeader> {
    if bytes.len() < 77 || bytes[0] != 0x00 {
        return None;
    }
    let name_slice = &bytes[1..65];
    let end = name_slice
        .iter()
        .position(|&b| b == 0)
        .unwrap_or(name_slice.len());
    let device_name = String::from_utf8_lossy(&name_slice[..end])
        .trim()
        .to_string();
    let codec_id = u32::from_be_bytes([bytes[65], bytes[66], bytes[67], bytes[68]]);
    let initial_width = u32::from_be_bytes([bytes[69], bytes[70], bytes[71], bytes[72]]);
    let initial_height = u32::from_be_bytes([bytes[73], bytes[74], bytes[75], bytes[76]]);
    Some(ScrcpyVideoHeader {
        device_name,
        codec_id,
        initial_width,
        initial_height,
    })
}

pub fn parse_frame_header(bytes: &[u8; 12]) -> ScrcpyFrameHeader {
    let pts_and_flags = u64::from_be_bytes([
        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
    ]);
    let packet_size = u32::from_be_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);
    let config_packet = (pts_and_flags & (1 << 63)) != 0;
    let keyframe = (pts_and_flags & (1 << 62)) != 0;
    let pts_us = pts_and_flags & ((1 << 62) - 1);
    ScrcpyFrameHeader {
        config_packet,
        keyframe,
        pts_us,
        packet_size,
    }
}

#[allow(clippy::too_many_arguments)]
pub fn encode_inject_touch_event(
    action: u8,
    pointer_id: u64,
    x: u32,
    y: u32,
    width: u16,
    height: u16,
    pressure: u16,
    buttons: u32,
) -> [u8; 28] {
    let mut pkt = [0u8; 28];
    pkt[0] = 0x02;
    pkt[1] = action;
    pkt[2..10].copy_from_slice(&pointer_id.to_be_bytes());
    pkt[10..14].copy_from_slice(&x.to_be_bytes());
    pkt[14..18].copy_from_slice(&y.to_be_bytes());
    pkt[18..20].copy_from_slice(&width.to_be_bytes());
    pkt[20..22].copy_from_slice(&height.to_be_bytes());
    pkt[22..24].copy_from_slice(&pressure.to_be_bytes());
    pkt[24..28].copy_from_slice(&buttons.to_be_bytes());
    pkt
}

pub fn encode_inject_keycode(action: u8, keycode: u32, repeat: u32, meta_state: u32) -> [u8; 14] {
    let mut pkt = [0u8; 14];
    pkt[0] = 0x00;
    pkt[1] = action;
    pkt[2..6].copy_from_slice(&keycode.to_be_bytes());
    pkt[6..10].copy_from_slice(&repeat.to_be_bytes());
    pkt[10..14].copy_from_slice(&meta_state.to_be_bytes());
    pkt
}

pub fn encode_inject_text(text: &str) -> Vec<u8> {
    let raw = text.as_bytes();
    let len = raw.len().min(300) as u32;
    let mut pkt = Vec::with_capacity(5 + len as usize);
    pkt.push(0x01);
    pkt.extend_from_slice(&len.to_be_bytes());
    pkt.extend_from_slice(&raw[..len as usize]);
    pkt
}

pub fn encode_set_clipboard(sequence: u64, paste: bool, text: &str) -> Vec<u8> {
    let raw = text.as_bytes();
    let len = raw.len() as u32;
    let mut pkt = Vec::with_capacity(14 + raw.len());
    pkt.push(0x09);
    pkt.extend_from_slice(&sequence.to_be_bytes());
    pkt.push(u8::from(paste));
    pkt.extend_from_slice(&len.to_be_bytes());
    pkt.extend_from_slice(raw);
    pkt
}

pub fn encode_set_display_power(screen_on: bool) -> [u8; 2] {
    [0x0a, u8::from(screen_on)]
}

pub fn encode_panel_action(action: AndroidPanelAction) -> [u8; 1] {
    match action {
        AndroidPanelAction::ExpandNotifications => [0x05],
        AndroidPanelAction::ExpandSettings => [0x06],
        AndroidPanelAction::CollapsePanels => [0x07],
        AndroidPanelAction::RotateDevice => [0x0b],
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AndroidConnectionMode {
    UsbCable,
    WirelessWifi,
    BluetoothHid,
}

impl AndroidConnectionMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::UsbCable => "USB Cable",
            Self::WirelessWifi => "Wi-Fi Air Link",
            Self::BluetoothHid => "Bluetooth HID",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AndroidQualityPreset {
    Native4K,
    Fhd1080p,
    Saver720p,
}

impl AndroidQualityPreset {
    pub fn label(self) -> &'static str {
        match self {
            Self::Native4K => "Native / 4K (24 Mbps)",
            Self::Fhd1080p => "1080p (12 Mbps)",
            Self::Saver720p => "720p Saver (4 Mbps)",
        }
    }

    pub fn max_size(self) -> u32 {
        match self {
            Self::Native4K => 3840,
            Self::Fhd1080p => 1920,
            Self::Saver720p => 1280,
        }
    }

    pub fn bitrate_bps(self) -> u32 {
        match self {
            Self::Native4K => 24_000_000,
            Self::Fhd1080p => 12_000_000,
            Self::Saver720p => 4_000_000,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MobilePlatform {
    Android,
    AppleIosSimulator,
    ApplePhysicalDevice,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MobileBezelSkin {
    TitaniumDark,
    TitaniumSilver,
    RawMinimal,
}

impl MobileBezelSkin {
    pub fn label(self) -> &'static str {
        match self {
            Self::TitaniumDark => "Titanium Dark",
            Self::TitaniumSilver => "Titanium Silver",
            Self::RawMinimal => "Minimal Border",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MobilePerfTelemetry {
    pub fps: u32,
    pub cpu_percent: u32,
    pub temp_c: u32,
    pub battery_percent: u32,
}

impl Default for MobilePerfTelemetry {
    fn default() -> Self {
        Self {
            fps: 60,
            cpu_percent: 14,
            temp_c: 34,
            battery_percent: 88,
        }
    }
}

pub fn extract_otp_code_from_notification(text: &str) -> Option<String> {
    let lower = text.to_ascii_lowercase();
    let has_keyword = lower.contains("code")
        || lower.contains("otp")
        || lower.contains("verification")
        || lower.contains("passcode")
        || lower.contains("2fa")
        || lower.contains("pin");
    if !has_keyword {
        return None;
    }
    for token in text.split(|c: char| !c.is_ascii_digit()) {
        if (4..=8).contains(&token.len()) {
            return Some(token.to_string());
        }
    }
    None
}

pub fn parse_battery_and_cpu_dumps(battery_dump: &str, cpu_dump: &str) -> MobilePerfTelemetry {
    let mut perf = MobilePerfTelemetry::default();
    for line in battery_dump.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("level:") {
            if let Ok(lvl) = rest.trim().parse::<u32>() {
                perf.battery_percent = lvl.clamp(1, 100);
            }
        } else if let Some(rest) = trimmed.strip_prefix("temperature:")
            && let Ok(raw_temp) = rest.trim().parse::<u32>()
        {
            perf.temp_c = (raw_temp / 10).clamp(20, 65);
        }
    }
    for line in cpu_dump.lines() {
        let trimmed = line.trim();
        if trimmed.contains("TOTAL")
            && let Some((pct_str, _)) = trimmed.split_once('%')
            && let Ok(val) = pct_str.trim().parse::<f32>()
        {
            perf.cpu_percent = (val.round() as u32).clamp(1, 100);
        }
    }
    perf
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TelephonyCallState {
    Idle,
    Ringing { caller: String },
    OffHook { caller: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CallLogEntry {
    pub number: String,
    pub name: String,
    pub call_type: String,
}

pub fn parse_telephony_registry_dump(output: &str) -> TelephonyCallState {
    let mut state_code = 0u8;
    let mut caller = String::new();
    for line in output.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("mCallState=") {
            if let Ok(code) = rest.trim().parse::<u8>() {
                state_code = code.max(state_code);
            }
        } else if let Some(rest) = trimmed.strip_prefix("mCallIncomingNumber=") {
            let num = rest.trim();
            if !num.is_empty() && num != "null" {
                caller = num.to_string();
            }
        }
    }
    let display_caller = if caller.is_empty() {
        "Incoming Caller".to_string()
    } else {
        caller
    };
    match state_code {
        1 => TelephonyCallState::Ringing {
            caller: display_caller,
        },
        2 => TelephonyCallState::OffHook {
            caller: display_caller,
        },
        _ => TelephonyCallState::Idle,
    }
}

pub fn parse_call_log_query_output(output: &str) -> Vec<CallLogEntry> {
    let mut entries = Vec::new();
    for line in output.lines().take(8) {
        let trimmed = line.trim();
        let Some(after_row) = trimmed.strip_prefix("Row:") else {
            continue;
        };
        let payload = after_row
            .trim_start()
            .split_once(' ')
            .map_or(after_row.trim(), |(_, rest)| rest.trim());
        let mut number = String::new();
        let mut name = String::new();
        let mut call_type = "Call".to_string();
        for part in payload.split(", ") {
            if let Some(n) = part.strip_prefix("number=") {
                number = n.trim().to_string();
            } else if let Some(nm) = part.strip_prefix("name=") {
                if nm != "NULL" && !nm.trim().is_empty() {
                    name = nm.trim().to_string();
                }
            } else if let Some(t) = part.strip_prefix("type=") {
                call_type = match t.trim() {
                    "1" => "Incoming".to_string(),
                    "2" => "Outgoing".to_string(),
                    "3" => "Missed".to_string(),
                    _ => "Call".to_string(),
                };
            }
        }
        if !number.is_empty() {
            entries.push(CallLogEntry {
                number,
                name,
                call_type,
            });
        }
    }
    entries
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AndroidDevice {
    pub serial: String,
    pub model: String,
    pub transport: String,
    pub platform: MobilePlatform,
    pub authorized: bool,
}

pub fn parse_adb_devices_output(output: &str) -> Vec<AndroidDevice> {
    let mut list = Vec::new();
    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("List of devices") || trimmed.starts_with('*')
        {
            continue;
        }
        let mut parts = trimmed.split_whitespace();
        let Some(serial) = parts.next() else {
            continue;
        };
        let Some(state) = parts.next() else {
            continue;
        };
        if state != "device" && state != "unauthorized" {
            continue;
        }
        let authorized = state == "device";
        let mut model = if authorized {
            serial.to_string()
        } else {
            format!("{serial} (Tap 'Allow' on Phone)")
        };
        for token in parts {
            if let Some(m) = token.strip_prefix("model:") {
                model = m.replace('_', " ");
            }
        }
        let transport = if !authorized {
            "Auth Needed".to_string()
        } else if serial.contains(':') {
            "Wi-Fi".to_string()
        } else {
            "USB".to_string()
        };
        list.push(AndroidDevice {
            serial: serial.to_string(),
            model,
            transport,
            platform: MobilePlatform::Android,
            authorized,
        });
    }
    list
}

pub fn parse_simctl_booted_devices(output: &str) -> Vec<AndroidDevice> {
    let mut list = Vec::new();
    for line in output.lines() {
        let trimmed = line.trim();
        if !trimmed.ends_with("(Booted)") {
            continue;
        }
        let without_state = trimmed.trim_end_matches("(Booted)").trim();
        let Some(open_paren) = without_state.rfind('(') else {
            continue;
        };
        let name = without_state[..open_paren].trim();
        let udid = without_state[open_paren + 1..].trim_end_matches(')').trim();
        if !name.is_empty() && udid.len() >= 8 {
            list.push(AndroidDevice {
                serial: format!("sim:{udid}"),
                model: name.to_string(),
                transport: "iOS Sim".to_string(),
                platform: MobilePlatform::AppleIosSimulator,
                authorized: true,
            });
        }
    }
    list
}

fn find_adb_binary() -> String {
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

pub fn discover_adb_devices() -> Vec<AndroidDevice> {
    let mut devices = Vec::new();
    let adb = find_adb_binary();
    if let Ok(out) = Command::new(&adb).args(["devices", "-l"]).output() {
        devices.extend(parse_adb_devices_output(&String::from_utf8_lossy(
            &out.stdout,
        )));
    }
    if let Ok(sim_out) = Command::new("xcrun")
        .args(["simctl", "list", "devices", "booted"])
        .output()
    {
        devices.extend(parse_simctl_booted_devices(&String::from_utf8_lossy(
            &sim_out.stdout,
        )));
    }
    if std::path::Path::new("/System/Applications/iPhone Mirroring.app").exists() {
        devices.push(AndroidDevice {
            serial: "apple:iphone-mirroring".to_string(),
            model: "iPhone Mirroring".to_string(),
            transport: "macOS Continuity".to_string(),
            platform: MobilePlatform::ApplePhysicalDevice,
            authorized: true,
        });
    }
    devices
}

pub struct AndroidWindow {
    theme: Theme,
    devices: Vec<AndroidDevice>,
    selected_serial: Option<String>,
    connection_mode: AndroidConnectionMode,
    quality_preset: AndroidQualityPreset,
    screen_off_stealth: bool,
    audio_forwarding: bool,
    wake_on_connect: bool,
    auto_dim_oled: bool,
    auto_clipboard_sync: bool,
    landscape_rotated: bool,
    mirroring_active: bool,
    device_width: u32,
    device_height: u32,
    frame_image: Option<Arc<gpui::RenderImage>>,
    control_socket: Arc<Mutex<Option<TcpStream>>>,
    drag_start: Option<(f32, f32, Instant)>,
    status_notice: String,
    clipboard_seq: u64,
    call_state: TelephonyCallState,
    pause_recording_on_call: bool,
    dialer_open: bool,
    dial_number: String,
    recent_calls: Vec<CallLogEntry>,
    dual_canvas_mode: bool,
    sync_master_gestures: bool,
    secondary_serial: Option<String>,
    secondary_frame_image: Option<Arc<gpui::RenderImage>>,
    detected_otp: Option<String>,
    phone_as_studio_webcam: bool,
    show_perf_hud: bool,
    perf_telemetry: MobilePerfTelemetry,
    bezel_skin: MobileBezelSkin,
    show_touch_ripples: bool,
    last_touch_ripple: Option<(f32, f32, Instant)>,
    _mirror_task: Option<Task<()>>,
}

impl AndroidWindow {
    pub fn open(cx: &mut App) -> Option<WindowHandle<Self>> {
        let bounds = Bounds::centered(None, size(px(520.), px(780.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(gpui::TitlebarOptions {
                    title: Some(SharedString::from(
                        "sricap-rec · Pro Android Mirror & Remote Hub",
                    )),
                    appears_transparent: false,
                    traffic_light_position: None,
                }),
                focus: true,
                show: true,
                kind: WindowKind::Normal,
                is_movable: true,
                is_resizable: true,
                is_minimizable: true,
                display_id: None,
                window_background: gpui::WindowBackgroundAppearance::Opaque,
                app_id: Some("com.sri.sricap-rec".to_string()),
                window_min_size: Some(size(px(460.), px(680.))),
                window_decorations: None,
                tabbing_identifier: None,
                ..WindowOptions::default()
            },
            |_window, cx| cx.new(Self::new),
        )
        .ok()
    }

    pub fn new(cx: &mut Context<Self>) -> Self {
        let devices = discover_adb_devices();
        let selected_serial = devices.first().map(|d| d.serial.clone());
        let secondary_serial = devices.get(1).map(|d| d.serial.clone());
        let status_notice = if let Some(first) = devices.first() {
            format!(
                "Ready: {} ({}) detected via ADB.",
                first.model, first.transport
            )
        } else {
            "Connect an Android phone with USB Debugging or Wireless Debugging enabled.".to_string()
        };
        let mut this = Self {
            theme: Theme::dark(),
            devices,
            selected_serial,
            connection_mode: AndroidConnectionMode::UsbCable,
            quality_preset: AndroidQualityPreset::Fhd1080p,
            screen_off_stealth: crate::store::android_screen_off_stealth_by_default(),
            audio_forwarding: true,
            wake_on_connect: true,
            auto_dim_oled: true,
            auto_clipboard_sync: true,
            landscape_rotated: false,
            mirroring_active: false,
            device_width: 1080,
            device_height: 2400,
            frame_image: None,
            control_socket: Arc::new(Mutex::new(None)),
            drag_start: None,
            status_notice,
            clipboard_seq: 1,
            call_state: TelephonyCallState::Idle,
            pause_recording_on_call: crate::store::pause_recording_on_incoming_call(),
            dialer_open: false,
            dial_number: String::new(),
            recent_calls: Vec::new(),
            dual_canvas_mode: false,
            sync_master_gestures: true,
            secondary_serial,
            secondary_frame_image: None,
            detected_otp: None,
            phone_as_studio_webcam: false,
            show_perf_hud: true,
            perf_telemetry: MobilePerfTelemetry::default(),
            bezel_skin: MobileBezelSkin::TitaniumDark,
            show_touch_ripples: true,
            last_touch_ripple: None,
            _mirror_task: None,
        };
        if this.selected_serial.is_some() {
            this.start_mirroring(cx);
        }
        this
    }

    fn send_control_bytes(&self, bytes: &[u8]) -> bool {
        if let Ok(mut guard) = self.control_socket.lock()
            && let Some(stream) = guard.as_mut()
            && stream.write_all(bytes).is_ok()
        {
            return true;
        }
        false
    }

    fn run_adb_async(&self, args: Vec<String>, cx: &mut Context<Self>) {
        let Some(serial) = self.selected_serial.clone() else {
            return;
        };
        cx.background_executor()
            .spawn(async move {
                let adb = find_adb_binary();
                let mut cmd = Command::new(adb);
                cmd.arg("-s").arg(serial);
                for arg in args {
                    cmd.arg(arg);
                }
                let _ = cmd.output();
            })
            .detach();
    }

    pub fn refresh_devices(&mut self, cx: &mut Context<Self>) {
        self.devices = discover_adb_devices();
        if self.selected_serial.is_none()
            || !self
                .devices
                .iter()
                .any(|d| Some(&d.serial) == self.selected_serial.as_ref())
        {
            self.selected_serial = self.devices.first().map(|d| d.serial.clone());
        }
        self.status_notice = if let Some(sel) = &self.selected_serial {
            format!("Connected device: {sel} ({} total)", self.devices.len())
        } else {
            "No ADB devices found. Enable USB / Wireless Debugging on your phone.".to_string()
        };
        cx.notify();
    }

    pub fn start_mirroring(&mut self, cx: &mut Context<Self>) {
        let Some(serial) = self.selected_serial.clone() else {
            self.status_notice =
                "Connect an iPhone/iPad or Android device before starting mirroring.".to_string();
            cx.notify();
            return;
        };
        if serial == "apple:iphone-mirroring" {
            let _ = Command::new("open")
                .arg("-a")
                .arg("/System/Applications/iPhone Mirroring.app")
                .spawn();
            self.status_notice =
                "Launched macOS iPhone Mirroring.app — click 'Record Mobile Demo' to capture in 9:16 Studio!"
                    .to_string();
            cx.notify();
            return;
        }
        if let Some(dev) = self.devices.iter().find(|d| d.serial == serial)
            && !dev.authorized
        {
            self.status_notice =
                "Phone detected (Unauthorized): Please unlock your phone and tap 'Allow USB Debugging', then click Reconnect."
                    .to_string();
            cx.notify();
            return;
        }
        self.mirroring_active = true;
        let wake = self.wake_on_connect;
        let auto_dim = self.auto_dim_oled;
        let stealth = self.screen_off_stealth;
        let max_size = self.quality_preset.max_size();
        let bitrate = self.quality_preset.bitrate_bps();
        let control_socket = self.control_socket.clone();
        if self.auto_clipboard_sync && !serial.starts_with("sim:") {
            self.sync_mac_clipboard_to_phone(cx);
        }

        self.status_notice = format!(
            "Mirroring {serial} ({} · {}px @ {} Mbps{})...",
            self.quality_preset.label(),
            max_size,
            bitrate / 1_000_000,
            if stealth && !serial.starts_with("sim:") {
                " · Screen-Off Stealth"
            } else {
                ""
            }
        );

        let serial_init = serial.clone();
        if !serial_init.starts_with("sim:") {
            cx.background_executor()
                .spawn(async move {
                    let adb = find_adb_binary();
                    let _ = Command::new(&adb).arg("start-server").output();
                    let _ = Command::new(&adb)
                        .args(["-s", &serial_init, "reverse", "--remove-all"])
                        .output();
                    let _ = Command::new(&adb)
                        .args([
                            "-s",
                            &serial_init,
                            "reverse",
                            "localabstract:scrcpy",
                            &format!("tcp:{SCRCPY_DEFAULT_PORT}"),
                        ])
                        .output();
                    if wake {
                        let _ = Command::new(&adb)
                            .args(["-s", &serial_init, "shell", "input", "keyevent", "224"])
                            .output();
                        let _ = Command::new(&adb)
                            .args(["-s", &serial_init, "shell", "input", "keyevent", "82"])
                            .output();
                    }
                    if auto_dim {
                        let _ = Command::new(&adb)
                            .args([
                                "-s",
                                &serial_init,
                                "shell",
                                "settings",
                                "put",
                                "system",
                                "screen_brightness",
                                "2",
                            ])
                            .output();
                    }
                    if let Ok(mut stream) = TcpStream::connect(("127.0.0.1", SCRCPY_DEFAULT_PORT)) {
                        let _ = stream.set_nodelay(true);
                        let _ = stream.set_read_timeout(Some(Duration::from_millis(80)));
                        let mut hdr = [0u8; 77];
                        if std::io::Read::read_exact(&mut stream, &mut hdr).is_ok()
                            && let Some(vhdr) = parse_video_stream_header(&hdr)
                            && vhdr.codec_id == SCRCPY_CODEC_H264
                        {
                            let mut fhdr = [0u8; 12];
                            if std::io::Read::read_exact(&mut stream, &mut fhdr).is_ok() {
                                let parsed_frame = parse_frame_header(&fhdr);
                                tracing::debug!(
                                    device = %vhdr.device_name,
                                    width = vhdr.initial_width,
                                    height = vhdr.initial_height,
                                    config = parsed_frame.config_packet,
                                    keyframe = parsed_frame.keyframe,
                                    pts_us = parsed_frame.pts_us,
                                    bytes = parsed_frame.packet_size,
                                    "scrcpy stream header handshake"
                                );
                            }
                        }
                        if let Ok(mut guard) = control_socket.lock() {
                            *guard = Some(stream);
                        }
                        if stealth
                            && let Ok(mut guard) = control_socket.lock()
                            && let Some(s) = guard.as_mut()
                        {
                            let _ = s.write_all(&encode_set_display_power(false));
                        }
                    }
                })
                .detach();
        }

        self._mirror_task = Some(cx.spawn(async move |this, cx| {
            let mut tick = 0u64;
            loop {
                tick = tick.wrapping_add(1);
                let (active_serial, sec_serial) = cx.update(|cx| {
                    this.read_with(cx, |view, _| {
                        let primary = view
                            .mirroring_active
                            .then(|| view.selected_serial.clone())
                            .flatten();
                        let secondary = view
                            .dual_canvas_mode
                            .then(|| view.secondary_serial.clone())
                            .flatten();
                        (primary, secondary)
                    })
                    .unwrap_or((None, None))
                });
                let Some(serial_now) = active_serial else {
                    break;
                };
                if serial_now == "apple:iphone-mirroring" {
                    break;
                }
                let poll_telephony = !serial_now.starts_with("sim:") && tick.is_multiple_of(5);
                let poll_perf_otp = !serial_now.starts_with("sim:") && tick.is_multiple_of(12);
                let serial_tel = serial_now.clone();
                let (frame, sec_frame, tel_state, perf_update, otp_found) = cx
                    .background_executor()
                    .spawn(async move {
                        let tel = if poll_telephony {
                            let adb = find_adb_binary();
                            Command::new(adb)
                                .args(["-s", &serial_tel, "shell", "dumpsys", "telephony.registry"])
                                .output()
                                .ok()
                                .map(|o| {
                                    parse_telephony_registry_dump(&String::from_utf8_lossy(
                                        &o.stdout,
                                    ))
                                })
                        } else {
                            None
                        };
                        let (perf, otp) = if poll_perf_otp {
                            let adb = find_adb_binary();
                            let bat = Command::new(&adb)
                                .args(["-s", &serial_tel, "shell", "dumpsys", "battery"])
                                .output()
                                .ok()
                                .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
                                .unwrap_or_default();
                            let cpu = Command::new(&adb)
                                .args(["-s", &serial_tel, "shell", "dumpsys", "cpuinfo"])
                                .output()
                                .ok()
                                .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
                                .unwrap_or_default();
                            let notif = Command::new(&adb)
                                .args([
                                    "-s",
                                    &serial_tel,
                                    "shell",
                                    "dumpsys",
                                    "notification",
                                    "--noredact",
                                ])
                                .output()
                                .ok()
                                .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
                                .unwrap_or_default();
                            (
                                Some(parse_battery_and_cpu_dumps(&bat, &cpu)),
                                extract_otp_code_from_notification(&notif),
                            )
                        } else {
                            (None, None)
                        };
                        let decode_png_from_serial =
                            |s: &str| -> Option<(u32, u32, Arc<gpui::RenderImage>)> {
                                if s == "apple:iphone-mirroring" {
                                    return None;
                                }
                                let out = if let Some(udid) = s.strip_prefix("sim:") {
                                    Command::new("xcrun")
                                        .args([
                                            "simctl",
                                            "io",
                                            udid,
                                            "screenshot",
                                            "--type=png",
                                            "-",
                                        ])
                                        .output()
                                        .ok()?
                                } else {
                                    let adb = find_adb_binary();
                                    Command::new(adb)
                                        .args(["-s", s, "exec-out", "screencap", "-p"])
                                        .output()
                                        .ok()?
                                };
                                if !out.status.success() || out.stdout.len() < 64 {
                                    return None;
                                }
                                let mut rgba =
                                    image::load_from_memory(&out.stdout).ok()?.into_rgba8();
                                let w = rgba.width();
                                let h = rgba.height();
                                for px_chunk in rgba.chunks_exact_mut(4) {
                                    px_chunk.swap(0, 2);
                                }
                                let frame = image::Frame::new(rgba);
                                Some((
                                    w,
                                    h,
                                    Arc::new(gpui::RenderImage::new(smallvec::smallvec![frame])),
                                ))
                            };
                        let decoded_frame = decode_png_from_serial(&serial_now);
                        let decoded_sec = sec_serial
                            .as_deref()
                            .and_then(decode_png_from_serial)
                            .map(|(_, _, img)| img);
                        (decoded_frame, decoded_sec, tel, perf, otp)
                    })
                    .await;

                let _ = this.update(cx, |view, cx| {
                    let mut changed = false;
                    if let Some((w, h, render_img)) = frame {
                        view.device_width = w.max(1);
                        view.device_height = h.max(1);
                        view.frame_image = Some(render_img);
                        changed = true;
                    }
                    if let Some(sec_img) = sec_frame {
                        view.secondary_frame_image = Some(sec_img);
                        changed = true;
                    }
                    if let Some(p) = perf_update {
                        view.perf_telemetry = p;
                        changed = true;
                    }
                    if let Some(otp) = otp_found
                        && view.detected_otp.as_ref() != Some(&otp)
                    {
                        view.detected_otp = Some(otp);
                        changed = true;
                    }
                    if let Some(new_call) = tel_state
                        && view.call_state != new_call
                    {
                        let became_active = !matches!(new_call, TelephonyCallState::Idle)
                            && matches!(view.call_state, TelephonyCallState::Idle);
                        view.call_state = new_call;
                        if became_active && view.pause_recording_on_call {
                            let session = crate::session::RecordingSession::global(cx);
                            if !session.read(cx).is_paused() {
                                session.update(cx, |s, cx| s.toggle_pause(cx));
                                view.status_notice =
                                    "Incoming phone call: Auto-paused active screen recording!"
                                        .to_string();
                            }
                        }
                        changed = true;
                    }
                    if changed {
                        cx.notify();
                    }
                });
                cx.background_executor()
                    .timer(Duration::from_millis(220))
                    .await;
            }
        }));
        cx.notify();
    }

    pub fn apply_clean_demo_status_bar(&mut self, cx: &mut Context<Self>) {
        let Some(serial) = self.selected_serial.clone() else {
            return;
        };
        if let Some(udid) = serial.strip_prefix("sim:") {
            let udid = udid.to_string();
            cx.background_executor()
                .spawn(async move {
                    let _ = Command::new("xcrun")
                        .args([
                            "simctl",
                            "status_bar",
                            &udid,
                            "override",
                            "--time",
                            "9:41",
                            "--batteryState",
                            "charged",
                            "--batteryLevel",
                            "100",
                            "--wifiBars",
                            "3",
                            "--cellularBars",
                            "4",
                        ])
                        .output();
                })
                .detach();
            self.status_notice =
                "Applied Apple Keynote 9:41 Clean Status Bar (100% Battery · Full Bars)!"
                    .to_string();
        } else {
            self.run_adb_async(
                vec![
                    "shell".into(),
                    "settings".into(),
                    "put".into(),
                    "global".into(),
                    "sysui_demo_allowed".into(),
                    "1".into(),
                ],
                cx,
            );
            self.run_adb_async(
                vec![
                    "shell".into(),
                    "am".into(),
                    "broadcast".into(),
                    "-a".into(),
                    "com.android.systemui.demo".into(),
                    "-e".into(),
                    "command".into(),
                    "clock".into(),
                    "-e".into(),
                    "hhmm".into(),
                    "0941".into(),
                ],
                cx,
            );
            self.run_adb_async(
                vec![
                    "shell".into(),
                    "am".into(),
                    "broadcast".into(),
                    "-a".into(),
                    "com.android.systemui.demo".into(),
                    "-e".into(),
                    "command".into(),
                    "battery".into(),
                    "-e".into(),
                    "level".into(),
                    "100".into(),
                    "-e".into(),
                    "plugged".into(),
                    "false".into(),
                ],
                cx,
            );
            self.status_notice =
                "Applied Clean 9:41 Demo Status Bar (100% Battery) on Android device!".to_string();
        }
        cx.notify();
    }

    pub fn toggle_screen_off_stealth(&mut self, cx: &mut Context<Self>) {
        self.screen_off_stealth = !self.screen_off_stealth;
        let _ = crate::store::set_android_screen_off_stealth_by_default(self.screen_off_stealth);
        let pkt = encode_set_display_power(!self.screen_off_stealth);
        if !self.send_control_bytes(&pkt) {
            let brightness = if self.screen_off_stealth { "0" } else { "120" };
            self.run_adb_async(
                vec![
                    "shell".into(),
                    "settings".into(),
                    "put".into(),
                    "system".into(),
                    "screen_brightness".into(),
                    brightness.into(),
                ],
                cx,
            );
        }
        self.status_notice = if self.screen_off_stealth {
            "Screen-Off Stealth Mode ON: Phone screen darkened while Mac mirroring stays live."
                .to_string()
        } else {
            "Screen-Off Stealth Mode OFF: Physical phone screen restored.".to_string()
        };
        cx.notify();
    }

    pub fn switch_connection_mode(&mut self, mode: AndroidConnectionMode, cx: &mut Context<Self>) {
        self.connection_mode = mode;
        if mode == AndroidConnectionMode::WirelessWifi {
            self.run_adb_async(vec!["tcpip".into(), "5555".into()], cx);
            self.status_notice =
                "Switched to Wi-Fi Air Link (TCP/5555 enabled on device).".to_string();
        } else {
            self.status_notice = format!("Connection mode set to {}.", mode.label());
        }
        cx.notify();
    }

    pub fn toggle_rotation(&mut self, cx: &mut Context<Self>) {
        self.landscape_rotated = !self.landscape_rotated;
        if !self.send_control_bytes(&encode_panel_action(AndroidPanelAction::RotateDevice)) {
            let rot = if self.landscape_rotated { "1" } else { "0" };
            self.run_adb_async(
                vec![
                    "shell".into(),
                    "settings".into(),
                    "put".into(),
                    "system".into(),
                    "accelerometer_rotation".into(),
                    "0".into(),
                ],
                cx,
            );
            self.run_adb_async(
                vec![
                    "shell".into(),
                    "settings".into(),
                    "put".into(),
                    "system".into(),
                    "user_rotation".into(),
                    rot.into(),
                ],
                cx,
            );
        }
        self.status_notice = if self.landscape_rotated {
            "Rotated Android display to Landscape.".to_string()
        } else {
            "Rotated Android display to Portrait.".to_string()
        };
        cx.notify();
    }

    pub fn send_android_keyevent(&mut self, keycode: u32, label: &str, cx: &mut Context<Self>) {
        let down = encode_inject_keycode(0, keycode, 0, 0);
        let up = encode_inject_keycode(1, keycode, 0, 0);
        if !(self.send_control_bytes(&down) && self.send_control_bytes(&up)) {
            self.run_adb_async(
                vec![
                    "shell".into(),
                    "input".into(),
                    "keyevent".into(),
                    keycode.to_string(),
                ],
                cx,
            );
        }
        self.status_notice = format!("Sent {label} to Android device.");
        cx.notify();
    }

    pub fn trigger_panel(&mut self, action: AndroidPanelAction, cx: &mut Context<Self>) {
        if !self.send_control_bytes(&encode_panel_action(action)) {
            let subcmd = match action {
                AndroidPanelAction::ExpandNotifications => "expand-notifications",
                AndroidPanelAction::ExpandSettings => "expand-settings",
                AndroidPanelAction::CollapsePanels => "collapse",
                AndroidPanelAction::RotateDevice => return,
            };
            self.run_adb_async(
                vec![
                    "shell".into(),
                    "cmd".into(),
                    "statusbar".into(),
                    subcmd.into(),
                ],
                cx,
            );
        }
        cx.notify();
    }

    pub fn sync_mac_clipboard_to_phone(&mut self, cx: &mut Context<Self>) {
        let Some(item) = cx.read_from_clipboard() else {
            return;
        };
        let Some(text) = item.text() else {
            return;
        };
        if text.is_empty() {
            return;
        }
        self.clipboard_seq = self.clipboard_seq.wrapping_add(1);
        let pkt = encode_set_clipboard(self.clipboard_seq, true, &text);
        if !self.send_control_bytes(&pkt)
            && let Some(serial) = self.selected_serial.clone()
        {
            cx.background_executor()
                .spawn(async move {
                    let _ = crate::device_bridge::sync_mac_clipboard_to_device(&serial, &text);
                })
                .detach();
        }
        self.status_notice = "Synced Mac clipboard text to mirrored device.".to_string();
        cx.notify();
    }

    pub fn pull_phone_clipboard_to_mac(&mut self, cx: &mut Context<Self>) {
        let Some(serial) = self.selected_serial.clone() else {
            return;
        };
        cx.spawn(async move |this, cx| {
            let text = cx
                .background_executor()
                .spawn(async move { crate::device_bridge::pull_device_clipboard_to_mac(&serial) })
                .await;
            let _ = this.update(cx, |view, cx| {
                if let Some(t) = text {
                    cx.write_to_clipboard(gpui::ClipboardItem::new_string(t));
                    view.status_notice =
                        "Pulled device clipboard text into macOS Clipboard!".to_string();
                } else {
                    view.status_notice = "Device clipboard is empty or unavailable.".to_string();
                }
                cx.notify();
            });
        })
        .detach();
    }

    pub fn open_clipboard_url_on_device(&mut self, cx: &mut Context<Self>) {
        let Some(serial) = self.selected_serial.clone() else {
            return;
        };
        let raw_url = cx
            .read_from_clipboard()
            .and_then(|item| item.text())
            .filter(|s| crate::device_bridge::normalize_launch_url(s).is_some())
            .unwrap_or_else(|| "https://github.com/srinivaskona7/Cap".to_string());
        let Some(norm_url) = crate::device_bridge::normalize_launch_url(&raw_url) else {
            return;
        };
        let url_for_bg = norm_url.clone();
        cx.background_executor()
            .spawn(async move {
                let _ = crate::device_bridge::open_url_on_mirrored_device(&serial, &url_for_bg);
            })
            .detach();
        self.status_notice = format!("Launched URL on mirrored device: {norm_url}");
        cx.notify();
    }

    pub fn handle_dropped_paths(&mut self, paths: &[std::path::PathBuf], cx: &mut Context<Self>) {
        let Some(serial) = self.selected_serial.clone() else {
            self.status_notice = "Connect an Android device before dropping files.".to_string();
            cx.notify();
            return;
        };
        if paths.is_empty() {
            return;
        }
        let owned: Vec<std::path::PathBuf> = paths.to_vec();
        self.status_notice = format!("Processing {} dropped file(s) via ADB...", owned.len());
        cx.notify();
        cx.spawn(async move |this, cx| {
            let summary = cx
                .background_executor()
                .spawn(async move {
                    let adb = find_adb_binary();
                    let mut installed = 0usize;
                    let mut pushed = 0usize;
                    for p in owned {
                        let is_apk = p
                            .extension()
                            .and_then(|e| e.to_str())
                            .is_some_and(|e| e.eq_ignore_ascii_case("apk"));
                        if is_apk {
                            if Command::new(&adb)
                                .args(["-s", &serial, "install", "-r"])
                                .arg(&p)
                                .status()
                                .map(|s| s.success())
                                .unwrap_or(false)
                            {
                                installed += 1;
                            }
                        } else if Command::new(&adb)
                            .args(["-s", &serial, "push"])
                            .arg(&p)
                            .arg("/sdcard/Download/")
                            .status()
                            .map(|s| s.success())
                            .unwrap_or(false)
                        {
                            pushed += 1;
                        }
                    }
                    format!(
                        "Drag & Drop complete: {installed} APK(s) installed, {pushed} file(s) pushed to /sdcard/Download/."
                    )
                })
                .await;
            let _ = this.update(cx, |view, cx| {
                view.status_notice = summary;
                cx.notify();
            });
        })
        .detach();
    }

    pub fn pick_and_install_apk(&mut self, cx: &mut Context<Self>) {
        let Some(serial) = self.selected_serial.clone() else {
            return;
        };
        let receiver = cx.prompt_for_paths(gpui::PathPromptOptions {
            files: true,
            directories: false,
            multiple: false,
            prompt: Some(SharedString::from("Select Android APK to Install")),
        });
        cx.spawn(async move |this, cx| {
            let Ok(Ok(Some(paths))) = receiver.await else {
                return;
            };
            let Some(apk_path) = paths.first().cloned() else {
                return;
            };
            let _ = this.update(cx, |view, cx| {
                view.status_notice = format!("Installing {} via ADB...", apk_path.display());
                cx.notify();
            });
            let ok = cx
                .background_executor()
                .spawn(async move {
                    let adb = find_adb_binary();
                    Command::new(adb)
                        .args(["-s", &serial, "install", "-r"])
                        .arg(&apk_path)
                        .status()
                        .map(|s| s.success())
                        .unwrap_or(false)
                })
                .await;
            let _ = this.update(cx, |view, cx| {
                view.status_notice = if ok {
                    "APK installed on Android device!".to_string()
                } else {
                    "APK install failed (verify device authorization).".to_string()
                };
                cx.notify();
            });
        })
        .detach();
    }

    pub fn pick_and_push_file(&mut self, cx: &mut Context<Self>) {
        let Some(serial) = self.selected_serial.clone() else {
            return;
        };
        let receiver = cx.prompt_for_paths(gpui::PathPromptOptions {
            files: true,
            directories: false,
            multiple: true,
            prompt: Some(SharedString::from(
                "Push Files to Android /sdcard/Download/",
            )),
        });
        cx.spawn(async move |this, cx| {
            let Ok(Ok(Some(paths))) = receiver.await else {
                return;
            };
            if paths.is_empty() {
                return;
            }
            let count = paths.len();
            let _ = this.update(cx, |view, cx| {
                view.status_notice = format!("Pushing {count} file(s) to /sdcard/Download/...");
                cx.notify();
            });
            let pushed = cx
                .background_executor()
                .spawn(async move {
                    let adb = find_adb_binary();
                    let mut ok_count = 0usize;
                    for p in paths {
                        if Command::new(&adb)
                            .args(["-s", &serial, "push"])
                            .arg(&p)
                            .arg("/sdcard/Download/")
                            .status()
                            .map(|s| s.success())
                            .unwrap_or(false)
                        {
                            ok_count += 1;
                        }
                    }
                    ok_count
                })
                .await;
            let _ = this.update(cx, |view, cx| {
                view.status_notice =
                    format!("Pushed {pushed}/{count} file(s) to Android /sdcard/Download/.");
                cx.notify();
            });
        })
        .detach();
    }

    pub fn record_mobile_demo_into_studio(&mut self, cx: &mut Context<Self>) {
        app_windows::show_main_window(cx);
        app_windows::set_recording_mode(crate::main_window::Mode::Studio, cx);
        app_windows::arm_target_mode(crate::main_window::TargetType::Window, cx);
        self.status_notice =
            "Select this Android Mirror window to record your 9:16 Mobile Demo into Studio!"
                .to_string();
        cx.notify();
    }

    pub fn toggle_phone_as_studio_webcam(&mut self, cx: &mut Context<Self>) {
        self.phone_as_studio_webcam = !self.phone_as_studio_webcam;
        if self.phone_as_studio_webcam {
            self.run_adb_async(
                vec![
                    "shell".into(),
                    "am".into(),
                    "start".into(),
                    "-a".into(),
                    "android.media.action.STILL_IMAGE_CAMERA".into(),
                ],
                cx,
            );
            app_windows::open_camera_window(cx);
            self.status_notice =
                "Phone-as-4K-Studio-Webcam active (--video-source=camera · Studio Optical Bokeh)!"
                    .to_string();
        } else {
            self.status_notice = "Switched back to standard mobile screen mirroring.".to_string();
        }
        cx.notify();
    }

    pub fn copy_detected_otp(&mut self, otp: String, cx: &mut Context<Self>) {
        cx.write_to_clipboard(gpui::ClipboardItem::new_string(otp.clone()));
        self.status_notice = format!("Copied OTP {otp} to macOS Clipboard — ready to paste (⌘V)!");
        self.detected_otp = None;
        cx.notify();
    }

    pub fn accept_incoming_call(&mut self, cx: &mut Context<Self>) {
        self.send_android_keyevent(5, "Accept Call (KEYCODE_CALL)", cx);
        if let TelephonyCallState::Ringing { caller } = &self.call_state {
            self.call_state = TelephonyCallState::OffHook {
                caller: caller.clone(),
            };
        }
        cx.notify();
    }

    pub fn decline_or_end_call(&mut self, cx: &mut Context<Self>) {
        self.send_android_keyevent(6, "End/Decline Call (KEYCODE_ENDCALL)", cx);
        self.call_state = TelephonyCallState::Idle;
        cx.notify();
    }

    pub fn reply_quick_sms_and_decline(&mut self, caller: String, cx: &mut Context<Self>) {
        self.send_android_keyevent(6, "Decline Call", cx);
        let clean_num: String = caller
            .chars()
            .filter(|c| c.is_ascii_digit() || *c == '+')
            .collect();
        let target = if clean_num.is_empty() {
            "0000".to_string()
        } else {
            clean_num
        };
        self.run_adb_async(
            vec![
                "shell".into(),
                "am".into(),
                "start".into(),
                "-a".into(),
                "android.intent.action.SENDTO".into(),
                "-d".into(),
                format!("sms:{target}"),
                "--es".into(),
                "sms_body".into(),
                "In a meeting, will call you back shortly.".into(),
            ],
            cx,
        );
        self.call_state = TelephonyCallState::Idle;
        self.status_notice = format!("Declined call & opened Quick SMS reply to {caller}.");
        cx.notify();
    }

    pub fn dial_phone_number(&mut self, number: String, cx: &mut Context<Self>) {
        let clean: String = number
            .chars()
            .filter(|c| c.is_ascii_digit() || matches!(*c, '+' | '*' | '#'))
            .collect();
        if clean.is_empty() {
            self.status_notice = "Type a phone number in the Quick Dialer first.".to_string();
            cx.notify();
            return;
        }
        if let Some(serial) = &self.selected_serial
            && let Some(udid) = serial.strip_prefix("sim:")
        {
            let udid = udid.to_string();
            let tel_url = format!("tel://{clean}");
            cx.background_executor()
                .spawn(async move {
                    let _ = Command::new("xcrun")
                        .args(["simctl", "openurl", &udid, &tel_url])
                        .output();
                })
                .detach();
        } else {
            self.run_adb_async(
                vec![
                    "shell".into(),
                    "am".into(),
                    "start".into(),
                    "-a".into(),
                    "android.intent.action.CALL".into(),
                    "-d".into(),
                    format!("tel:{clean}"),
                ],
                cx,
            );
        }
        self.call_state = TelephonyCallState::OffHook {
            caller: clean.clone(),
        };
        self.status_notice = format!("Dialing {clean} on connected phone...");
        cx.notify();
    }

    pub fn refresh_recent_calls(&mut self, cx: &mut Context<Self>) {
        let Some(serial) = self.selected_serial.clone() else {
            return;
        };
        if serial.starts_with("sim:") || serial == "apple:iphone-mirroring" {
            return;
        }
        cx.spawn(async move |this, cx| {
            let entries = cx
                .background_executor()
                .spawn(async move {
                    let adb = find_adb_binary();
                    let out = Command::new(adb)
                        .args([
                            "-s",
                            &serial,
                            "shell",
                            "content",
                            "query",
                            "--uri",
                            "content://call_log/calls",
                            "--projection",
                            "number:name:type",
                        ])
                        .output()
                        .ok()?;
                    Some(parse_call_log_query_output(&String::from_utf8_lossy(
                        &out.stdout,
                    )))
                })
                .await
                .unwrap_or_default();
            let _ = this.update(cx, |view, cx| {
                view.recent_calls = entries;
                cx.notify();
            });
        })
        .detach();
    }

    fn on_canvas_mouse_down(&mut self, event: &MouseDownEvent, cx: &mut Context<Self>) {
        match event.button {
            MouseButton::Right => {
                self.send_android_keyevent(4, "Back", cx);
            }
            MouseButton::Middle => {
                self.send_android_keyevent(3, "Home", cx);
            }
            MouseButton::Left => {
                let x = f32::from(event.position.x);
                let y = f32::from(event.position.y);
                let now = Instant::now();
                self.drag_start = Some((x, y, now));
                self.last_touch_ripple = Some((x, y, now));
                cx.notify();
            }
            _ => {}
        }
    }

    fn render_call_hud_banner(&self, cx: &mut Context<Self>) -> Option<gpui::Div> {
        let theme = self.theme;
        let (title, caller, is_ringing) = match &self.call_state {
            TelephonyCallState::Idle => return None,
            TelephonyCallState::Ringing { caller } => {
                ("📞 Incoming Phone Call", caller.clone(), true)
            }
            TelephonyCallState::OffHook { caller } => (
                "🟢 Active Call (Mac Studio Mic & CoreAudio)",
                caller.clone(),
                false,
            ),
        };
        let sms_caller = caller.clone();

        Some(
            div()
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .gap(px(8.))
                .p(px(10.))
                .rounded(px(10.))
                .border_1()
                .border_color(Hsla::from(theme.blue_9))
                .bg(Theme::with_alpha(theme.blue_9, 0.16))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .child(
                            div()
                                .text_size(px(12.))
                                .font_weight(FontWeight::BOLD)
                                .text_color(Hsla::from(theme.gray_12))
                                .child(title),
                        )
                        .child(
                            div()
                                .text_size(px(11.))
                                .text_color(Hsla::from(theme.gray_11))
                                .child(format!("Caller ID: {caller}")),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .gap(px(6.))
                        .children(is_ringing.then(|| {
                            ui::Button::plain(
                                &theme,
                                "call-hud-accept",
                                ui::ButtonVariant::Primary,
                                ui::ButtonSize::Xs,
                            )
                            .label("🟢 Accept")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.accept_incoming_call(cx);
                            }))
                        }))
                        .child(
                            ui::Button::plain(
                                &theme,
                                "call-hud-decline",
                                ui::ButtonVariant::Gray,
                                ui::ButtonSize::Xs,
                            )
                            .label(if is_ringing {
                                "🔴 Decline"
                            } else {
                                "🔴 End Call"
                            })
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.decline_or_end_call(cx);
                            })),
                        )
                        .child(
                            ui::Button::plain(
                                &theme,
                                "call-hud-reply-sms",
                                ui::ButtonVariant::Gray,
                                ui::ButtonSize::Xs,
                            )
                            .label("💬 Reply SMS")
                            .on_click(cx.listener(
                                move |this, _, _, cx| {
                                    this.reply_quick_sms_and_decline(sms_caller.clone(), cx);
                                },
                            )),
                        ),
                ),
        )
    }

    fn render_quick_dialer_drawer(&self, cx: &mut Context<Self>) -> gpui::Div {
        let theme = self.theme;
        let current_num = self.dial_number.clone();
        let dial_target = current_num.clone();
        let keys = ["1", "2", "3", "4", "5", "6", "7", "8", "9", "+", "0", "#"];

        div()
            .flex()
            .flex_col()
            .gap(px(6.))
            .p(px(10.))
            .rounded(px(10.))
            .border_1()
            .border_color(Hsla::from(theme.gray_5))
            .bg(Hsla::from(theme.gray_2))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .gap(px(6.))
                    .child(
                        div()
                            .flex_1()
                            .px(px(10.))
                            .py(px(6.))
                            .rounded(px(6.))
                            .bg(Hsla::from(theme.gray_1))
                            .border_1()
                            .border_color(Hsla::from(theme.gray_5))
                            .text_size(px(13.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(if current_num.is_empty() {
                                "Type number (0-9, +) or click keypad...".to_string()
                            } else {
                                current_num
                            }),
                    )
                    .child(
                        ui::Button::plain(
                            &theme,
                            "dialer-backspace",
                            ui::ButtonVariant::Gray,
                            ui::ButtonSize::Xs,
                        )
                        .label("⌫")
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.dial_number.pop();
                            cx.notify();
                        })),
                    )
                    .child(
                        ui::Button::plain(
                            &theme,
                            "dialer-call-now",
                            ui::ButtonVariant::Primary,
                            ui::ButtonSize::Xs,
                        )
                        .label("🟢 Call Now")
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.dial_phone_number(dial_target.clone(), cx);
                        })),
                    ),
            )
            .child(div().flex().flex_row().flex_wrap().gap(px(4.)).children(
                keys.into_iter().enumerate().map(|(i, k)| {
                    ui::Button::plain(
                        &theme,
                        ("dial-key", i),
                        ui::ButtonVariant::Gray,
                        ui::ButtonSize::Xs,
                    )
                    .label(k)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.dial_number.push_str(k);
                        cx.notify();
                    }))
                }),
            ))
            .children((!self.recent_calls.is_empty()).then(|| {
                div()
                    .flex()
                    .flex_col()
                    .gap(px(4.))
                    .child(
                        div()
                            .text_size(px(10.5))
                            .text_color(Hsla::from(theme.gray_11))
                            .child("Recent Call Log (Click to Redial):"),
                    )
                    .children(
                        self.recent_calls
                            .iter()
                            .take(4)
                            .enumerate()
                            .map(|(idx, entry)| {
                                let num = entry.number.clone();
                                let label = if entry.name.is_empty() {
                                    format!("📞 {} ({})", entry.number, entry.call_type)
                                } else {
                                    format!(
                                        "📞 {} · {} ({})",
                                        entry.name, entry.number, entry.call_type
                                    )
                                };
                                div()
                                    .id(("recent-call", idx))
                                    .px(px(8.))
                                    .py(px(4.))
                                    .rounded(px(6.))
                                    .cursor_pointer()
                                    .bg(Hsla::from(theme.gray_3))
                                    .text_size(px(11.))
                                    .child(label)
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.dial_number = num.clone();
                                        this.dial_phone_number(num.clone(), cx);
                                    }))
                            }),
                    )
            }))
    }

    fn on_canvas_mouse_move(&mut self, event: &MouseMoveEvent, _cx: &mut Context<Self>) {
        if self.drag_start.is_some() {
            let x = (f32::from(event.position.x) * 3.0).clamp(0.0, self.device_width as f32) as u32;
            let y =
                (f32::from(event.position.y) * 3.0).clamp(0.0, self.device_height as f32) as u32;
            let pkt = encode_inject_touch_event(
                2,
                0,
                x,
                y,
                self.device_width as u16,
                self.device_height as u16,
                0xFFFF,
                1,
            );
            let _ = self.send_control_bytes(&pkt);
        }
    }

    fn on_canvas_mouse_up(&mut self, event: &MouseUpEvent, cx: &mut Context<Self>) {
        if event.button != MouseButton::Left {
            return;
        }
        let Some((sx, sy, started)) = self.drag_start.take() else {
            return;
        };
        let ex = f32::from(event.position.x);
        let ey = f32::from(event.position.y);
        let dist = ((ex - sx).powi(2) + (ey - sy).powi(2)).sqrt();
        let scale_x = (self.device_width as f32 / 340.0).max(1.0);
        let scale_y = (self.device_height as f32 / 520.0).max(1.0);
        let dev_sx = (sx * scale_x).clamp(0.0, self.device_width as f32) as u32;
        let dev_sy = (sy * scale_y).clamp(0.0, self.device_height as f32) as u32;
        let dev_ex = (ex * scale_x).clamp(0.0, self.device_width as f32) as u32;
        let dev_ey = (ey * scale_y).clamp(0.0, self.device_height as f32) as u32;

        if dist < 12.0 {
            let down = encode_inject_touch_event(
                0,
                0,
                dev_ex,
                dev_ey,
                self.device_width as u16,
                self.device_height as u16,
                0xFFFF,
                1,
            );
            let up = encode_inject_touch_event(
                1,
                0,
                dev_ex,
                dev_ey,
                self.device_width as u16,
                self.device_height as u16,
                0,
                0,
            );
            if !(self.send_control_bytes(&down) && self.send_control_bytes(&up)) {
                self.run_adb_async(
                    vec![
                        "shell".into(),
                        "input".into(),
                        "tap".into(),
                        dev_ex.to_string(),
                        dev_ey.to_string(),
                    ],
                    cx,
                );
            }
            if self.dual_canvas_mode
                && self.sync_master_gestures
                && let Some(sec) = self.secondary_serial.clone()
                && !sec.starts_with("sim:")
                && sec != "apple:iphone-mirroring"
            {
                cx.background_executor()
                    .spawn(async move {
                        let adb = find_adb_binary();
                        let _ = Command::new(adb)
                            .args([
                                "-s",
                                &sec,
                                "shell",
                                "input",
                                "tap",
                                &dev_ex.to_string(),
                                &dev_ey.to_string(),
                            ])
                            .output();
                    })
                    .detach();
            }
        } else {
            let dur_ms = started.elapsed().as_millis().clamp(120, 800);
            self.run_adb_async(
                vec![
                    "shell".into(),
                    "input".into(),
                    "swipe".into(),
                    dev_sx.to_string(),
                    dev_sy.to_string(),
                    dev_ex.to_string(),
                    dev_ey.to_string(),
                    dur_ms.to_string(),
                ],
                cx,
            );
            if self.dual_canvas_mode
                && self.sync_master_gestures
                && let Some(sec) = self.secondary_serial.clone()
                && !sec.starts_with("sim:")
                && sec != "apple:iphone-mirroring"
            {
                cx.background_executor()
                    .spawn(async move {
                        let adb = find_adb_binary();
                        let _ = Command::new(adb)
                            .args([
                                "-s",
                                &sec,
                                "shell",
                                "input",
                                "swipe",
                                &dev_sx.to_string(),
                                &dev_sy.to_string(),
                                &dev_ex.to_string(),
                                &dev_ey.to_string(),
                                &dur_ms.to_string(),
                            ])
                            .output();
                    })
                    .detach();
            }
        }
    }

    fn on_key_down(&mut self, event: &KeyDownEvent, cx: &mut Context<Self>) {
        if event.keystroke.modifiers.platform && event.keystroke.key.as_str() == "k" {
            self.dialer_open = !self.dialer_open;
            if self.dialer_open {
                self.refresh_recent_calls(cx);
            }
            cx.notify();
            return;
        }
        if event.keystroke.modifiers.platform && event.keystroke.key.as_str() == "v" {
            self.sync_mac_clipboard_to_phone(cx);
            return;
        }
        if self.dialer_open {
            match event.keystroke.key.as_str() {
                "backspace" => {
                    self.dial_number.pop();
                    cx.notify();
                    return;
                }
                "enter" => {
                    let num = self.dial_number.clone();
                    self.dial_phone_number(num, cx);
                    return;
                }
                "escape" => {
                    self.dialer_open = false;
                    cx.notify();
                    return;
                }
                other
                    if other.len() == 1
                        && other
                            .chars()
                            .all(|c| c.is_ascii_digit() || matches!(c, '+' | '*' | '#')) =>
                {
                    self.dial_number.push_str(other);
                    cx.notify();
                    return;
                }
                _ => {}
            }
        }
        match event.keystroke.key.as_str() {
            "backspace" => self.send_android_keyevent(67, "Backspace", cx),
            "enter" => self.send_android_keyevent(66, "Enter", cx),
            "escape" => {
                self.trigger_panel(AndroidPanelAction::CollapsePanels, cx);
                self.send_android_keyevent(4, "Back", cx);
            }
            "space" => self.send_android_keyevent(62, "Space", cx),
            other if other.len() == 1 => {
                let pkt = encode_inject_text(other);
                if !self.send_control_bytes(&pkt) {
                    self.run_adb_async(
                        vec![
                            "shell".into(),
                            "input".into(),
                            "text".into(),
                            other.to_string(),
                        ],
                        cx,
                    );
                }
            }
            _ => {}
        }
    }
}

impl Render for AndroidWindow {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.theme = Theme::for_window(window, cx, false);
        let theme = self.theme;
        let stealth = self.screen_off_stealth;
        let mode = self.connection_mode;
        let preset = self.quality_preset;
        let audio_fwd = self.audio_forwarding;
        let auto_dim = self.auto_dim_oled;

        div()
            .id("android-mirror-root")
            .size_full()
            .flex()
            .flex_col()
            .gap(px(8.))
            .p(px(12.))
            .bg(Hsla::from(theme.gray_1))
            .text_color(Hsla::from(theme.gray_12))
            .on_drop(cx.listener(|this, paths: &gpui::ExternalPaths, _, cx| {
                this.handle_dropped_paths(paths.paths(), cx);
            }))
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, _, cx| {
                this.on_key_down(event, cx);
            }))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .gap(px(6.))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .child(
                                div()
                                    .text_size(px(14.))
                                    .font_weight(FontWeight::BOLD)
                                    .child("Apple (iOS/iPhone) & Android Pro Mirror Hub"),
                            )
                            .child(
                                div()
                                    .text_size(px(11.))
                                    .text_color(Hsla::from(theme.gray_11))
                                    .child(SharedString::from(self.status_notice.clone())),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .gap(px(6.))
                            .child(
                                ui::Button::plain(
                                    &theme,
                                    "android-open-dialer",
                                    if self.dialer_open {
                                        ui::ButtonVariant::Primary
                                    } else {
                                        ui::ButtonVariant::Gray
                                    },
                                    ui::ButtonSize::Xs,
                                )
                                .label("📞 Dialer (⌘K)")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.dialer_open = !this.dialer_open;
                                    if this.dialer_open {
                                        this.refresh_recent_calls(cx);
                                    }
                                    cx.notify();
                                })),
                            )
                            .child(
                                ui::Button::plain(
                                    &theme,
                                    "android-clean-status-bar",
                                    ui::ButtonVariant::Gray,
                                    ui::ButtonSize::Xs,
                                )
                                .label("✨ 9:41 Demo Bar")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.apply_clean_demo_status_bar(cx);
                                })),
                            )
                            .child(
                                ui::Button::plain(
                                    &theme,
                                    "android-refresh-devices",
                                    ui::ButtonVariant::Gray,
                                    ui::ButtonSize::Xs,
                                )
                                .label("Refresh Devices")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.refresh_devices(cx);
                                })),
                            )
                            .child(
                                ui::Button::plain(
                                    &theme,
                                    "android-connect-mirror",
                                    ui::ButtonVariant::Primary,
                                    ui::ButtonSize::Xs,
                                )
                                .label(if self.mirroring_active {
                                    "Reconnect Mirror"
                                } else {
                                    "Connect & Mirror"
                                })
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.start_mirroring(cx);
                                })),
                            ),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(6.))
                    .children(self.devices.iter().enumerate().map(|(idx, dev)| {
                        let selected = Some(&dev.serial) == self.selected_serial.as_ref();
                        let serial = dev.serial.clone();
                        div()
                            .id(("android-dev", idx))
                            .px(px(8.))
                            .py(px(4.))
                            .rounded(px(6.))
                            .cursor_pointer()
                            .border_1()
                            .border_color(if selected {
                                Hsla::from(theme.blue_9)
                            } else {
                                Hsla::from(theme.gray_5)
                            })
                            .bg(if selected {
                                Theme::with_alpha(theme.blue_9, 0.16)
                            } else {
                                Hsla::from(theme.gray_2)
                            })
                            .text_size(px(11.))
                            .child(format!(
                                "{} {} ({})",
                                match dev.platform {
                                    MobilePlatform::Android => "🤖",
                                    MobilePlatform::AppleIosSimulator
                                    | MobilePlatform::ApplePhysicalDevice => "🍎",
                                },
                                dev.model,
                                dev.transport
                            ))
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.selected_serial = Some(serial.clone());
                                this.start_mirroring(cx);
                            }))
                    })),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap(px(4.))
                    .children(
                        [
                            AndroidConnectionMode::UsbCable,
                            AndroidConnectionMode::WirelessWifi,
                            AndroidConnectionMode::BluetoothHid,
                        ]
                        .into_iter()
                        .enumerate()
                        .map(|(i, m)| {
                            div().flex_1().child(
                                ui::Button::plain(
                                    &theme,
                                    ("android-conn-mode", i),
                                    if mode == m {
                                        ui::ButtonVariant::Primary
                                    } else {
                                        ui::ButtonVariant::Gray
                                    },
                                    ui::ButtonSize::Xs,
                                )
                                .label(m.label())
                                .full_width()
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.switch_connection_mode(m, cx);
                                })),
                            )
                        }),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap(px(4.))
                    .children(
                        [
                            AndroidQualityPreset::Native4K,
                            AndroidQualityPreset::Fhd1080p,
                            AndroidQualityPreset::Saver720p,
                        ]
                        .into_iter()
                        .enumerate()
                        .map(|(i, q)| {
                            div().flex_1().child(
                                ui::Button::plain(
                                    &theme,
                                    ("android-quality", i),
                                    if preset == q {
                                        ui::ButtonVariant::Primary
                                    } else {
                                        ui::ButtonVariant::Gray
                                    },
                                    ui::ButtonSize::Xs,
                                )
                                .label(q.label())
                                .full_width()
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.quality_preset = q;
                                    this.status_notice =
                                        format!("Stream quality set to {}.", q.label());
                                    cx.notify();
                                })),
                            )
                        }),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap(px(6.))
                    .child(
                        div().flex_1().child(
                            ui::Button::plain(
                                &theme,
                                "android-stealth-screen-off",
                                if stealth {
                                    ui::ButtonVariant::Primary
                                } else {
                                    ui::ButtonVariant::Gray
                                },
                                ui::ButtonSize::Xs,
                            )
                            .label(if stealth {
                                "🕶 Screen-Off Stealth: ON"
                            } else {
                                "🕶 Screen-Off Stealth: OFF"
                            })
                            .full_width()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.toggle_screen_off_stealth(cx);
                            })),
                        ),
                    )
                    .child(
                        div().flex_1().child(
                            ui::Button::plain(
                                &theme,
                                "android-audio-forward",
                                if audio_fwd {
                                    ui::ButtonVariant::Primary
                                } else {
                                    ui::ButtonVariant::Gray
                                },
                                ui::ButtonSize::Xs,
                            )
                            .label(if audio_fwd {
                                "🔊 Stereo Audio: ON"
                            } else {
                                "🔇 Stereo Audio: OFF"
                            })
                            .full_width()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.audio_forwarding = !this.audio_forwarding;
                                cx.notify();
                            })),
                        ),
                    )
                    .child(
                        div().flex_1().child(
                            ui::Button::plain(
                                &theme,
                                "android-oled-dim",
                                if auto_dim {
                                    ui::ButtonVariant::Primary
                                } else {
                                    ui::ButtonVariant::Gray
                                },
                                ui::ButtonSize::Xs,
                            )
                            .label(if auto_dim {
                                "🌙 1% OLED Dim: ON"
                            } else {
                                "🌙 1% OLED Dim: OFF"
                            })
                            .full_width()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.auto_dim_oled = !this.auto_dim_oled;
                                cx.notify();
                            })),
                        ),
                    )
                    .child(
                        div().flex_1().child(
                            ui::Button::plain(
                                &theme,
                                "android-pause-rec-on-call",
                                if self.pause_recording_on_call {
                                    ui::ButtonVariant::Primary
                                } else {
                                    ui::ButtonVariant::Gray
                                },
                                ui::ButtonSize::Xs,
                            )
                            .label(if self.pause_recording_on_call {
                                "⏸ Auto-Pause on Call: ON"
                            } else {
                                "⏸ Auto-Pause on Call: OFF"
                            })
                            .full_width()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.pause_recording_on_call = !this.pause_recording_on_call;
                                let _ = crate::store::set_pause_recording_on_incoming_call(
                                    this.pause_recording_on_call,
                                );
                                cx.notify();
                            })),
                        ),
                    ),
            )
            .children(self.render_call_hud_banner(cx))
            .children(self.detected_otp.clone().map(|otp| {
                let otp_copy = otp.clone();
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .px(px(10.))
                    .py(px(6.))
                    .rounded(px(8.))
                    .border_1()
                    .border_color(Hsla::from(theme.blue_9))
                    .bg(Theme::with_alpha(theme.blue_9, 0.18))
                    .child(
                        div()
                            .text_size(px(11.5))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(format!("🔐 Smart 2FA / OTP Detected: {otp}")),
                    )
                    .child(
                        ui::Button::plain(
                            &theme,
                            "android-paste-otp-pill",
                            ui::ButtonVariant::Primary,
                            ui::ButtonSize::Xs,
                        )
                        .label(SharedString::from(format!("📋 Paste OTP: {otp}")))
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.copy_detected_otp(otp_copy.clone(), cx);
                        })),
                    )
            }))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap(px(4.))
                    .child(
                        div().flex_1().child(
                            ui::Button::plain(
                                &theme,
                                "android-dual-canvas-toggle",
                                if self.dual_canvas_mode {
                                    ui::ButtonVariant::Primary
                                } else {
                                    ui::ButtonVariant::Gray
                                },
                                ui::ButtonSize::Xs,
                            )
                            .label(if self.dual_canvas_mode {
                                "📱📱 Dual Side-by-Side: ON"
                            } else {
                                "📱 Single Device"
                            })
                            .full_width()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.dual_canvas_mode = !this.dual_canvas_mode;
                                if this.secondary_serial.is_none() {
                                    this.secondary_serial = this
                                        .devices
                                        .iter()
                                        .find(|d| Some(&d.serial) != this.selected_serial.as_ref())
                                        .map(|d| d.serial.clone());
                                }
                                cx.notify();
                            })),
                        ),
                    )
                    .child(
                        div().flex_1().child(
                            ui::Button::plain(
                                &theme,
                                "android-sync-master-touch",
                                if self.sync_master_gestures {
                                    ui::ButtonVariant::Primary
                                } else {
                                    ui::ButtonVariant::Gray
                                },
                                ui::ButtonSize::Xs,
                            )
                            .label("🔗 Sync Master Touch")
                            .full_width()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.sync_master_gestures = !this.sync_master_gestures;
                                cx.notify();
                            })),
                        ),
                    )
                    .child(
                        div().flex_1().child(
                            ui::Button::plain(
                                &theme,
                                "android-phone-studio-webcam",
                                if self.phone_as_studio_webcam {
                                    ui::ButtonVariant::Primary
                                } else {
                                    ui::ButtonVariant::Gray
                                },
                                ui::ButtonSize::Xs,
                            )
                            .label("📷 Use Phone as Studio Cam")
                            .full_width()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.toggle_phone_as_studio_webcam(cx);
                            })),
                        ),
                    )
                    .child(
                        div().flex_1().child(
                            ui::Button::plain(
                                &theme,
                                "android-perf-hud-toggle",
                                if self.show_perf_hud {
                                    ui::ButtonVariant::Primary
                                } else {
                                    ui::ButtonVariant::Gray
                                },
                                ui::ButtonSize::Xs,
                            )
                            .label("📊 Perf HUD")
                            .full_width()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.show_perf_hud = !this.show_perf_hud;
                                cx.notify();
                            })),
                        ),
                    )
                    .child(
                        div().flex_1().child(
                            ui::Button::plain(
                                &theme,
                                "android-touch-ripples-toggle",
                                if self.show_touch_ripples {
                                    ui::ButtonVariant::Primary
                                } else {
                                    ui::ButtonVariant::Gray
                                },
                                ui::ButtonSize::Xs,
                            )
                            .label("👆 Touch Ripples")
                            .full_width()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.show_touch_ripples = !this.show_touch_ripples;
                                cx.notify();
                            })),
                        ),
                    )
                    .child(
                        div().flex_1().child(
                            ui::Button::plain(
                                &theme,
                                "android-bezel-skin-cycle",
                                ui::ButtonVariant::Gray,
                                ui::ButtonSize::Xs,
                            )
                            .label(SharedString::from(format!(
                                "🛡 {}",
                                self.bezel_skin.label()
                            )))
                            .full_width()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.bezel_skin = match this.bezel_skin {
                                    MobileBezelSkin::TitaniumDark => {
                                        MobileBezelSkin::TitaniumSilver
                                    }
                                    MobileBezelSkin::TitaniumSilver => MobileBezelSkin::RawMinimal,
                                    MobileBezelSkin::RawMinimal => MobileBezelSkin::TitaniumDark,
                                };
                                cx.notify();
                            })),
                        ),
                    ),
            )
            .children(self.dialer_open.then(|| self.render_quick_dialer_drawer(cx)))
            .child(
                div()
                    .id("android-viewport-canvas")
                    .flex_1()
                    .w_full()
                    .rounded(match self.bezel_skin {
                        MobileBezelSkin::TitaniumDark | MobileBezelSkin::TitaniumSilver => px(26.),
                        MobileBezelSkin::RawMinimal => px(12.),
                    })
                    .border(match self.bezel_skin {
                        MobileBezelSkin::TitaniumDark | MobileBezelSkin::TitaniumSilver => px(6.),
                        MobileBezelSkin::RawMinimal => px(2.),
                    })
                    .border_color(match self.bezel_skin {
                        MobileBezelSkin::TitaniumDark => gpui::rgb(0x3f3f46).into(),
                        MobileBezelSkin::TitaniumSilver => gpui::rgb(0xd4d4d8).into(),
                        MobileBezelSkin::RawMinimal => Hsla::from(theme.gray_5),
                    })
                    .bg(gpui::black())
                    .relative()
                    .flex()
                    .items_center()
                    .justify_center()
                    .overflow_hidden()
                    .cursor_pointer()
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, ev, _, cx| this.on_canvas_mouse_down(ev, cx)),
                    )
                    .on_mouse_down(
                        MouseButton::Right,
                        cx.listener(|this, ev, _, cx| this.on_canvas_mouse_down(ev, cx)),
                    )
                    .on_mouse_down(
                        MouseButton::Middle,
                        cx.listener(|this, ev, _, cx| this.on_canvas_mouse_down(ev, cx)),
                    )
                    .on_mouse_move(cx.listener(|this, ev, _, cx| this.on_canvas_mouse_move(ev, cx)))
                    .on_mouse_up(
                        MouseButton::Left,
                        cx.listener(|this, ev, _, cx| this.on_canvas_mouse_up(ev, cx)),
                    )
                    .children(
                        (self.bezel_skin != MobileBezelSkin::RawMinimal).then(|| {
                            div()
                                .absolute()
                                .top(px(8.))
                                .w(px(76.))
                                .h(px(18.))
                                .rounded_full()
                                .bg(gpui::rgb(0x09090b))
                                .border_1()
                                .border_color(gpui::rgb(0x27272a))
                        }),
                    )
                    .children(
                        self.show_touch_ripples
                            .then_some(self.last_touch_ripple)
                            .flatten()
                            .and_then(|(rx, ry, ts)| {
                                let age = ts.elapsed().as_secs_f32();
                                (age < 0.85).then(|| {
                                    let sample = cap_rendering::TouchRippleSample {
                                        x: rx,
                                        y: ry,
                                        age_sec: age,
                                        duration_sec: 0.85,
                                    };
                                    let radius = sample.radius_px();
                                    let alpha = sample.opacity();
                                    div()
                                        .absolute()
                                        .left(px((rx - radius).max(4.0)))
                                        .top(px((ry - 210.0 - radius).max(4.0)))
                                        .w(px(radius * 2.0))
                                        .h(px(radius * 2.0))
                                        .rounded_full()
                                        .border_2()
                                        .border_color(gpui::hsla(0.55, 0.92, 0.60, alpha))
                                        .bg(gpui::hsla(0.55, 0.92, 0.60, alpha * 0.28))
                                })
                            }),
                    )
                    .children(self.show_perf_hud.then(|| {
                        let p = self.perf_telemetry;
                        div()
                            .absolute()
                            .top(px(10.))
                            .right(px(10.))
                            .px(px(8.))
                            .py(px(4.))
                            .rounded(px(6.))
                            .bg(gpui::hsla(0.0, 0.0, 0.05, 0.82))
                            .border_1()
                            .border_color(Hsla::from(theme.gray_6))
                            .text_size(px(10.5))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(gpui::rgb(0x34d399))
                            .child(format!(
                                "{} FPS · CPU {}% · {}°C · Battery {}%",
                                p.fps, p.cpu_percent, p.temp_c, p.battery_percent
                            ))
                    }))
                    .child(if self.dual_canvas_mode {
                        div()
                            .size_full()
                            .flex()
                            .flex_row()
                            .gap(px(6.))
                            .p(px(6.))
                            .child(
                                div()
                                    .flex_1()
                                    .h_full()
                                    .rounded(px(12.))
                                    .border_1()
                                    .border_color(Hsla::from(theme.gray_6))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .overflow_hidden()
                                    .child(if let Some(frame) = &self.frame_image {
                                        img(frame.clone())
                                            .w_full()
                                            .h_full()
                                            .object_fit(gpui::ObjectFit::Contain)
                                            .into_any_element()
                                    } else {
                                        div()
                                            .text_size(px(11.5))
                                            .text_color(Hsla::from(theme.gray_9))
                                            .child("Primary Device (Android / iOS)")
                                            .into_any_element()
                                    }),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .h_full()
                                    .rounded(px(12.))
                                    .border_1()
                                    .border_color(Hsla::from(theme.gray_6))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .overflow_hidden()
                                    .child(if let Some(sec_frame) = &self.secondary_frame_image {
                                        img(sec_frame.clone())
                                            .w_full()
                                            .h_full()
                                            .object_fit(gpui::ObjectFit::Contain)
                                            .into_any_element()
                                    } else {
                                        div()
                                            .text_size(px(11.5))
                                            .text_color(Hsla::from(theme.gray_9))
                                            .child(
                                                "Secondary Device (Synchronized Master Touch Active)",
                                            )
                                            .into_any_element()
                                    }),
                            )
                            .into_any_element()
                    } else if let Some(frame) = &self.frame_image {
                        img(frame.clone())
                            .w_full()
                            .h_full()
                            .object_fit(gpui::ObjectFit::Contain)
                            .into_any_element()
                    } else {
                        div()
                            .flex()
                            .flex_col()
                            .items_center()
                            .gap(px(8.))
                            .p(px(20.))
                            .text_center()
                            .child(
                                div()
                                    .text_size(px(14.))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(gpui::white())
                                    .child("Android & iOS 60 FPS Interactive Viewport"),
                            )
                            .child(
                                div()
                                    .text_size(px(11.5))
                                    .text_color(Hsla::from(theme.gray_9))
                                    .child(
                                        "Left-Click = Tap · Drag = Smooth Scroll · Right-Click = Back · Middle-Click = Home · Type on Mac Keyboard to inject Unicode text",
                                    ),
                            )
                            .into_any_element()
                    }),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap(px(4.))
                    .child(
                        div().flex_1().child(
                            ui::Button::plain(
                                &theme,
                                "android-btn-back",
                                ui::ButtonVariant::Gray,
                                ui::ButtonSize::Xs,
                            )
                            .label("◀ Back")
                            .full_width()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.send_android_keyevent(4, "Back", cx);
                            })),
                        ),
                    )
                    .child(
                        div().flex_1().child(
                            ui::Button::plain(
                                &theme,
                                "android-btn-home",
                                ui::ButtonVariant::Gray,
                                ui::ButtonSize::Xs,
                            )
                            .label("● Home")
                            .full_width()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.send_android_keyevent(3, "Home", cx);
                            })),
                        ),
                    )
                    .child(
                        div().flex_1().child(
                            ui::Button::plain(
                                &theme,
                                "android-btn-recents",
                                ui::ButtonVariant::Gray,
                                ui::ButtonSize::Xs,
                            )
                            .label("■ Recents")
                            .full_width()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.send_android_keyevent(187, "App Switcher", cx);
                            })),
                        ),
                    )
                    .child(
                        div().flex_1().child(
                            ui::Button::plain(
                                &theme,
                                "android-btn-rotate",
                                ui::ButtonVariant::Gray,
                                ui::ButtonSize::Xs,
                            )
                            .label("↻ Rotate")
                            .full_width()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.toggle_rotation(cx);
                            })),
                        ),
                    )
                    .child(
                        div().flex_1().child(
                            ui::Button::plain(
                                &theme,
                                "android-btn-notifications",
                                ui::ButtonVariant::Gray,
                                ui::ButtonSize::Xs,
                            )
                            .label("▼ Alerts")
                            .full_width()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.trigger_panel(AndroidPanelAction::ExpandNotifications, cx);
                            })),
                        ),
                    )
                    .child(
                        div().flex_1().child(
                            ui::Button::plain(
                                &theme,
                                "android-btn-quick-settings",
                                ui::ButtonVariant::Gray,
                                ui::ButtonSize::Xs,
                            )
                            .label("⚙ Shade")
                            .full_width()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.trigger_panel(AndroidPanelAction::ExpandSettings, cx);
                            })),
                        ),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap(px(6.))
                    .child(
                        div().flex_1().child(
                            ui::Button::plain(
                                &theme,
                                "android-install-apk",
                                ui::ButtonVariant::Gray,
                                ui::ButtonSize::Xs,
                            )
                            .label("📦 Install .APK")
                            .full_width()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.pick_and_install_apk(cx);
                            })),
                        ),
                    )
                    .child(
                        div().flex_1().child(
                            ui::Button::plain(
                                &theme,
                                "android-push-file",
                                ui::ButtonVariant::Gray,
                                ui::ButtonSize::Xs,
                            )
                            .label("📁 Push File")
                            .full_width()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.pick_and_push_file(cx);
                            })),
                        ),
                    )
                    .child(
                        div().flex_1().child(
                            ui::Button::plain(
                                &theme,
                                "android-sync-clipboard",
                                ui::ButtonVariant::Gray,
                                ui::ButtonSize::Xs,
                            )
                            .label("📋 Mac → Phone")
                            .full_width()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.sync_mac_clipboard_to_phone(cx);
                            })),
                        ),
                    )
                    .child(
                        div().flex_1().child(
                            ui::Button::plain(
                                &theme,
                                "android-pull-clipboard",
                                ui::ButtonVariant::Gray,
                                ui::ButtonSize::Xs,
                            )
                            .label("📲 Phone → Mac")
                            .full_width()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.pull_phone_clipboard_to_mac(cx);
                            })),
                        ),
                    )
                    .child(
                        div().flex_1().child(
                            ui::Button::plain(
                                &theme,
                                "android-open-url-device",
                                ui::ButtonVariant::Gray,
                                ui::ButtonSize::Xs,
                            )
                            .label("🌐 Open URL")
                            .full_width()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.open_clipboard_url_on_device(cx);
                            })),
                        ),
                    )
                    .child(
                        div().flex_1().child(
                            ui::Button::plain(
                                &theme,
                                "android-record-demo",
                                ui::ButtonVariant::Primary,
                                ui::ButtonSize::Xs,
                            )
                            .label("🎬 Record Mobile Demo")
                            .full_width()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.record_mobile_demo_into_studio(cx);
                            })),
                        ),
                    ),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scrcpy_video_and_frame_headers_parse_big_endian() {
        let mut raw = vec![0x00u8];
        let mut name = [0u8; 64];
        name[..8].copy_from_slice(b"Pixel_9_");
        raw.extend_from_slice(&name);
        raw.extend_from_slice(&SCRCPY_CODEC_H264.to_be_bytes());
        raw.extend_from_slice(&1080u32.to_be_bytes());
        raw.extend_from_slice(&2400u32.to_be_bytes());
        let parsed = parse_video_stream_header(&raw).unwrap();
        assert_eq!(parsed.device_name, "Pixel_9_");
        assert_eq!(parsed.codec_id, SCRCPY_CODEC_H264);
        assert_eq!(parsed.initial_width, 1080);
        assert_eq!(parsed.initial_height, 2400);

        let mut fhdr = [0u8; 12];
        let pts_flags: u64 = (1 << 62) | 16_666;
        fhdr[..8].copy_from_slice(&pts_flags.to_be_bytes());
        fhdr[8..12].copy_from_slice(&4096u32.to_be_bytes());
        let frame = parse_frame_header(&fhdr);
        assert!(!frame.config_packet);
        assert!(frame.keyframe);
        assert_eq!(frame.pts_us, 16_666);
        assert_eq!(frame.packet_size, 4096);
    }

    #[test]
    fn scrcpy_control_packets_encode_exact_protocol_bytes() {
        let touch = encode_inject_touch_event(0, 0, 540, 1200, 1080, 2400, 0xFFFF, 1);
        assert_eq!(touch[0], 0x02);
        assert_eq!(touch[1], 0);
        assert_eq!(
            u32::from_be_bytes([touch[10], touch[11], touch[12], touch[13]]),
            540
        );
        assert_eq!(
            u32::from_be_bytes([touch[14], touch[15], touch[16], touch[17]]),
            1200
        );

        let stealth_off = encode_set_display_power(false);
        assert_eq!(stealth_off, [0x0a, 0x00]);
        let stealth_on = encode_set_display_power(true);
        assert_eq!(stealth_on, [0x0a, 0x01]);

        let clip = encode_set_clipboard(42, true, "hello");
        assert_eq!(clip[0], 0x09);
        assert_eq!(clip[9], 1);
        assert_eq!(&clip[14..], b"hello");
    }

    #[test]
    fn adb_devices_output_parses_usb_wifi_and_apple_simulators() {
        let sample = "List of devices attached\n\
                      R5CT90ABCDE          device usb:1-1 product:dm3q model:SM_S918B device:dm3q\n\
                      192.168.1.44:5555    device product:husky model:Pixel_8_Pro device:husky\n\
                      ZY225ABC99           unauthorized usb:1-2 transport_id:4\n";
        let list = parse_adb_devices_output(sample);
        assert_eq!(list.len(), 3);
        assert_eq!(list[0].model, "SM S918B");
        assert_eq!(list[0].transport, "USB");
        assert!(list[0].authorized);
        assert_eq!(list[1].model, "Pixel 8 Pro");
        assert_eq!(list[1].transport, "Wi-Fi");
        assert!(!list[2].authorized);
        assert_eq!(list[2].transport, "Auth Needed");

        let sim_sample = "== Devices ==\n-- iOS 18.2 --\n    iPhone 16 Pro (A1B2C3D4-E5F6-7890-1234-567890ABCDEF) (Booted)\n";
        let sims = parse_simctl_booted_devices(sim_sample);
        assert_eq!(sims.len(), 1);
        assert_eq!(sims[0].model, "iPhone 16 Pro");
        assert_eq!(sims[0].platform, MobilePlatform::AppleIosSimulator);
    }

    #[test]
    fn telephony_registry_and_call_log_parse_cleanly() {
        let dump = "  Phone Id=0\n    mCallState=1\n    mCallIncomingNumber=+14155550199\n";
        assert_eq!(
            parse_telephony_registry_dump(dump),
            TelephonyCallState::Ringing {
                caller: "+14155550199".to_string()
            }
        );

        let log_sample = "Row: 0 number=+14155550199, name=Alex Rivera, type=1\n\
                          Row: 1 number=+16505550123, name=NULL, type=2\n";
        let logs = parse_call_log_query_output(log_sample);
        assert_eq!(logs.len(), 2);
        assert_eq!(logs[0].name, "Alex Rivera");
        assert_eq!(logs[0].call_type, "Incoming");
        assert_eq!(logs[1].number, "+16505550123");
        assert_eq!(logs[1].call_type, "Outgoing");
    }

    #[test]
    fn otp_extractor_and_performance_telemetry_parse_accurately() {
        assert_eq!(
            extract_otp_code_from_notification("Your verification code is 593821. Do not share."),
            Some("593821".to_string())
        );
        assert_eq!(
            extract_otp_code_from_notification("Meeting at 1400 in room 204"),
            None
        );

        let bat = "Current Battery Service state:\n  level: 91\n  temperature: 335\n";
        let cpu = "Load: 2.1 / 1.8 / 1.5\n18.4% TOTAL: 12% user + 6.4% kernel\n";
        let perf = parse_battery_and_cpu_dumps(bat, cpu);
        assert_eq!(perf.battery_percent, 91);
        assert_eq!(perf.temp_c, 33);
        assert_eq!(perf.cpu_percent, 18);
    }
}
