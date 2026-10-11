use gpui::*;
use crate::theme::Theme;

pub struct ResponsePane {
    pub theme: Theme,
}

impl ResponsePane {
    pub fn new() -> Self {
        Self {
            theme: Theme::apple_default(),
        }
    }
}

impl Render for ResponsePane {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .flex_1()
            .h_full()
            .bg(rgb(0xfafafc)) // Apple Pearl canvas
            .border_l_1()
            .border_color(rgb(0xe0e0e0))
            .p_4()
            .child(
                // Header status row
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .mb_4()
                    .child(
                        // Status pill
                        div()
                            .px_3()
                            .py_1()
                            .rounded(px(self.theme.radii.pill))
                            .bg(rgb(0x107c41)) // 200 OK
                            .text_color(rgb(0xffffff))
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .child("200 OK")
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(0x7a7a7a))
                            .child("42 ms")
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(0x7a7a7a))
                            .child("1.2 KB")
                    )
            )
    }
}
