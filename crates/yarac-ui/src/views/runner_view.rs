use gpui::*;
use crate::theme::Theme;

pub struct RunnerView {
    pub theme: Theme,
}

impl RunnerView {
    pub fn new() -> Self {
        Self {
            theme: Theme::apple_default(),
        }
    }
}

impl Render for RunnerView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .flex_1()
            .h_full()
            .bg(rgb(0xffffff))
            .p_6()
            .child(
                div()
                    .text_xl()
                    .font_weight(FontWeight::BOLD)
                    .mb_4()
                    .child("Collection Runner")
            )
    }
}
