use derive_display::derive_display;

pub struct Thing;

#[derive_display]
impl Thing {
    pub fn method(&self) {}
}

fn main() {}
