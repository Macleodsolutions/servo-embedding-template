use std::cell::Cell;
use std::path::PathBuf;
use servo_default_resources as _;
use std::rc::Rc;

use dpi::PhysicalSize;
use servo::protocol_handler::{DirectoryProtocolHandler, ProtocolRegistry};
use servo::{ServoBuilder, SoftwareRenderingContext, WebView, WebViewBuilder, WebViewDelegate};
use url::Url;

struct App {
    needs_repaint: Cell<bool>,
}

impl WebViewDelegate for App {
    fn notify_new_frame_ready(&self, _webview: WebView) {
        self.needs_repaint.set(true);
    }

    fn handle_game_engine_spawn_enemy(&self, webview: WebView, enemy_id: String, x: f32, y: f32) {
        webview.fire_gameengine_enemydied(enemy_id, x, y);
    }
}

fn main() {
    let mut protocol_registry = ProtocolRegistry::default();
    let _ = protocol_registry.register(
        "app",
        DirectoryProtocolHandler::new(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui")),
    );

    let servo = ServoBuilder::default()
        .protocol_registry(protocol_registry)
        .build_with_logging();

    let size = PhysicalSize::new(800u32, 600u32);
    let render_ctx = Rc::new(SoftwareRenderingContext::new(size).unwrap());
    let app = Rc::new(App { needs_repaint: Cell::new(false) });

    let webview = WebViewBuilder::new(&servo, render_ctx.clone())
        .delegate(app.clone())
        .url(Url::parse("app://main/index.html").unwrap())
        .build();

    webview.focus();

    for _ in 0..100 {
        servo.spin_event_loop();
        if app.needs_repaint.get() {
            app.needs_repaint.set(false);
            webview.paint();
        }
    }
}
