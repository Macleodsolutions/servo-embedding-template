use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use dpi::PhysicalSize;
use servo_default_resources as _;
use servo::{
    JSValue, JavaScriptEvaluationError, LoadStatus, ServoBuilder, SoftwareRenderingContext,
    WebView, WebViewBuilder, WebViewDelegate,
};
use url::Url;

struct ServoTest {
    servo: servo::Servo,
    rendering_context: Rc<SoftwareRenderingContext>,
}

impl ServoTest {
    fn new() -> Self {
        #[derive(Clone)]
        struct Waker(Arc<AtomicBool>);
        impl servo::EventLoopWaker for Waker {
            fn clone_box(&self) -> Box<dyn servo::EventLoopWaker> {
                Box::new(self.clone())
            }
            fn wake(&self) {
                self.0.store(true, Ordering::Relaxed);
            }
        }

        let rendering_context = Rc::new(
            SoftwareRenderingContext::new(PhysicalSize { width: 800, height: 600 }).unwrap(),
        );

        let servo = ServoBuilder::default()
            .event_loop_waker(Box::new(Waker(Arc::new(AtomicBool::new(false)))))
            .build_with_logging();

        Self { servo, rendering_context }
    }

    fn spin(&self, keep_going: impl Fn() -> bool) {
        while keep_going() {
            self.servo.spin_event_loop();
            std::thread::sleep(Duration::from_millis(1));
        }
    }

    fn evaluate_javascript(
        &self,
        webview: WebView,
        script: impl ToString,
    ) -> Result<JSValue, JavaScriptEvaluationError> {
        let wv = webview.clone();
        self.spin(move || wv.load_status() != LoadStatus::Complete);

        let result: Rc<RefCell<Option<Result<JSValue, JavaScriptEvaluationError>>>> =
            Rc::new(RefCell::new(None));
        let cb_result = result.clone();
        webview.evaluate_javascript(script, move |r| *cb_result.borrow_mut() = Some(r));

        let spin_result = result.clone();
        self.spin(move || spin_result.borrow().is_none());

        result.borrow().clone().expect("JS evaluation result must be set")
    }
}

#[derive(Default)]
struct TestDelegate {
    spawn_fired: Cell<bool>,
    received_enemy_id: RefCell<Option<String>>,
    received_coords: Cell<Option<(f32, f32)>>,
    needs_repaint: Cell<bool>,
}

impl WebViewDelegate for TestDelegate {
    fn notify_new_frame_ready(&self, _webview: WebView) {
        self.needs_repaint.set(true);
    }

    fn handle_game_engine_spawn_enemy(
        &self,
        webview: WebView,
        enemy_id: String,
        x: f32,
        y: f32,
    ) {
        self.spawn_fired.set(true);
        *self.received_enemy_id.borrow_mut() = Some(enemy_id.clone());
        self.received_coords.set(Some((x, y)));
        webview.fire_gameengine_enemydied(enemy_id, x, y);
    }
}

fn test_js_to_rust_spawn_enemy_fires_delegate(harness: &ServoTest) {
    let delegate = Rc::new(TestDelegate::default());

    let html = r#"data:text/html,<script>
        window.addEventListener('load', () => {
            window.gameEngine.spawnEnemy('goblin', 10.5, 20.0);
        });
    </script>"#;

    let webview = WebViewBuilder::new(&harness.servo, harness.rendering_context.clone())
        .delegate(delegate.clone())
        .url(Url::parse(html).unwrap())
        .build();
    webview.focus();

    let d = delegate.clone();
    harness.spin(move || !d.spawn_fired.get());

    assert!(delegate.spawn_fired.get(), "delegate was never called");
    assert_eq!(
        delegate.received_enemy_id.borrow().as_deref(),
        Some("goblin"),
        "wrong enemy_id"
    );
    let (x, y) = delegate.received_coords.get().expect("coords not set");
    assert!((x - 10.5).abs() < f32::EPSILON, "wrong x: {x}");
    assert!((y - 20.0).abs() < f32::EPSILON, "wrong y: {y}");
}

fn test_rust_to_js_enemydied_event_received(harness: &ServoTest) {
    let delegate = Rc::new(TestDelegate::default());

    let html = r#"data:text/html,<script>
        window._result = null;
        window.gameEngine.addEventListener('enemydied', function(e) {
            window._result = { id: e.enemyId, x: e.x, y: e.y };
        });
        window.addEventListener('load', () => {
            window.gameEngine.spawnEnemy('orc', 3.0, 7.5);
        });
    </script>"#;

    let webview = WebViewBuilder::new(&harness.servo, harness.rendering_context.clone())
        .delegate(delegate.clone())
        .url(Url::parse(html).unwrap())
        .build();
    webview.focus();

    let d = delegate.clone();
    harness.spin(move || !d.spawn_fired.get());

    for _ in 0..50 {
        harness.servo.spin_event_loop();
        std::thread::sleep(Duration::from_millis(1));
    }

    let result = harness.evaluate_javascript(webview, "JSON.stringify(window._result)");
    match result {
        Ok(JSValue::String(json)) => {
            assert!(json.contains("\"id\":\"orc\""), "wrong id in event: {json}");
            assert!(json.contains("\"x\":3"), "wrong x in event: {json}");
            assert!(json.contains("\"y\":7.5"), "wrong y in event: {json}");
        },
        other => panic!("unexpected JS result: {other:?}"),
    }
}

fn test_spawn_enemy_returns_true(harness: &ServoTest) {
    let delegate = Rc::new(TestDelegate::default());

    let html = "data:text/html,<script></script>";
    let webview = WebViewBuilder::new(&harness.servo, harness.rendering_context.clone())
        .delegate(delegate.clone())
        .url(Url::parse(html).unwrap())
        .build();
    webview.focus();

    let result = harness.evaluate_javascript(
        webview,
        "window.gameEngine.spawnEnemy('test', 0.0, 0.0)",
    );
    assert_eq!(result, Ok(JSValue::Boolean(true)), "spawnEnemy should return true");
}

#[test]
fn test_bridge_roundtrip_suite() {
    let harness = ServoTest::new();
    test_js_to_rust_spawn_enemy_fires_delegate(&harness);
    test_rust_to_js_enemydied_event_received(&harness);
    test_spawn_enemy_returns_true(&harness);
}
