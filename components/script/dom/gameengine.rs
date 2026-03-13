use dom_struct::dom_struct;
use js::rust::HandleObject;

use crate::dom::bindings::codegen::Bindings::GameEngineBinding::GameEngineMethods;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::reflector::{reflect_dom_object_with_proto, DomGlobal, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::window::Window;
use crate::script_runtime::CanGc;
use script_bindings::inheritance::Castable;

#[dom_struct]
pub(crate) struct GameEngine {
    reflector_: Reflector,
}

impl GameEngine {
    fn new_inherited() -> GameEngine {
        GameEngine {
            reflector_: Reflector::new(),
        }
    }

    pub fn new(
        window: &Window,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<GameEngine>> {
        let global = window.upcast::<GlobalScope>();
        Ok(reflect_dom_object_with_proto(
            Box::new(GameEngine::new_inherited()),
            global,
            proto,
            can_gc,
        ))
    }
}

impl GameEngineMethods<crate::DomTypeHolder> for GameEngine {
    fn Constructor(
        window: &Window,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<GameEngine>> {
        GameEngine::new(window, proto, can_gc)
    }

    fn SpawnEnemy(&self, enemy_id: DOMString, x: f32, y: f32) -> bool {
        let id_str = String::from(enemy_id);
        println!("NATIVE RUST: Spawning {} at {}, {}", id_str, x, y);

        let global = self.global();
        let webview_id = global.downcast::<Window>().map(|w: &Window| w.webview_id());
        global.send_to_embedder(embedder_traits::EmbedderMsg::GameEngineSpawnEnemy(
            webview_id,
            id_str,
            x,
            y,
        ));

        true
    }
}
