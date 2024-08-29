use std::collections::HashMap;

#[derive(Default)]
pub struct Lapis {
    pub buffer: String,
    pub input: String,
    pub settings: bool,
    pub fmap: HashMap<String, f32>,
    pub vmap: HashMap<String, Vec<f32>>,
}
