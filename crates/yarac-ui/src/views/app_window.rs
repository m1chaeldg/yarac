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
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let is_studio = matches!(self.active_tab, ActiveTab::Studio);
        let is_runner = matches!(self.active_tab, ActiveTab::Runner);

        let nav_studio = div()
            .id("nav-tab-studio")
            .text_xs()
            .cursor_pointer()
            .text_color(if is_studio { rgb(0xffffff) } else { rgb(0x8e8e93) })
            .font_weight(if is_studio { FontWeight::BOLD } else { FontWeight::NORMAL })
            .child("Studio")
            .on_click(cx.listener(|this, _event, _window, cx| {
                this.active_tab = ActiveTab::Studio;
                cx.notify();
            }));

        let nav_runner = div()
            .id("nav-tab-runner")
            .text_xs()
            .cursor_pointer()
            .text_color(if is_runner { rgb(0xffffff) } else { rgb(0x8e8e93) })
            .font_weight(if is_runner { FontWeight::BOLD } else { FontWeight::NORMAL })
            .child("Runner")
            .on_click(cx.listener(|this, _event, _window, cx| {
                this.active_tab = ActiveTab::Runner;
                cx.notify();
            }));

        let main_content = if is_runner {
            div()
                .flex()
                .flex_1()
                .h_full()
                .child(self.runner_view.clone())
        } else {
            div()
                .flex()
                .flex_1()
                .h_full()
                .child(self.sidebar.clone())
                .child(self.editor_pane.clone())
                .child(self.response_pane.clone())
        };

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
                            .child(nav_studio)
                            .child(nav_runner)
                    )
            )
            .child(main_content)
    }
}
