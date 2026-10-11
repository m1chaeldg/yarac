use gpui::*;
use crate::theme::Theme;

pub struct EnvModal {
    pub is_open: bool,
    pub theme: Theme,
}

impl EnvModal {
    pub fn new() -> Self {
        Self {
            is_open: false,
            theme: Theme::apple_default(),
        }
    }
}

impl Render for EnvModal {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .p_4()
            .rounded(px(self.theme.radii.card_lg))
            .bg(rgb(0xffffff))
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child("Manage Environments")
            )
    }
}
