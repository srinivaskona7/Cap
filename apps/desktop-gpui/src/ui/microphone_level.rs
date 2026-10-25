use gpui::{
    Context, Entity, Hsla, IntoElement, ParentElement as _, Render, Styled as _, Window, div,
    prelude::FluentBuilder as _, px,
};

use crate::feeds::{self, Feeds};

pub(crate) struct MicrophoneLevel {
    active: bool,
    fill: f32,
    color: Hsla,
}

impl MicrophoneLevel {
    pub(crate) fn new(feeds: &Entity<Feeds>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        cx.observe_in(feeds, window, |this, feeds, window, cx| {
            let visible = if cfg!(any(target_os = "macos", target_os = "windows")) {
                crate::platform::window_is_visible(window)
            } else {
                window.is_window_active()
            };
            if this.active && visible {
                let fill = Self::fill(feeds.read(cx));
                if this.fill != fill {
                    this.fill = fill;
                    cx.notify();
                }
            }
        })
        .detach();
        Self {
            active: false,
            fill: 0.,
            color: gpui::transparent_black(),
        }
    }

    pub(crate) fn fill(feeds: &Feeds) -> f32 {
        if feeds.microphone.is_some() && feeds.mic_level_db.is_finite() {
            (1. - feeds::picker_level(feeds.mic_level_db)) as f32
        } else {
            0.
        }
    }

    pub(crate) fn configure(&mut self, active: bool, color: Hsla, cx: &mut Context<Self>) {
        let fill = if active {
            Self::fill(Feeds::global(cx).read(cx))
        } else {
            0.
        };
        if self.active != active || self.color != color || self.fill != fill {
            self.active = active;
            self.color = color;
            self.fill = fill;
            cx.notify();
        }
    }

    pub(crate) fn snapshot(fill: f32, color: Hsla) -> gpui::Div {
        let mut background = color;
        background.a *= 0.14;
        let clamped = fill.clamp(0., 1.);
        let active_leds = ((clamped * 14.).ceil() as usize).min(14);

        div()
            .size_full()
            .relative()
            .when(clamped > 0., |this| {
                this.child(
                    div()
                        .h_full()
                        .w(gpui::relative(clamped))
                        .rounded(px(10.))
                        .bg(background),
                )
            })
            .child(
                div()
                    .absolute()
                    .bottom(px(3.))
                    .left(px(38.))
                    .right(px(58.))
                    .h(px(3.))
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(2.))
                    .children((0..14).map(|idx| {
                        let lit = idx < active_leds && clamped > 0.01;
                        let led_color: Hsla = if idx >= 12 {
                            gpui::rgb(crate::theme::Theme::ARGON_EMBER_RED).into()
                        } else if idx >= 9 {
                            gpui::rgb(crate::theme::Theme::ARGON_AMBER_LED).into()
                        } else {
                            gpui::rgb(crate::theme::Theme::ARGON_EMERALD_LED).into()
                        };
                        let mut unlit = led_color;
                        unlit.a = 0.18;
                        div().flex_1().h_full().rounded_full().bg(if lit {
                            led_color
                        } else {
                            unlit
                        })
                    })),
            )
    }
}

impl Render for MicrophoneLevel {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        Self::snapshot(self.fill, self.color)
    }
}
