#[macro_use]
extern crate gdnative as godot;

#[derive(gdnative::NativeClass)]
#[inherit(gdnative::Node)]
struct Client {}

#[gdnative::methods]
impl Client {

    fn _init(_owner: gdnative::Node) -> Self {
        Client {}
    }

    #[export]
    fn _ready(&self, _owner: gdnative::Node) {
        godot_print!("hello, world.")
    }

    #[export]
    fn send(&self, _owner: gdnative::Node, message: godot::GodotString) {
        godot_print!("send packet: {}", message.to_string())
    }
}

fn init(handle: gdnative::init::InitHandle) {
    handle.add_class::<Client>();
}

godot_gdnative_init!();
godot_nativescript_init!(init);
godot_gdnative_terminate!();