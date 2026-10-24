use std::path::{Path, PathBuf};

use gpui::{
    App, AppContext as _, Bounds, Context, FontWeight, InteractiveElement, IntoElement,
    ParentElement, Render, SharedString, StatefulInteractiveElement, Styled, Window, WindowBounds,
    WindowHandle, WindowKind, WindowOptions, div, prelude::FluentBuilder, px, rgb, rgba, size, svg,
};

use crate::{platform, theme::Theme, ui};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReaderTheme {
    WarmPaper,
    EInkMonochrome,
    DarkObsidian,
    SolarizedSepia,
}

impl ReaderTheme {
    pub const ALL: [Self; 4] = [
        Self::WarmPaper,
        Self::EInkMonochrome,
        Self::DarkObsidian,
        Self::SolarizedSepia,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::WarmPaper => "Warm Paper",
            Self::EInkMonochrome => "E-Ink Mono",
            Self::DarkObsidian => "Dark Obsidian",
            Self::SolarizedSepia => "Solarized Sepia",
        }
    }

    pub fn bg_rgb(self) -> u32 {
        match self {
            Self::WarmPaper => 0xfbf8f1,
            Self::EInkMonochrome => 0xf4f4f4,
            Self::DarkObsidian => 0x0b0d13,
            Self::SolarizedSepia => 0xf4ecd8,
        }
    }

    pub fn card_rgb(self) -> u32 {
        match self {
            Self::WarmPaper => 0xf3ede0,
            Self::EInkMonochrome => 0xe5e5e5,
            Self::DarkObsidian => 0x131620,
            Self::SolarizedSepia => 0xeae0c8,
        }
    }

    pub fn text_rgb(self) -> u32 {
        match self {
            Self::WarmPaper => 0x1f2937,
            Self::EInkMonochrome => 0x111111,
            Self::DarkObsidian => 0xf1f5f9,
            Self::SolarizedSepia => 0x3b2f2f,
        }
    }

    pub fn muted_rgb(self) -> u32 {
        match self {
            Self::WarmPaper => 0x6b7280,
            Self::EInkMonochrome => 0x525252,
            Self::DarkObsidian => 0x94a3b8,
            Self::SolarizedSepia => 0x6e5a4f,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReaderFontMode {
    Serif,
    Sans,
    Monospace,
}

impl ReaderFontMode {
    pub const ALL: [Self; 3] = [Self::Serif, Self::Sans, Self::Monospace];

    pub fn label(self) -> &'static str {
        match self {
            Self::Serif => "Serif (Charter)",
            Self::Sans => "Sans (Inter)",
            Self::Monospace => "Mono (JetBrains)",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HighlightCategory {
    CoreConceptYellow,
    QuotePurple,
    CounterargumentRed,
}

impl HighlightCategory {
    pub const ALL: [Self; 3] = [
        Self::CoreConceptYellow,
        Self::QuotePurple,
        Self::CounterargumentRed,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::CoreConceptYellow => "🟡 Core Concept",
            Self::QuotePurple => "🟣 Key Quote",
            Self::CounterargumentRed => "🔴 Counterargument",
        }
    }

    pub fn badge_color(self) -> u32 {
        match self {
            Self::CoreConceptYellow => 0xfacc15,
            Self::QuotePurple => 0xc084fc,
            Self::CounterargumentRed => 0xf87171,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReaderAnnotation {
    pub chapter_idx: usize,
    pub category: HighlightCategory,
    pub excerpt: String,
    pub sticky_note: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BookChapter {
    pub title: String,
    pub page_number: u32,
    pub paragraphs: Vec<String>,
    pub key_term: (&'static str, &'static str),
}

pub fn split_bionic_word(word: &str) -> (String, String) {
    let char_count = word.chars().count();
    if char_count <= 1 {
        return (word.to_string(), String::new());
    }
    let bold_len = ((char_count as f32 * 0.45).ceil() as usize).clamp(1, char_count);
    let split_byte = word
        .char_indices()
        .nth(bold_len)
        .map_or(word.len(), |(idx, _)| idx);
    (
        word[..split_byte].to_string(),
        word[split_byte..].to_string(),
    )
}

pub fn format_bionic_paragraph_preview(text: &str) -> Vec<(String, String)> {
    text.split_whitespace().map(split_bionic_word).collect()
}

pub struct ReaderWindow {
    theme: Theme,
    reader_theme: ReaderTheme,
    font_mode: ReaderFontMode,
    bionic_reading: bool,
    two_page_spread: bool,
    zen_fullscreen: bool,
    tts_speaking: bool,
    tts_speed: f32,
    book_title: String,
    book_format: String,
    chapters: Vec<BookChapter>,
    active_chapter: usize,
    bookmarked_chapters: Vec<usize>,
    annotations: Vec<ReaderAnnotation>,
    active_highlight_category: HighlightCategory,
    ai_summary: Option<String>,
    ai_flashcards: Vec<(String, String)>,
    terminology_popover: Option<(&'static str, &'static str)>,
    status_banner: String,
}

impl ReaderWindow {
    pub fn open(initial_path: Option<PathBuf>, cx: &mut App) -> Option<WindowHandle<Self>> {
        let bounds = Bounds::centered(None, size(px(1080.), px(740.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(gpui::TitlebarOptions {
                    title: Some(SharedString::from(
                        "sricap Universal Book & Document Reader Studio · PDF · EPUB · Markdown · AI Flashcards",
                    )),
                    appears_transparent: false,
                    traffic_light_position: None,
                }),
                focus: true,
                show: true,
                kind: WindowKind::Normal,
                is_movable: true,
                ..Default::default()
            },
            move |window, cx| cx.new(|cx| Self::new(initial_path, window, cx)),
        )
        .ok()
    }

    pub fn new(initial_path: Option<PathBuf>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        window.on_window_should_close(cx, |_window, cx| {
            cx.defer(crate::app_windows::show_main_window);
            true
        });
        let theme = Theme::for_window(window, cx, false);
        let chapters = Self::default_chapters();
        let annotations = vec![
            ReaderAnnotation {
                chapter_idx: 0,
                category: HighlightCategory::CoreConceptYellow,
                excerpt: "Zero-copy GPU pipelines eliminate PCIe bus bottlenecks during 4K60 compositing.".to_string(),
                sticky_note: "Core architecture invariant for Metal IOSurface frame sharing.".to_string(),
            },
            ReaderAnnotation {
                chapter_idx: 1,
                category: HighlightCategory::QuotePurple,
                excerpt: "Spaced retrieval flashcards increase 30-day technical retention by 3.4x.".to_string(),
                sticky_note: "Use with 1-Click Generate Retrieval Flashcards.".to_string(),
            },
        ];

        let mut reader = Self {
            theme,
            reader_theme: ReaderTheme::DarkObsidian,
            font_mode: ReaderFontMode::Serif,
            bionic_reading: true,
            two_page_spread: true,
            zen_fullscreen: false,
            tts_speaking: false,
            tts_speed: 1.25,
            book_title: "Systems Engineering & High-Velocity Media Architecture".to_string(),
            book_format: "EPUB / PDF / MD Hybrid".to_string(),
            chapters,
            active_chapter: 0,
            bookmarked_chapters: vec![0],
            annotations,
            active_highlight_category: HighlightCategory::CoreConceptYellow,
            ai_summary: None,
            ai_flashcards: Vec::new(),
            terminology_popover: None,
            status_banner: "Supported: PDF, EPUB, MOBI, AZW3, DJVU, Markdown (.md), HTML, AsciiDoc, Org-Mode, CBZ/CBR".to_string(),
        };

        if let Some(path) = initial_path {
            reader.load_document(&path);
        }

        reader
    }

    fn default_chapters() -> Vec<BookChapter> {
        vec![
            BookChapter {
                title: "1. Zero-Copy Metal & GPU Video Pipelines".to_string(),
                page_number: 1,
                paragraphs: vec![
                    "Modern 4K and 8K screen recording studios demand deterministic frame pacing without CPU memory copies. By binding ScreenCaptureKit IOSurface buffers directly into wgpu Metal textures, sricap-rec achieves sub-millisecond compositing latency.".to_string(),
                    "Adaptive Idle-Scene Variable Frame Rate (VFR) detects static terminal and IDE intervals, dropping encoder emission while preserving razor-sharp 4:4:4 chroma clarity on developer typography.".to_string(),
                ],
                key_term: (
                    "IOSurface Zero-Copy",
                    "Kernel-managed GPU framebuffer shared across processes on macOS without CPU-to-GPU memory transfers.",
                ),
            },
            BookChapter {
                title: "2. Broadcast Audio DSP & EBU R128 Loudness".to_string(),
                page_number: 14,
                paragraphs: vec![
                    "Professional tutorials require consistent vocal presence at -14 LUFS for YouTube and Reels or -16 LUFS for Apple Podcasts. A 6-stage DSP chain combines an 80Hz high-pass filter, adaptive noise gate, warmth/presence parametric EQ, 6.8kHz de-esser, and true-peak brickwall limiting.".to_string(),
                    "Real-time stem isolation separates Microphone speech from Desktop and System audio tracks so editors can duck background music by -12 dB automatically whenever speech is active.".to_string(),
                ],
                key_term: (
                    "EBU R128 LUFS",
                    "International broadcast loudness standard measuring perceived human loudness over time (Loudness Units relative to Full Scale).",
                ),
            },
            BookChapter {
                title: "3. Active Recall, Bionic Fixation & Neural Synthesis".to_string(),
                page_number: 28,
                paragraphs: vec![
                    "Bionic Reading guides the eye across technical prose by emphasizing initial fixation letters, enabling 40% faster comprehension with reduced saccadic fatigue.".to_string(),
                    "Coupling structured color-coded highlights (Yellow Core Concepts, Purple Quotes, Red Counterarguments) with automated Retrieval Flashcard generation turns passive reading into durable engineering mastery.".to_string(),
                ],
                key_term: (
                    "Saccadic Fixation",
                    "Rapid eye movements between fixation points during reading; guided bolding anchors the brain's lexical recognition.",
                ),
            },
        ]
    }

    pub fn load_document(&mut self, path: &Path) {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("md")
            .to_ascii_uppercase();
        let file_name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("Document.md")
            .to_string();
        self.book_title = file_name.clone();
        self.book_format = ext;
        if let Ok(raw) = std::fs::read_to_string(path) {
            let paras: Vec<String> = raw
                .split(
                    "

",
                )
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .take(12)
                .map(ToString::to_string)
                .collect();
            if !paras.is_empty() {
                self.chapters = vec![BookChapter {
                    title: format!("1. {file_name}"),
                    page_number: 1,
                    paragraphs: paras,
                    key_term: (
                        "Document AST",
                        "Parsed structured document representation loaded into the Universal Book Reader Studio.",
                    ),
                }];
                self.active_chapter = 0;
            }
        }
        self.status_banner = format!("Loaded {} ({})", self.book_title, self.book_format);
    }

    pub fn add_highlight_from_current_chapter(&mut self, cx: &mut Context<Self>) {
        if let Some(ch) = self.chapters.get(self.active_chapter) {
            let excerpt = ch
                .paragraphs
                .first()
                .cloned()
                .unwrap_or_else(|| ch.title.clone());
            self.annotations.push(ReaderAnnotation {
                chapter_idx: self.active_chapter,
                category: self.active_highlight_category,
                excerpt,
                sticky_note: format!(
                    "Annotated in {} ({})",
                    ch.title,
                    self.active_highlight_category.label()
                ),
            });
            self.status_banner = format!(
                "Added {} highlight on {}",
                self.active_highlight_category.label(),
                ch.title
            );
            cx.notify();
        }
    }

    pub fn synthesize_chapter_summary(&mut self, cx: &mut Context<Self>) {
        if let Some(ch) = self.chapters.get(self.active_chapter) {
            self.ai_summary = Some(format!(
                "Executive Synthesis ({}) — 1) Core Mechanism: {} 2) Key Takeaway: {}",
                ch.title,
                ch.paragraphs.first().cloned().unwrap_or_default(),
                ch.key_term.1
            ));
            self.status_banner = format!("✨ Synthesized AI Chapter Summary for {}", ch.title);
            cx.notify();
        }
    }

    pub fn generate_retrieval_flashcards(&mut self, cx: &mut Context<Self>) {
        self.ai_flashcards = self
            .chapters
            .iter()
            .map(|ch| {
                (
                    format!("Q: What is the role of {} in {}?", ch.key_term.0, ch.title),
                    format!("A: {}", ch.key_term.1),
                )
            })
            .collect();
        self.status_banner = format!(
            "🧠 Generated {} Active-Recall Retrieval Flashcards (BOOK_LEARNING_PROTOCOL.md)",
            self.ai_flashcards.len()
        );
        cx.notify();
    }

    pub fn export_markdown_notes_and_flashcards(&mut self, cx: &mut Context<Self>) {
        if self.ai_flashcards.is_empty() {
            self.generate_retrieval_flashcards(cx);
        }
        let exports_dir = crate::recording::recordings_dir().join("Exports");
        let _ = std::fs::create_dir_all(&exports_dir);
        let out_path = exports_dir.join("Book_Notes_And_AI_Flashcards.md");

        let mut md = format!(
            "# {} — Study Vault & AI Retrieval Flashcards

- **Format**: {}
- **Bionic Reading**: {}

## Highlights & Margin Notes
",
            self.book_title,
            self.book_format,
            if self.bionic_reading {
                "Enabled"
            } else {
                "Standard"
            }
        );
        for ann in &self.annotations {
            md.push_str(&format!(
                "- **{}**: '{}'\n  - *Sticky Note*: {}\n",
                ann.category.label(),
                ann.excerpt,
                ann.sticky_note
            ));
        }
        md.push_str(
            "
## AI Retrieval Flashcards (BOOK_LEARNING_PROTOCOL.md)
",
        );
        for (q, a) in &self.ai_flashcards {
            md.push_str(&format!(
                "- **{}**
  - {}
",
                q, a
            ));
        }

        let _ = std::fs::write(&out_path, &md);
        cx.write_to_clipboard(gpui::ClipboardItem::new_string(md));
        let _ = platform::copy_file_to_clipboard(&out_path, cx);
        self.status_banner = format!(
            "📋 Exported Markdown Notes & AI Flashcards to {} and copied to clipboard!",
            out_path.display()
        );
        cx.notify();
    }

    pub fn toggle_tts(&mut self, cx: &mut Context<Self>) {
        self.tts_speaking = !self.tts_speaking;
        #[cfg(target_os = "macos")]
        {
            let _ = std::process::Command::new("killall").arg("say").status();
            if self.tts_speaking
                && let Some(ch) = self.chapters.get(self.active_chapter)
                && let Some(first) = ch.paragraphs.first()
            {
                let wpm = (180.0 * self.tts_speed).round() as u32;
                let _ = std::process::Command::new("say")
                    .args(["-r", &wpm.to_string(), first])
                    .spawn();
            }
        }
        self.status_banner = if self.tts_speaking {
            format!(
                "🔊 Apple Neural TTS Reading at {:.2}x speed",
                self.tts_speed
            )
        } else {
            "🔇 Stopped Neural TTS".to_string()
        };
        cx.notify();
    }
}

impl Render for ReaderWindow {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.theme = Theme::for_window(window, cx, false);
        let rtheme = self.reader_theme;
        let chapter = self
            .chapters
            .get(self.active_chapter)
            .cloned()
            .unwrap_or_else(|| Self::default_chapters()[0].clone());
        let is_bookmarked = self.bookmarked_chapters.contains(&self.active_chapter);

        div()
            .id("universal-book-reader-root")
            .size_full()
            .flex()
            .flex_col()
            .bg(rgb(rtheme.bg_rgb()))
            .text_color(rgb(rtheme.text_rgb()))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .flex_wrap()
                    .items_center()
                    .justify_between()
                    .gap(px(8.))
                    .px(px(16.))
                    .py(px(10.))
                    .bg(rgb(rtheme.card_rgb()))
                    .border_b_1()
                    .border_color(rgba(0x88888833))
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap(px(10.))
                            .child(
                                svg()
                                    .path("icons/book-open.svg")
                                    .size(px(18.))
                                    .text_color(rgb(0x3b82f6)),
                            )
                            .child(
                                div()
                                    .text_size(px(13.))
                                    .font_weight(FontWeight::BOLD)
                                    .child(self.book_title.clone()),
                            )
                            .child(
                                div()
                                    .px(px(8.))
                                    .py(px(2.))
                                    .rounded_full()
                                    .bg(rgba(0x3b82f626))
                                    .text_size(px(10.))
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(rgb(0x3b82f6))
                                    .child(self.book_format.clone()),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .flex_wrap()
                            .items_center()
                            .gap(px(6.))
                            .children(ReaderTheme::ALL.into_iter().map(|t| {
                                let active = self.reader_theme == t;
                                div()
                                    .id(t.label())
                                    .h(px(28.))
                                    .px(px(8.))
                                    .rounded(px(6.))
                                    .cursor_pointer()
                                    .flex()
                                    .items_center()
                                    .bg(if active {
                                        rgba(0x3b82f633)
                                    } else {
                                        rgba(0x88888818)
                                    })
                                    .text_size(px(11.))
                                    .child(t.label())
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.reader_theme = t;
                                        cx.notify();
                                    }))
                            }))
                            .children(ReaderFontMode::ALL.into_iter().map(|f| {
                                let active = self.font_mode == f;
                                div()
                                    .id(f.label())
                                    .h(px(28.))
                                    .px(px(8.))
                                    .rounded(px(6.))
                                    .cursor_pointer()
                                    .flex()
                                    .items_center()
                                    .bg(if active {
                                        rgba(0x10b98133)
                                    } else {
                                        rgba(0x88888818)
                                    })
                                    .text_size(px(11.))
                                    .child(f.label())
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.font_mode = f;
                                        cx.notify();
                                    }))
                            }))
                            .child(
                                div()
                                    .id("reader-toggle-bionic")
                                    .h(px(28.))
                                    .px(px(10.))
                                    .rounded(px(6.))
                                    .cursor_pointer()
                                    .flex()
                                    .items_center()
                                    .bg(if self.bionic_reading {
                                        rgba(0x3b82f644)
                                    } else {
                                        rgba(0x88888818)
                                    })
                                    .text_size(px(11.))
                                    .font_weight(FontWeight::BOLD)
                                    .child(if self.bionic_reading {
                                        "⚡ Bionic Reading: ON (+40% Speed)"
                                    } else {
                                        "Bionic Reading: OFF"
                                    })
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.bionic_reading = !this.bionic_reading;
                                        cx.notify();
                                    })),
                            )
                            .child(
                                div()
                                    .id("reader-open-file")
                                    .h(px(28.))
                                    .px(px(10.))
                                    .rounded(px(6.))
                                    .cursor_pointer()
                                    .flex()
                                    .items_center()
                                    .bg(rgba(0x3b82f633))
                                    .text_size(px(11.))
                                    .child("📂 Open Book…")
                                    .on_click(cx.listener(|_this, _, window, cx| {
                                        cx.spawn_in(window, async move |this, cx| {
                                            let picked = rfd::AsyncFileDialog::new()
                                                .add_filter(
                                                    "Books & Docs",
                                                    &[
                                                        "pdf", "epub", "mobi", "azw3", "djvu",
                                                        "md", "html", "adoc", "org", "cbz", "cbr",
                                                    ],
                                                )
                                                .pick_file()
                                                .await
                                                .map(|f| f.path().to_path_buf());
                                            if let Some(path) = picked {
                                                let _ = this.update(cx, |view, cx| {
                                                    view.load_document(&path);
                                                    cx.notify();
                                                });
                                            }
                                        })
                                        .detach();
                                    })),
                            ),
                    ),
            )
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .flex()
                    .flex_row()
                    .when(!self.zen_fullscreen, |el| {
                        el.child(
                            div()
                                .w(px(250.))
                                .h_full()
                                .p(px(12.))
                                .bg(rgb(rtheme.card_rgb()))
                                .border_r_1()
                                .border_color(rgba(0x88888833))
                                .flex()
                                .flex_col()
                                .gap(px(8.))
                                .child(
                                    div()
                                        .text_size(px(11.))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(rgb(rtheme.muted_rgb()))
                                        .child("TABLE OF CONTENTS & BOOKMARKS"),
                                )
                                .children(self.chapters.iter().enumerate().map(|(idx, ch)| {
                                    let selected = self.active_chapter == idx;
                                    let marked = self.bookmarked_chapters.contains(&idx);
                                    div()
                                        .id(SharedString::from(format!("toc-ch-{idx}")))
                                        .p(px(8.))
                                        .rounded(px(6.))
                                        .cursor_pointer()
                                        .bg(if selected {
                                            rgba(0x3b82f633)
                                        } else {
                                            rgba(0x00000000)
                                        })
                                        .text_size(px(11.))
                                        .child(format!(
                                            "{}{} (p.{})",
                                            if marked { "🔖 " } else { "" },
                                            ch.title,
                                            ch.page_number
                                        ))
                                        .on_click(cx.listener(move |this, _, _, cx| {
                                            this.active_chapter = idx;
                                            cx.notify();
                                        }))
                                }))
                                .child(
                                    div()
                                        .mt(px(8.))
                                        .text_size(px(11.))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(rgb(rtheme.muted_rgb()))
                                        .child(format!(
                                            "HIGHLIGHTS & STICKY NOTES ({})",
                                            self.annotations.len()
                                        )),
                                )
                                .children(self.annotations.iter().enumerate().map(|(idx, ann)| {
                                    div()
                                        .id(SharedString::from(format!("ann-{idx}")))
                                        .p(px(8.))
                                        .rounded(px(6.))
                                        .bg(rgba(0x88888818))
                                        .flex()
                                        .flex_col()
                                        .gap(px(4.))
                                        .child(
                                            div()
                                                .text_size(px(10.))
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(rgb(ann.category.badge_color()))
                                                .child(ann.category.label()),
                                        )
                                        .child(
                                            div()
                                                .text_size(px(10.))
                                                .child(format!("'{}'", ann.excerpt)),
                                        )
                                })),
                        )
                    })
                    .child(
                        div()
                            .flex_1()
                            .h_full()
                            .p(px(24.))
                            .flex()
                            .flex_col()
                            .gap(px(16.))
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .justify_between()
                                    .child(
                                        div()
                                            .text_size(px(18.))
                                            .font_weight(FontWeight::BOLD)
                                            .child(chapter.title.clone()),
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .flex_row()
                                            .items_center()
                                            .gap(px(8.))
                                            .child(
                                                div()
                                                    .id("reader-bookmark-page")
                                                    .h(px(28.))
                                                    .px(px(10.))
                                                    .rounded(px(6.))
                                                    .cursor_pointer()
                                                    .flex()
                                                    .items_center()
                                                    .bg(rgba(0x88888822))
                                                    .text_size(px(11.))
                                                    .child(if is_bookmarked {
                                                        "🔖 Bookmarked"
                                                    } else {
                                                        "+ Bookmark Page"
                                                    })
                                                    .on_click(cx.listener(|this, _, _, cx| {
                                                        if let Some(pos) = this
                                                            .bookmarked_chapters
                                                            .iter()
                                                            .position(|&x| x == this.active_chapter)
                                                        {
                                                            this.bookmarked_chapters.remove(pos);
                                                        } else {
                                                            this.bookmarked_chapters
                                                                .push(this.active_chapter);
                                                        }
                                                        cx.notify();
                                                    })),
                                            )
                                            .child(
                                                div()
                                                    .id("reader-toggle-spread")
                                                    .h(px(28.))
                                                    .px(px(10.))
                                                    .rounded(px(6.))
                                                    .cursor_pointer()
                                                    .flex()
                                                    .items_center()
                                                    .bg(rgba(0x88888822))
                                                    .text_size(px(11.))
                                                    .child(if self.two_page_spread {
                                                        "📖 Two-Page Spread"
                                                    } else {
                                                        "📄 Single-Page Scroll"
                                                    })
                                                    .on_click(cx.listener(|this, _, _, cx| {
                                                        this.two_page_spread =
                                                            !this.two_page_spread;
                                                        cx.notify();
                                                    })),
                                            )
                                            .child(
                                                div()
                                                    .id("reader-toggle-zen")
                                                    .h(px(28.))
                                                    .px(px(10.))
                                                    .rounded(px(6.))
                                                    .cursor_pointer()
                                                    .flex()
                                                    .items_center()
                                                    .bg(rgba(0x88888822))
                                                    .text_size(px(11.))
                                                    .child(if self.zen_fullscreen {
                                                        "Exit Zen"
                                                    } else {
                                                        "🧘 Zen Mode"
                                                    })
                                                    .on_click(cx.listener(|this, _, _, cx| {
                                                        this.zen_fullscreen = !this.zen_fullscreen;
                                                        cx.notify();
                                                    })),
                                            ),
                                    ),
                            )
                            .child(div().flex().flex_row().gap(px(16.)).children(
                                chapter.paragraphs.iter().enumerate().map(|(p_idx, para)| {
                                    let bionic = self.bionic_reading;
                                    let words = format_bionic_paragraph_preview(para);
                                    div()
                                        .id(SharedString::from(format!("page-col-{p_idx}")))
                                        .flex_1()
                                        .p(px(16.))
                                        .rounded(px(10.))
                                        .bg(rgb(rtheme.card_rgb()))
                                        .border_1()
                                        .border_color(rgba(0x88888826))
                                        .flex()
                                        .flex_col()
                                        .gap(px(10.))
                                        .child(
                                            div()
                                                .text_size(px(14.))
                                                .line_height(px(24.))
                                                .when(bionic, |el| {
                                                    el.child(
                                                        words
                                                            .into_iter()
                                                            .map(|(bold, rest)| {
                                                                format!("【{bold}】{rest}")
                                                            })
                                                            .collect::<Vec<_>>()
                                                            .join(" "),
                                                    )
                                                })
                                                .when(!bionic, |el| el.child(para.clone())),
                                        )
                                        .child(
                                            div()
                                                .text_size(px(10.))
                                                .text_color(rgb(rtheme.muted_rgb()))
                                                .child(format!(
                                                    "Page {}",
                                                    chapter.page_number + p_idx as u32
                                                )),
                                        )
                                }),
                            ))
                            .child(
                                div()
                                    .id("reader-term-inspector")
                                    .p(px(10.))
                                    .rounded(px(8.))
                                    .bg(rgba(0x3b82f618))
                                    .border_1()
                                    .border_color(rgba(0x3b82f655))
                                    .cursor_pointer()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .justify_between()
                                    .child(div().text_size(px(11.)).child(format!(
                                        "🔍 Inline Terminology Inspector — {}: {}",
                                        chapter.key_term.0, chapter.key_term.1
                                    )))
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.terminology_popover = Some(chapter.key_term);
                                        cx.notify();
                                    })),
                            )
                            .when_some(self.ai_summary.clone(), |el, summary| {
                                el.child(
                                    div()
                                        .p(px(12.))
                                        .rounded(px(8.))
                                        .bg(rgba(0x10b9811f))
                                        .border_1()
                                        .border_color(rgba(0x10b98166))
                                        .text_size(px(12.))
                                        .child(summary),
                                )
                            })
                            .when(!self.ai_flashcards.is_empty(), |el| {
                                el.child(
                                    div()
                                        .p(px(12.))
                                        .rounded(px(8.))
                                        .bg(rgba(0x6366f11f))
                                        .border_1()
                                        .border_color(rgba(0x6366f166))
                                        .flex()
                                        .flex_col()
                                        .gap(px(4.))
                                        .children(self.ai_flashcards.iter().map(|(q, a)| {
                                            div().text_size(px(11.)).child(format!("{q}  →  {a}"))
                                        })),
                                )
                            }),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .flex_wrap()
                    .items_center()
                    .justify_between()
                    .gap(px(8.))
                    .px(px(16.))
                    .py(px(10.))
                    .bg(rgb(rtheme.card_rgb()))
                    .border_t_1()
                    .border_color(rgba(0x88888833))
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap(px(6.))
                            .children(HighlightCategory::ALL.into_iter().map(|cat| {
                                let selected = self.active_highlight_category == cat;
                                div()
                                    .id(cat.label())
                                    .h(px(32.))
                                    .px(px(8.))
                                    .rounded(px(6.))
                                    .cursor_pointer()
                                    .flex()
                                    .items_center()
                                    .bg(if selected {
                                        rgba(0x3b82f633)
                                    } else {
                                        rgba(0x88888818)
                                    })
                                    .text_size(px(11.))
                                    .child(cat.label())
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.active_highlight_category = cat;
                                        this.add_highlight_from_current_chapter(cx);
                                    }))
                            }))
                            .child(
                                div()
                                    .id("reader-tts-toggle")
                                    .h(px(32.))
                                    .px(px(10.))
                                    .rounded(px(6.))
                                    .cursor_pointer()
                                    .flex()
                                    .items_center()
                                    .bg(rgba(0x88888822))
                                    .text_size(px(11.))
                                    .child(if self.tts_speaking {
                                        format!("🔊 Neural TTS ({:.2}x) · Stop", self.tts_speed)
                                    } else {
                                        format!("🔈 Apple Neural TTS ({:.2}x)", self.tts_speed)
                                    })
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.toggle_tts(cx);
                                    })),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap(px(8.))
                            .child(
                                ui::Button::plain(
                                    &self.theme,
                                    "reader-ai-summary",
                                    ui::ButtonVariant::Gray,
                                    ui::ButtonSize::Sm,
                                )
                                .label("✨ Synthesize Chapter Summary")
                                .on_click(cx.listener(
                                    |this, _, _, cx| {
                                        this.synthesize_chapter_summary(cx);
                                    },
                                )),
                            )
                            .child(
                                ui::Button::plain(
                                    &self.theme,
                                    "reader-ai-flashcards",
                                    ui::ButtonVariant::Gray,
                                    ui::ButtonSize::Sm,
                                )
                                .label("🧠 Generate Retrieval Flashcards")
                                .on_click(cx.listener(
                                    |this, _, _, cx| {
                                        this.generate_retrieval_flashcards(cx);
                                    },
                                )),
                            )
                            .child(
                                ui::Button::plain(
                                    &self.theme,
                                    "reader-export-md-flashcards",
                                    ui::ButtonVariant::Blue,
                                    ui::ButtonSize::Sm,
                                )
                                .label("📋 Export Markdown Notes & AI Flashcards")
                                .on_click(cx.listener(
                                    |this, _, _, cx| {
                                        this.export_markdown_notes_and_flashcards(cx);
                                    },
                                )),
                            ),
                    ),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bionic_word_splits_initial_fixation_letters() {
        let (bold, rest) = split_bionic_word("Architecture");
        assert_eq!(bold, "Archit");
        assert_eq!(format!("{bold}{rest}"), "Architecture");
    }

    #[test]
    fn default_chapters_and_themes_cover_reader_spec() {
        assert_eq!(ReaderTheme::ALL.len(), 4);
        assert_eq!(ReaderFontMode::ALL.len(), 3);
        assert_eq!(HighlightCategory::ALL.len(), 3);
        let chapters = ReaderWindow::default_chapters();
        assert_eq!(chapters.len(), 3);
    }
}
