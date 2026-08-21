use derive_display::derive_display;

pub struct Thing;

pub trait Unrelated {
    fn method(&self);
}

#[derive_display]
impl Unrelated for Thing {
    fn method(&self) {}
}

fn main() {}
