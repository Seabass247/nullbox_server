use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum DataType {
    Coords { x: f32, y: f32, z: f32 },
    ASCII { string: String },
}
