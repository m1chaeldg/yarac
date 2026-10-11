use gpui::*;
use crate::theme::Theme;
use crate::state::app_controller::AppController;
use super::sidebar::Sidebar;
use super::editor_pane::EditorPane;
use super::response_pane::ResponsePane;
use super::runner_view::RunnerView;
use super::modals::env_modal::EnvModal;
use super::modals::interop_modal::InteropModal;
use super::modals::conflict_modal::ConflictModal;

pub enum ActiveTab {
    Studio,
    Runner,
}

pub struct AppWindow {
    pub active_tab: ActiveTab,
    pub theme: Theme,
    pub controller: Entity<AppController>,
    pub sidebar: Entity<Sidebar>,
    pub editor_pane: Entity<EditorPane>,
    pub response_pane: Entity<ResponsePane>,
    pub runner_view: Entity<RunnerView>,
    pub env_modal: Entity<EnvModal>,
    pub interop_modal: Entity<InteropModal>,
    pub conflict_modal: Entity<ConflictModal>,
}

impl AppWindow {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let controller = cx.new(|_| AppController::new());
        Self {
            active_tab: ActiveTab::Studio,
            theme: Theme::apple_default(),
            sidebar: cx.new(|_| Sidebar::new(controller.clone())),
            editor_pane: cx.new(|_| EditorPane::new(controller.clone())),
            response_pane: cx.new(|_| ResponsePane::new(controller.clone())),
            runner_view: cx.new(|_| RunnerView::new(controller.clone())),
            env_modal: cx.new(|_| EnvModal::new()),
            interop_modal: cx.new(|_| InteropModal::new()),
            conflict_modal: cx.new(|_| ConflictModal::new()),
            controller,
        }
    }
}

impl Render for AppWindow {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(0xffffff))
            .child(
                // Global top bar (44px per DESIGN.md global-nav)
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .h(px(44.0))
                    .px_4()
                    .bg(rgb(0x000000)) // Apple Surface Black
                    .text_color(rgb(0xffffff))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_4()
                            .child(
                                div()
                                    .font_weight(FontWeight::BOLD)
                                    .text_sm()
                                    .child("Yarac")
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(0xcccccc))
                                    .child("Studio")
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(0xcccccc))
                                    .child("Runner")
                            )
                    )
            )
            .child(
                // Main Workspace Layout
                div()
                    .flex()
                    .flex_1()
                    .h_full()
                    .child(self.sidebar.clone())
                    .child(self.editor_pane.clone())
                    .child(self.response_pane.clone())
            )
    }
}
