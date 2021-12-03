use std::collections::HashMap;

#[derive(Debug)]
pub struct Route {
    pub name: String,
    pub parameters: HashMap<String, String>,
}
