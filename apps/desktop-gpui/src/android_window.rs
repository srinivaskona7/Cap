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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AndroidDevice {
    pub serial: String,
    pub model: String,
    pub transport: String,
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
        if state != "device" {
            continue;
        }
        let mut model = serial.to_string();
        for token in parts {
            if let Some(m) = token.strip_prefix("model:") {
                model = m.replace('_', " ");
            }
        }
        let transport = if serial.contains(':') {
            "Wi-Fi".to_string()
        } else {
            "USB".to_string()
        };
        list.push(AndroidDevice {
            serial: serial.to_string(),
            model,
            transport,
        });
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
    let adb = find_adb_binary();
    let Ok(out) = Command::new(adb).args(["devices", "-l"]).output() else {
        return Vec::new();
    };
    parse_adb_devices_output(&String::from_utf8_lossy(&out.stdout))
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
            self.status_notice = "Connect an Android device before starting mirroring.".to_string();
            cx.notify();
            return;
        };
        self.mirroring_active = true;
        let wake = self.wake_on_connect;
        let auto_dim = self.auto_dim_oled;
        let stealth = self.screen_off_stealth;
        let max_size = self.quality_preset.max_size();
        let bitrate = self.quality_preset.bitrate_bps();
        let control_socket = self.control_socket.clone();
        if self.auto_clipboard_sync {
            self.sync_mac_clipboard_to_phone(cx);
        }

        self.status_notice = format!(
            "Mirroring {serial} ({} · {}px @ {} Mbps{})...",
            self.quality_preset.label(),
            max_size,
            bitrate / 1_000_000,
            if stealth {
                " · Screen-Off Stealth"
            } else {
                ""
            }
        );

        let serial_init = serial.clone();
        cx.background_executor()
            .spawn(async move {
                let adb = find_adb_binary();
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

        self._mirror_task = Some(cx.spawn(async move |this, cx| {
            loop {
                let active_serial = cx.update(|cx| {
                    this.read_with(cx, |view, _| {
                        view.mirroring_active
                            .then(|| view.selected_serial.clone())
                            .flatten()
                    })
                    .ok()
                    .flatten()
                });
                let Some(serial_now) = active_serial else {
                    break;
                };
                let frame = cx
                    .background_executor()
                    .spawn(async move {
                        let adb = find_adb_binary();
                        let out = Command::new(adb)
                            .args(["-s", &serial_now, "exec-out", "screencap", "-p"])
                            .output()
                            .ok()?;
                        if !out.status.success() || out.stdout.len() < 64 {
                            return None;
                        }
                        let mut rgba = image::load_from_memory(&out.stdout).ok()?.into_rgba8();
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
                    })
                    .await;

                if let Some((w, h, render_img)) = frame {
                    let _ = this.update(cx, |view, cx| {
                        view.device_width = w.max(1);
                        view.device_height = h.max(1);
                        view.frame_image = Some(render_img);
                        cx.notify();
                    });
                }
                cx.background_executor()
                    .timer(Duration::from_millis(220))
                    .await;
            }
        }));
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
        if !self.send_control_bytes(&pkt) {
            let escaped = text
                .replace(' ', "%s")
                .replace("\"", "")
                .replace('\u{27}', "");
            self.run_adb_async(
                vec!["shell".into(), "input".into(), "text".into(), escaped],
                cx,
            );
        }
        self.status_notice = "Synced Mac clipboard text to Android device.".to_string();
        cx.notify();
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
                self.drag_start = Some((x, y, Instant::now()));
            }
            _ => {}
        }
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
        }
    }

    fn on_key_down(&mut self, event: &KeyDownEvent, cx: &mut Context<Self>) {
        if event.keystroke.modifiers.platform && event.keystroke.key.as_str() == "v" {
            self.sync_mac_clipboard_to_phone(cx);
            return;
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
                                    .child("Pro Android Mirror & Remote Control Hub"),
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
                                    "android-refresh-devices",
                                    ui::ButtonVariant::Gray,
                                    ui::ButtonSize::Xs,
                                )
                                .label("Refresh ADB")
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
                            .child(format!("📱 {} ({})", dev.model, dev.transport))
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
                    ),
            )
            .child(
                div()
                    .id("android-viewport-canvas")
                    .flex_1()
                    .w_full()
                    .rounded(px(14.))
                    .border_2()
                    .border_color(Hsla::from(theme.gray_5))
                    .bg(gpui::black())
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
                    .child(if let Some(frame) = &self.frame_image {
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
                                    .child("Android 60 FPS Interactive Viewport"),
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
                            .label("📋 Paste Mac Clipboard")
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
    fn adb_devices_output_parses_usb_and_wifi_transports() {
        let sample = "List of devices attached

                      R5CT90ABCDE          device usb:1-1 product:dm3q model:SM_S918B device:dm3q

                      192.168.1.44:5555    device product:husky model:Pixel_8_Pro device:husky
";
        let list = parse_adb_devices_output(sample);
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].model, "SM S918B");
        assert_eq!(list[0].transport, "USB");
        assert_eq!(list[1].model, "Pixel 8 Pro");
        assert_eq!(list[1].transport, "Wi-Fi");
    }
}
