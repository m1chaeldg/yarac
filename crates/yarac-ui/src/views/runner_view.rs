use gpui::*;
use crate::theme::Theme;
use crate::state::app_controller::AppController;

pub struct RunnerView {
    pub theme: Theme,
    pub controller: Entity<AppController>,
}

impl RunnerView {
    pub fn new(controller: Entity<AppController>) -> Self {
        Self {
            theme: Theme::apple_default(),
            controller,
        }
    }
}

impl Render for RunnerView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let app = self.controller.read(cx);
        let stats = app.runner.compute_stats();

        div()
            .flex()
            .flex_col()
            .flex_1()
            .h_full()
            .bg(rgb(0xffffff))
            .p_6()
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .mb_4()
                    .child(
                        div()
                            .text_xl()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(0x1d1d1f))
                            .child("Collection Runner")
                    )
                    .child(
                        div()
                            .px_4()
                            .py_1()
                            .rounded(px(self.theme.radii.pill))
                            .bg(rgb(0x0066cc))
                            .text_color(rgb(0xffffff))
                            .text_sm()
                            .font_weight(FontWeight::MEDIUM)
                            .child("Run Collection")
                    )
            )
            .child(
                // Metrics Cards
                div()
                    .flex()
                    .gap_3()
                    .mb_6()
                    .child(
                        div()
                            .p_3()
                            .rounded(px(self.theme.radii.btn_sm))
                            .bg(rgb(0xf5f5f7))
                            .child(div().text_xs().text_color(rgb(0x7a7a7a)).child("Total Runs"))
                            .child(div().text_lg().font_weight(FontWeight::BOLD).child(format!("{}", stats.count)))
                    )
                    .child(
                        div()
                            .p_3()
                            .rounded(px(self.theme.radii.btn_sm))
                            .bg(rgb(0xf5f5f7))
                            .child(div().text_xs().text_color(rgb(0x7a7a7a)).child("Passed"))
                            .child(div().text_lg().font_weight(FontWeight::BOLD).text_color(rgb(0x107c41)).child(format!("{}", stats.success_count)))
                    )
                    .child(
                        div()
                            .p_3()
                            .rounded(px(self.theme.radii.btn_sm))
                            .bg(rgb(0xf5f5f7))
                            .child(div().text_xs().text_color(rgb(0x7a7a7a)).child("Failed"))
                            .child(div().text_lg().font_weight(FontWeight::BOLD).text_color(rgb(0xd83b01)).child(format!("{}", stats.failure_count)))
                    )
                    .child(
                        div()
                            .p_3()
                            .rounded(px(self.theme.radii.btn_sm))
                            .bg(rgb(0xf5f5f7))
                            .child(div().text_xs().text_color(rgb(0x7a7a7a)).child("p50 Latency"))
                            .child(div().text_lg().font_weight(FontWeight::BOLD).child(format!("{} ms", stats.p50_ms)))
                    )
                    .child(
                        div()
                            .p_3()
                            .rounded(px(self.theme.radii.btn_sm))
                            .bg(rgb(0xf5f5f7))
                            .child(div().text_xs().text_color(rgb(0x7a7a7a)).child("p99 Latency"))
                            .child(div().text_lg().font_weight(FontWeight::BOLD).child(format!("{} ms", stats.p99_ms)))
                    )
            )
    }
}
