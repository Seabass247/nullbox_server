#[macro_use]
extern crate gdnative as godot;

#[derive(godot::NativeClass)]
#[inherit(godot::Node)]
struct Game;

#[godot::methods]
impl Game {

    fn _init(_owner: godot::Node) -> Self {
        Game
    }

    #[export]
    fn _ready(&self, _owner: godot::Node) {
        godot_print!("hello from rust.")
    }
}

fn init(handle: godot::init::InitHandle) {
    handle.add_class::<Game>();
}

godot_gdnative_init!();
godot_nativescript_init!(init);
godot_gdnative_terminate!();