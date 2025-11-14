#![allow(dead_code)]
pub const RATIO: f32 = 1.125;
pub const H1: f32 = H2 * RATIO;
pub const H2: f32 = H3 * RATIO;
pub const H3: f32 = H4 * RATIO;
pub const H4: f32 = H5 * RATIO;
pub const H5: f32 = H6  * RATIO;
pub const H6: f32 = P * RATIO;
pub const P: f32 = 16.0;
pub const H7: f32 = P / RATIO;
pub const H8: f32 = H7 / RATIO;
