use gpui::*;
use crate::theme::Theme;
use crate::state::app_controller::AppController;

pub struct EditorPane {
    pub theme: Theme,
    pub controller: Entity<AppController>,
    pub active_tab: String, // "params", "headers", "auth", "body", "scripts"
}

impl EditorPane {
    pub fn new(controller: Entity<AppController>) -> Self {
        Self {
            theme: Theme::apple_default(),
            controller,
            active_tab: "headers".into(),
        }
    }
}

impl Render for EditorPane {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let app = self.controller.read(cx);
        let req = app.workspace.get_active_request().cloned();
        let method = req.as_ref().map(|r| r.method.clone()).unwrap_or_else(|| "GET".into());
        let url = req.as_ref().map(|r| r.url.clone()).unwrap_or_else(|| "".into());

        let tab_btn = |name: &'static str, label: &'static str, is_active: bool, cx: &mut Context<Self>| {
            let tab_id = format!("editor-tab-{}", name);
            div()
                .id(ElementId::from(SharedString::from(tab_id)))
                .px_3()
                .py_1()
                .cursor_pointer()
                .text_sm()
                .font_weight(if is_active { FontWeight::BOLD } else { FontWeight::NORMAL })
                .text_color(if is_active { rgb(0x0066cc) } else { rgb(0x7a7a7a) })
                .border_b_2()
                .border_color(if is_active { rgb(0x0066cc) } else { rgb(0xffffff) })
                .on_click(cx.listener(move |this, _event, _window, cx| {
                    this.active_tab = name.to_string();
                    cx.notify();
                }))
                .child(label)
        };

        let controller = self.controller.clone();
        let req_to_send = req.clone();

        let send_btn = div()
            .id("btn-send-request")
            .px_5()
            .py_1()
            .cursor_pointer()
            .rounded(px(self.theme.radii.pill))
            .bg(rgb(0x0066cc)) // Apple Action Blue
            .text_color(rgb(0xffffff))
            .text_sm()
            .font_weight(FontWeight::MEDIUM)
            .child("Send")
            .on_click(move |_event, _window, cx| {
                if let Some(target_req) = req_to_send.clone() {
                    let ctrl = controller.clone();
                    cx.spawn(move |cx: &mut AsyncApp| {
                        let mut cx = cx.clone();
                        async move {
                            let req_exec: Result<AppController, _> = ctrl.read_with(&cx, |c: &AppController, _cx| c.clone());
                            if let Ok(mut c_val) = req_exec {
                                let result = c_val.execute_request(&target_req).await;
                                let _ = ctrl.update(&mut cx, |c: &mut AppController, cx| {
                                    if let Ok(resp) = result {
                                        c.active_response = resp;
                                    }
                                    cx.notify();
                                });
                            }
                        }
                    }).detach();
                }
            });

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
                            .bg(rgb(match method.as_str() {
                                "GET" => 0x0066cc,
                                "POST" => 0x107c41,
                                "PUT" => 0xb15c00,
                                "DELETE" => 0xd83b01,
                                _ => 0x7a7a7a,
                            }))
                            .text_color(rgb(0xffffff))
                            .text_sm()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(method)
                    )
                    .child(
                        // URL Input Pill
                        div()
                            .flex_1()
                            .px_3()
                            .py_1()
                            .rounded(px(self.theme.radii.pill))
                            .bg(rgb(0xf5f5f7))
                            .border_1()
                            .border_color(rgb(0xe0e0e0))
                            .text_sm()
                            .child(url)
                    )
                    .child(send_btn)
            )
            .child(
                // Tabs header
                div()
                    .flex()
                    .gap_3()
                    .border_b_1()
                    .border_color(rgb(0xe0e0e0))
                    .mb_3()
                    .child(tab_btn("params", "Params", self.active_tab == "params", cx))
                    .child(tab_btn("headers", "Headers", self.active_tab == "headers", cx))
                    .child(tab_btn("auth", "Auth", self.active_tab == "auth", cx))
                    .child(tab_btn("body", "Body", self.active_tab == "body", cx))
                    .child(tab_btn("scripts", "Scripts", self.active_tab == "scripts", cx))
            )
            .child(
                // Tab Content Area
                div()
                    .flex_1()
                    .p_2()
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(0x7a7a7a))
                            .child(format!("Tab [{}]: Configured key-values and raw body synchronized with .yarac format.", self.active_tab))
                    )
            )
    }
}
