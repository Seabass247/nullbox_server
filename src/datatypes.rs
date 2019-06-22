use godot::ToVariant;
use serde_derive::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::convert::From;
use bincode::{deserialize, serialize};
use std::io::Error;

#[derive(Clone)]
#[derive(Serialize, Deserialize)]
pub enum VariantType {
    Vector2 { x: f32, y: f32 },
    Vector3 { x: f32, y: f32, z: f32 },
    GDString { string: String },
    Int { int: i64 },
    FloatArray { vec: Vec<f32> },
    GodotBool { boolean: bool },
    StringArr { vec: Vec<String> },
    Unknown {},
}

#[derive(Clone)]
#[derive(Serialize, Deserialize)]
pub enum MetaMessage {
    Heartbeat,
    Ack,
}

pub enum DeliveryType {
    RelOrd,
    RelSeq,
    RelUnord,
    Unrel,
    UnrelSeq,
}

trait SerializableEvent {
    fn to_packet(&self) -> Option<laminar::Packet>;
}

pub struct SendEvent {
    pub addr: SocketAddr,
    pub delivery: DeliveryType,
    pub pack: Option<EventData>,
    pub meta: Option<MetaMessage>,
}

impl SendEvent {
    pub fn to_packet(&self) -> Option<laminar::Packet> {
        if let Some(packet_data) = &self.pack {
            let delivery = &self.delivery;
            let serialized = match serialize(&packet_data) {
                Ok(data) => data,
                Err(e) => return None,
            };

            let packet = match delivery {
                RelOrd => laminar::Packet::reliable_ordered(self.addr, serialized, None),
                RelSeq => laminar::Packet::reliable_sequenced(self.addr, serialized, None),
                RelUnord => laminar::Packet::reliable_unordered(self.addr, serialized),
                Unrel => laminar::Packet::unreliable(self.addr, serialized),
                UnrelSeq => laminar::Packet::unreliable_sequenced(self.addr, serialized, None),
            };

            Some(packet)

        } else if let Some(meta) = &self.meta {
            let delivery = &self.delivery;
            let serialized = match serialize(&meta) {
                Ok(data) => data,
                Err(e) => return None,
            };

            let packet = match delivery {
                RelOrd => laminar::Packet::reliable_ordered(self.addr, serialized, None),
                RelSeq => laminar::Packet::reliable_sequenced(self.addr, serialized, None),
                RelUnord => laminar::Packet::reliable_unordered(self.addr, serialized),
                Unrel => laminar::Packet::unreliable(self.addr, serialized),
                UnrelSeq => laminar::Packet::unreliable_sequenced(self.addr, serialized, None),
            };

            Some(packet)        

        } else {
            None
        }
    }
}

impl From<godot::Variant> for VariantType {
    fn from(variant: godot::Variant) -> Self {
        match variant.get_type() {
            godot::VariantType::Vector2 => VariantType::Vector2 {
                x: variant.to_vector2().x,
                y: variant.to_vector2().y,
            },
            godot::VariantType::Vector3 => VariantType::Vector3 {
                x: variant.to_vector3().x,
                y: variant.to_vector3().y,
                z: variant.to_vector3().z,
            },
            godot::VariantType::GodotString => VariantType::GDString {
                string: variant.to_string(),
            },
            godot::VariantType::I64 => VariantType::Int {
                int: variant.to_i64(),
            },
            godot::VariantType::Float32Array => {
                let float_arr = variant.to_float32_array();
                let vec: Vec<f32> = (0..float_arr.len()).map(|i| float_arr.get(i)).collect();
                VariantType::FloatArray { vec }
            }
            godot::VariantType::Bool => VariantType::GodotBool {
                boolean: variant.to_bool(),
            },
            godot::VariantType::StringArray => {
                let string_arr: godot::StringArray = variant.to_string_array();
                let vec: Vec<String> = (0..string_arr.len())
                    .map(|i| string_arr.get(i).to_string())
                    .collect();
                VariantType::StringArr { vec }
            }
            _ => VariantType::Unknown {},
        }
    }
}

#[derive(Clone)]
#[derive(Serialize, Deserialize)]
pub struct VariantTypes(pub Vec<VariantType>);

impl From<godot::Variant> for VariantTypes {
    fn from(variant: godot::Variant) -> Self {
        let mut variant_array = godot::VariantArray::from_variant(&variant).unwrap();
        let data_types: Vec<VariantType> = (0..variant_array.len())
            .map(|i| {
                let var = variant_array.get_val(i);
                let data_type = VariantType::from(var);
                data_type
            })
            .collect();
        VariantTypes(data_types)
    }
}

impl VariantType {
    pub fn to_variant(&self) -> godot::Variant {
        match self {
            VariantType::Vector3 { x, y, z } => {
                godot::Variant::from_vector3(&godot::Vector3::new(*x, *y, *z))
            }
            VariantType::Vector2 { x, y } => {
                godot::Variant::from_vector2(&godot::Vector2::new(*x, *y))
            }
            VariantType::GDString { string } => godot::Variant::from_str(string),
            VariantType::Int { int } => godot::Variant::from_i64(*int),
            VariantType::FloatArray { vec } => {
                let mut float_arr = godot::Float32Array::new();
                vec.iter().for_each(|f| float_arr.push(*f));
                godot::Variant::from_float32_array(&float_arr)
            }
            VariantType::GodotBool { boolean } => godot::Variant::from_bool(*boolean),
            VariantType::StringArr { vec } => {
                let mut str_arr = godot::StringArray::new();
                vec.iter()
                    .for_each(|s| str_arr.push(&godot::GodotString::from_str(s)));
                godot::Variant::from_string_array(&str_arr)
            }
            _ => godot::Variant::new(),
        }
    }
}

#[derive(Clone)]
#[derive(Serialize, Deserialize)]
pub struct EventData {
    pub node_path: String,
    pub method: String,
    pub variants: VariantTypes,
}
