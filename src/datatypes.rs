use godot::ToVariant;
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
enum DataType {
    Vector2 { x: f32, y: f32 },
    Vector3 { x: f32, y: f32, z: f32 },
    GDString { string: String },
    Int { int: i64 },
    FloatArray { vec: Vec<f32> },
    GodotBool { boolean: bool },
    StringArr { vec: Vec<String> },
    VariantArr { vec: Vec<DataType> },
    Unknown {},
}

impl From<godot::Variant> for DataType {
    fn from(variant: godot::Variant) -> Self {
        match variant.get_type() {
            godot::VariantType::Vector2 => DataType::Vector2 {
                x: variant.to_vector2().x,
                y: variant.to_vector2().y,
            },
            godot::VariantType::Vector3 => DataType::Vector3 {
                x: variant.to_vector3().x,
                y: variant.to_vector3().y,
                z: variant.to_vector3().z,
            },
            godot::VariantType::GodotString => DataType::GDString {
                string: variant.to_string(),
            },
            godot::VariantType::I64 => DataType::Int {
                int: variant.to_i64(),
            },
            godot::VariantType::Float32Array => {
                let float_arr = variant.to_float32_array();
                let vec: Vec<f32> = (0..float_arr.len()).map(|i| float_arr.get(i)).collect();
                DataType::FloatArray { vec }
            },
            godot::VariantType::Bool => DataType::GodotBool {
                boolean: variant.to_bool(),
            },
            godot::VariantType::StringArray => {
                let string_arr: godot::StringArray = variant.to_string_array();
                let vec: Vec<String> = (0..string_arr.len())
                    .map(|i| string_arr.get(i).to_string())
                    .collect();
                DataType::StringArr { vec }
            }
            godot::VariantType::VariantArray => {
                let mut var_array = godot::VariantArray::from_variant(&variant).unwrap();
                let vec_variant: Vec<DataType> = (0..var_array.len())
                    .map(|i| DataType::from(var_array.get_val(i)))
                    .collect();
                DataType::VariantArr { vec: vec_variant }
            }
            _ => DataType::Unknown {},
        }
    }
}

impl DataType {
    fn to_variant(&self) -> godot::Variant {
        match self {
            DataType::Vector3 {x,y,z} => {
                godot::Variant::from_vector3(&godot::Vector3::new(*x,*y,*z))
            },
            DataType::Vector2 {x, y} => {
                godot::Variant::from_vector2(&godot::Vector2::new(*x,*y))
            }
            DataType::GDString { string } => {
                godot::Variant::from_str(string)
            }
            DataType::Int { int } => {
                godot::Variant::from_i64(*int)
            }
            DataType::FloatArray { vec } => {
                let mut float_arr = godot::Float32Array::new();
                vec.iter().for_each(|f| float_arr.push(*f));
                godot::Variant::from_float32_array(&float_arr)
            }
            DataType::GodotBool { boolean } => {
                godot::Variant::from_bool(*boolean)
            }
            DataType::StringArr { vec } => {
                let mut str_arr = godot::StringArray::new();
                vec.iter().for_each(|s| str_arr.push(&godot::GodotString::from_str(s)));
                godot::Variant::from_string_array(&str_arr)
            },
            DataType::VariantArr { vec } => {
                let mut var_array = godot::VariantArray::new();
                vec.iter().for_each(|var| {
                    var_array.push(&var.to_variant())
                });
                godot::Variant::from_array(&var_array)
            }
            _ => {
                godot::Variant::new()
            }
        }
    }
}