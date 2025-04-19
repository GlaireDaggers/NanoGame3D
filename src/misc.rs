#[derive(Default, Clone, Copy)]
pub struct Color32 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8
}

impl Color32 {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Color32 {
        Color32 { r, g, b, a }
    }
}

#[derive(Clone, Copy)]
pub struct Rectangle {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32
}

impl Rectangle {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Rectangle {
        Rectangle { x, y, w, h }
    }
}