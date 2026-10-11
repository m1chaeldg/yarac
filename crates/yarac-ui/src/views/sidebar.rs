use gpui::*;
use crate::theme::Theme;
use crate::state::app_controller::AppController;

pub struct Sidebar {
    pub theme: Theme,
    pub controller: Entity<AppController>,
}

impl Sidebar {
    pub fn new(controller: Entity<AppController>) -> Self {
        Self {
            theme: Theme::apple_default(),
            controller,
        }
    }
}

impl Render for Sidebar {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let app = self.controller.read(cx);
        let active_id = app.workspace.active_request_id.clone();

        let mut collections_ui = div().flex().flex_col().gap_2();

        for col in &app.workspace.collections {
            let mut req_list = div().flex().flex_col().gap_1().pl_3();

            for req in &col.requests {
                let is_active = Some(req.id.clone()) == active_id;
                let _method_color = self.theme.method_color(&req.method);

                let req_item = div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .px_2()
                    .py_1()
                    .rounded(px(self.theme.radii.btn_sm))
                    .bg(if is_active { rgb(0xe0e0e0) } else { rgb(0xf5f5f7) })
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(match req.method.as_str() {
                                "GET" => 0x0066cc,
                                "POST" => 0x107c41,
                                "PUT" => 0xb15c00,
                                "DELETE" => 0xd83b01,
                                _ => 0x7a7a7a,
                            }))
                            .child(req.method.clone())
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(0x1d1d1f))
                            .child(req.name.clone())
                    );

                req_list = req_list.child(req_item);
            }

            collections_ui = collections_ui.child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .text_xs()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(rgb(0x7a7a7a))
                            .child(col.name.clone())
                    )
                    .child(req_list)
            );
        }

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
            .child(collections_ui)
    }
}
