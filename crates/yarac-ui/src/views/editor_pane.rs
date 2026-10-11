use gpui::*;
use crate::theme::Theme;

pub struct EditorPane {
    pub theme: Theme,
}

impl EditorPane {
    pub fn new() -> Self {
        Self {
            theme: Theme::apple_default(),
        }
    }
}

impl Render for EditorPane {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .flex_1()
            .h_full()
            .bg(rgb(0xffffff)) // Apple Pure White canvas
            .p_4()
            .child(
                // Method + URL + Send Row
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .mb_4()
                    .child(
                        // Method Badge
                        div()
                            .px_3()
                            .py_1()
                            .rounded(px(self.theme.radii.btn_sm))
                            .bg(rgb(0x0066cc))
                            .text_color(rgb(0xffffff))
                            .text_sm()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child("GET")
                    )
                    .child(
                        // URL Input
                        div()
                            .flex_1()
                            .px_3()
                            .py_1()
                            .rounded(px(self.theme.radii.pill))
                            .bg(rgb(0xf5f5f7))
                            .border_1()
                            .border_color(rgb(0xe0e0e0))
                            .text_sm()
                            .child("https://api.example.com/v1/users")
                    )
                    .child(
                        // Send Pill Button
                        div()
                            .px_5()
                            .py_1()
                            .rounded(px(self.theme.radii.pill))
                            .bg(rgb(0x0066cc)) // Apple Action Blue
                            .text_color(rgb(0xffffff))
                            .text_sm()
                            .font_weight(FontWeight::MEDIUM)
                            .child("Send")
                    )
            )
    }
}
