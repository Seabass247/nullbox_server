use super::super::godot;

#[derive(godot::NativeClass)]
#[inherit(godot::Spatial)]
pub struct PlayerClient;

#[godot::methods]
impl PlayerClient {

    fn _init(_owner: godot::Spatial) -> Self {
        PlayerClient
    }

    #[export]
    fn _ready(&mut self, mut owner: godot::Spatial) {
        godot_print!("Rust PlayerClient ready");

    }

    #[export]
    fn _process(&mut self, _owner: godot::Spatial, delta: f32) {
        
    }

}
