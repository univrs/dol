#[derive(Debug, Clone, PartialEq)]
pub struct Container {
    pub id: u64,
    pub name: String,
    pub running: bool,
}

impl Container {
    pub fn new(id: u64, name: String, running: bool) -> Self {
        Self {
            id,
            name,
            running,
        }
    }
}
