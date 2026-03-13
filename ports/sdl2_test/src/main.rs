use std::rc::Rc;
use std::cell::Cell;

use dpi::PhysicalSize;
use servo::{
    DeviceIntRect, DeviceIntPoint, DeviceIntSize, DevicePoint, RenderingContext, ServoBuilder, SoftwareRenderingContext, WebView,
    WebViewBuilder, WebViewDelegate, InputEvent, MouseButton as ServoMouseButton, MouseButtonAction, MouseButtonEvent,
    MouseMoveEvent, WheelDelta, WheelEvent, WheelMode, KeyboardEvent
};
use url::Url;
use keyboard_types::{Key, KeyState, NamedKey};
use std::pin::Pin;
use std::future::Future;
use servo::protocol_handler::{DoneChannel, FetchContext, ProtocolHandler, ProtocolRegistry, Request, Response, ResponseBody, ResourceFetchTiming};

struct App {
    needs_repaint: Cell<bool>,
}

impl WebViewDelegate for App {
    fn notify_new_frame_ready(&self, _webview: WebView) {
        self.needs_repaint.set(true);
    }
    
    fn notify_cursor_changed(&self, _webview: WebView, cursor: servo::Cursor) {
        println!("Cursor changed: {:?}", cursor);
    }
    


    fn handle_game_engine_spawn_enemy(&self, _webview: WebView, enemy_id: String, x: f32, y: f32) {
        println!("*** RECEIVED IN MAIN EVENT LOOP ***");
        println!("Requesting to spawn {} at coordinates ({}, {})", enemy_id, x, y);
    }
}

use servo::resources::{self, Resource};
use std::fs;
use std::path::PathBuf;

struct ResourceReader;

impl servo::resources::ResourceReaderMethods for ResourceReader {
    fn read(&self, file: Resource) -> Vec<u8> {
        let mut path = std::env::current_dir().unwrap();
        path.push("resources");
        path.push(file.filename());
        fs::read(path).expect("Can't read file")
    }
    
    fn sandbox_access_files_dirs(&self) -> Vec<PathBuf> {
        vec![]
    }
    
    fn sandbox_access_files(&self) -> Vec<PathBuf> {
        vec![]
    }
}

pub struct AppProtocolHandler;

impl ProtocolHandler for AppProtocolHandler {
    fn load(
        &self,
        request: &mut Request,
        _done_chan: &mut DoneChannel,
        _context: &FetchContext,
    ) -> Pin<Box<dyn Future<Output = Response> + Send>> {
        let url = request.current_url();
        
        let path = std::env::current_dir()
            .unwrap()
            .join("ports")
            .join("sdl2_test")
            .join("ui")
            .join(url.path().trim_start_matches('/'));

        let content = std::fs::read(&path).unwrap_or_else(|_| {
            format!("<html><body><h1>404 Not Found: {}</h1></body></html>", path.display()).into_bytes()
        });
        
        let response = Response::new(url.clone(), ResourceFetchTiming::new(request.timing_type()));
        *response.body.lock() = ResponseBody::Done(content);

        Box::pin(std::future::ready(response))
    }
}

pub fn main() {
    resources::set(Box::new(ResourceReader));
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let width = 800;
    let height = 600;

    let window = video_subsystem
        .window("Servo SDL2 Test", width, height)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let texture_creator = canvas.texture_creator();
    let mut ui_texture = texture_creator
        .create_texture_streaming(sdl2::pixels::PixelFormatEnum::ABGR8888, width, height)
        .unwrap();

    let size = PhysicalSize::new(width, height);
    let render_ctx = Rc::new(SoftwareRenderingContext::new(size).unwrap());
    
    let mut protocol_registry = ProtocolRegistry::default();
    let _ = protocol_registry.register("app", AppProtocolHandler);
    
    let servo = ServoBuilder::default().protocol_registry(protocol_registry).build();
    servo.setup_logging();
    let app = Rc::new(App { needs_repaint: Cell::new(false) });
    
    let url = Url::parse("app://main/index.html").unwrap();
    let webview = WebViewBuilder::new(&servo, render_ctx.clone())
        .delegate(app.clone())
        .url(url)
        .build();

    webview.focus();

    video_subsystem.text_input().start();
    let mut event_pump = sdl_context.event_pump().unwrap();

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit { .. } => break 'running,
                sdl2::event::Event::MouseMotion { x, y, .. } => {
                    let point = DevicePoint::new(x as f32, y as f32);
                    webview.notify_input_event(InputEvent::MouseMove(MouseMoveEvent::new(point.into())));
                }
                sdl2::event::Event::MouseButtonDown { mouse_btn, x, y, .. } => {
                    let point = DevicePoint::new(x as f32, y as f32);
                    let button = match mouse_btn {
                        sdl2::mouse::MouseButton::Left => ServoMouseButton::Left,
                        sdl2::mouse::MouseButton::Right => ServoMouseButton::Right,
                        sdl2::mouse::MouseButton::Middle => ServoMouseButton::Middle,
                        sdl2::mouse::MouseButton::X1 => ServoMouseButton::Back,
                        sdl2::mouse::MouseButton::X2 => ServoMouseButton::Forward,
                        _ => ServoMouseButton::Other(0),
                    };
                    webview.notify_input_event(InputEvent::MouseButton(MouseButtonEvent::new(
                        MouseButtonAction::Down,
                        button,
                        point.into(),
                    )));
                }
                sdl2::event::Event::MouseButtonUp { mouse_btn, x, y, .. } => {
                    let point = DevicePoint::new(x as f32, y as f32);
                    let button = match mouse_btn {
                        sdl2::mouse::MouseButton::Left => ServoMouseButton::Left,
                        sdl2::mouse::MouseButton::Right => ServoMouseButton::Right,
                        sdl2::mouse::MouseButton::Middle => ServoMouseButton::Middle,
                        sdl2::mouse::MouseButton::X1 => ServoMouseButton::Back,
                        sdl2::mouse::MouseButton::X2 => ServoMouseButton::Forward,
                        _ => ServoMouseButton::Other(0),
                    };
                    webview.notify_input_event(InputEvent::MouseButton(MouseButtonEvent::new(
                        MouseButtonAction::Up,
                        button,
                        point.into(),
                    )));
                }
                sdl2::event::Event::MouseWheel { x, y, .. } => {
                    let point = DevicePoint::new(0.0, 0.0);
                    let delta = WheelDelta {
                        x: (x as f64) * 38.0,
                        y: (y as f64) * -38.0,
                        z: 0.0,
                        mode: WheelMode::DeltaPixel,
                    };
                    webview.notify_input_event(InputEvent::Wheel(WheelEvent::new(delta, point.into())));
                }
                sdl2::event::Event::TextInput { text, .. } => {
                    let key = Key::Character(text);
                    let event = KeyboardEvent::from_state_and_key(KeyState::Down, key.clone());
                    webview.notify_input_event(InputEvent::Keyboard(event));
                    let event_up = KeyboardEvent::from_state_and_key(KeyState::Up, key);
                    webview.notify_input_event(InputEvent::Keyboard(event_up));
                }
                sdl2::event::Event::KeyDown { keycode: Some(keycode), .. } => {
                    let key = match keycode {
                        sdl2::keyboard::Keycode::Backspace => Some(Key::Named(NamedKey::Backspace)),
                        sdl2::keyboard::Keycode::Delete => Some(Key::Named(NamedKey::Delete)),
                        sdl2::keyboard::Keycode::Return => Some(Key::Named(NamedKey::Enter)),
                        sdl2::keyboard::Keycode::Escape => Some(Key::Named(NamedKey::Escape)),
                        sdl2::keyboard::Keycode::Up => Some(Key::Named(NamedKey::ArrowUp)),
                        sdl2::keyboard::Keycode::Down => Some(Key::Named(NamedKey::ArrowDown)),
                        sdl2::keyboard::Keycode::Left => Some(Key::Named(NamedKey::ArrowLeft)),
                        sdl2::keyboard::Keycode::Right => Some(Key::Named(NamedKey::ArrowRight)),
                        sdl2::keyboard::Keycode::F5 => {
                            webview.reload();
                            None
                        }
                        _ => None,
                    };
                    if let Some(k) = key {
                        let event = KeyboardEvent::from_state_and_key(KeyState::Down, k);
                        webview.notify_input_event(InputEvent::Keyboard(event));
                    }
                }
                sdl2::event::Event::KeyUp { keycode: Some(keycode), .. } => {
                    let key = match keycode {
                        sdl2::keyboard::Keycode::Backspace => Some(Key::Named(NamedKey::Backspace)),
                        sdl2::keyboard::Keycode::Delete => Some(Key::Named(NamedKey::Delete)),
                        sdl2::keyboard::Keycode::Return => Some(Key::Named(NamedKey::Enter)),
                        sdl2::keyboard::Keycode::Escape => Some(Key::Named(NamedKey::Escape)),
                        sdl2::keyboard::Keycode::Up => Some(Key::Named(NamedKey::ArrowUp)),
                        sdl2::keyboard::Keycode::Down => Some(Key::Named(NamedKey::ArrowDown)),
                        sdl2::keyboard::Keycode::Left => Some(Key::Named(NamedKey::ArrowLeft)),
                        sdl2::keyboard::Keycode::Right => Some(Key::Named(NamedKey::ArrowRight)),
                        _ => None,
                    };
                    if let Some(k) = key {
                        let event = KeyboardEvent::from_state_and_key(KeyState::Up, k);
                        webview.notify_input_event(InputEvent::Keyboard(event));
                    }
                }
                _ => {}
            }
        }

        servo.spin_event_loop();

        if app.needs_repaint.get() {
            app.needs_repaint.set(false);
            
            webview.paint();
            
            let rect = DeviceIntRect::from_origin_and_size(
                DeviceIntPoint::new(0, 0),
                DeviceIntSize::new(width as i32, height as i32),
            );
            
            if let Some(image) = render_ctx.read_to_image(rect) {
                ui_texture
                    .update(None, image.as_raw(), (width * 4) as usize)
                    .unwrap();
            }
        }

        canvas.clear();
        canvas.copy(&ui_texture, None, None).unwrap();
        canvas.present();
    }
}
