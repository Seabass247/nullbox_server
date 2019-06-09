use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum DataType {
    Coords { x: f32, y: f32, z: f32 },
    ASCII { string: String },
    Transform {
        xx: f32,
        xy: f32,
        xz: f32,
        yx: f32,
        yy: f32,
        yz: f32,
        zx: f32,
        zy: f32,
        zz: f32,
        orig_x: f32,
        orig_y: f32,
        orig_z: f32,
    },
}
