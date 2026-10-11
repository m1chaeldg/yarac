use gpui::{App, AppContext, Application, Bounds, px, size};
use yarac_ui::views::app_window::AppWindow;

fn main() {
    println!("Starting Yarac Native GPUI...");
    // Initialize the background Tokio runtime and enter it on the main thread so
    // any Tokio-bound components have an active reactor context.
    let _tokio_guard = yarac_ui::runtime::tokio_runtime().enter();

    Application::new().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(1280.0), px(800.0)), cx);
        let window_options = gpui::WindowOptions {
            window_bounds: Some(gpui::WindowBounds::Windowed(bounds)),
            titlebar: Some(gpui::TitlebarOptions {
                title: Some("Yarac - Fast, Local-First API Client".into()),
                appears_transparent: false,
                traffic_light_position: None,
            }),
            ..Default::default()
        };

        if let Err(e) = cx.open_window(window_options, |_window, cx| {
            cx.new(|cx| AppWindow::new(cx))
        }) {
            eprintln!("Failed to open GPUI window: {:?}", e);
        }
    });
}
