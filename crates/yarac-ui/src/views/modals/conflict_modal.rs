use gpui::*;
use crate::theme::Theme;

pub struct ConflictModal {
    pub is_open: bool,
    pub file_path: String,
    pub base_content: String,
    pub ours_content: String,
    pub theirs_content: String,
    pub theme: Theme,
}

impl ConflictModal {
    pub fn new() -> Self {
        Self {
            is_open: false,
            file_path: String::new(),
            base_content: String::new(),
            ours_content: String::new(),
            theirs_content: String::new(),
            theme: Theme::apple_default(),
        }
    }
}

impl Render for ConflictModal {
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
                    .child("Resolve 3-Way File Conflict")
            )
    }
}
