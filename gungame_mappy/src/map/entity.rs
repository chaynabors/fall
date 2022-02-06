use std::collections::HashMap;

use super::brush::Brush;

#[derive(Clone, Debug, Default)]
pub struct Entity {
    pub properties: HashMap<String, String>,
    pub brushes: Vec<Brush>,
}
