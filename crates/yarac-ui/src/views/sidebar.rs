use gpui::*;
use crate::theme::Theme;

pub struct Sidebar {
    pub theme: Theme,
}

impl Sidebar {
    pub fn new() -> Self {
        Self {
            theme: Theme::apple_default(),
        }
    }
}

impl Render for Sidebar {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .w(px(260.0))
            .h_full()
            .bg(rgb(0xf5f5f7)) // Apple Parchment
            .border_r_1()
            .border_color(rgb(0xe0e0e0))
            .p_3()
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .mb_3()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(0x1d1d1f))
                            .child("COLLECTIONS")
                    )
            )
    }
}
