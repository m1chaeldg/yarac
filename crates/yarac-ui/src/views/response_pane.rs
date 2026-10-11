use gpui::*;
use crate::theme::Theme;
use crate::state::app_controller::AppController;

pub struct ResponsePane {
    pub theme: Theme,
    pub controller: Entity<AppController>,
    pub active_tab: String, // "body", "headers", "assertions", "logs"
}

impl ResponsePane {
    pub fn new(controller: Entity<AppController>) -> Self {
        Self {
            theme: Theme::apple_default(),
            controller,
            active_tab: "body".into(),
        }
    }
}

impl Render for ResponsePane {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (status, duration_ms, size_bytes, body) = {
            let app = self.controller.read(cx);
            (
                app.active_response.status,
                app.active_response.duration_ms,
                app.active_response.size_bytes,
                app.active_response.body.clone(),
            )
        };

        let status_color = if status >= 200 && status < 300 {
            0x107c41 // green
        } else if status >= 400 {
            0xd83b01 // red
        } else {
            0x7a7a7a
        };

        let status_label = if status > 0 {
            format!("{} OK", status)
        } else {
            "No Response".into()
        };

        let tab_btn = |name: &'static str, label: &'static str, is_active: bool, cx: &mut Context<Self>| {
            let tab_id = format!("resp-tab-{}", name);
            div()
                .id(ElementId::from(SharedString::from(tab_id)))
                .px_2()
                .py_1()
                .cursor_pointer()
                .text_xs()
                .font_weight(if is_active { FontWeight::BOLD } else { FontWeight::NORMAL })
                .text_color(if is_active { rgb(0x0066cc) } else { rgb(0x7a7a7a) })
                .border_b_2()
                .border_color(if is_active { rgb(0x0066cc) } else { rgb(0xfafafc) })
                .on_click(cx.listener(move |this, _event, _window, cx| {
                    this.active_tab = name.to_string();
                    cx.notify();
                }))
                .child(label)
        };

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
                    .justify_between()
                    .mb_3()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_3()
                            .child(
                                // Status pill
                                div()
                                    .px_3()
                                    .py_1()
                                    .rounded(px(self.theme.radii.pill))
                                    .bg(rgb(status_color))
                                    .text_color(rgb(0xffffff))
                                    .text_xs()
                                    .font_weight(FontWeight::BOLD)
                                    .child(status_label)
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(0x7a7a7a))
                                    .child(format!("{} ms", duration_ms))
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(0x7a7a7a))
                                    .child(format!("{} B", size_bytes))
                            )
                    )
            )
            .child(
                // Response Tabs
                div()
                    .flex()
                    .gap_3()
                    .border_b_1()
                    .border_color(rgb(0xe0e0e0))
                    .mb_2()
                    .child(tab_btn("body", "Body", self.active_tab == "body", cx))
                    .child(tab_btn("headers", "Headers", self.active_tab == "headers", cx))
                    .child(tab_btn("assertions", "Tests", self.active_tab == "assertions", cx))
                    .child(tab_btn("logs", "Logs", self.active_tab == "logs", cx))
            )
            .child(
                // Body viewport
                div()
                    .flex_1()
                    .p_3()
                    .bg(rgb(0xffffff))
                    .rounded(px(self.theme.radii.btn_sm))
                    .border_1()
                    .border_color(rgb(0xe0e0e0))
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(0x1d1d1f))
                            .child(if body.is_empty() {
                                "Send a request to inspect response body and headers.".to_string()
                            } else {
                                body.clone()
                            })
                    )
            )
    }
}
