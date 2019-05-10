#[derive(Debug, Clone)]
pub struct Token {
    pub left_id : u16,
    pub right_id: u16,
    pub cost    : i16,
}

impl PartialEq for Token {
    fn eq(&self, other: &Token) -> bool {
        self.left_id == other.left_id && 
        self.right_id == other.right_id &&
        self.cost == other.cost
    }
}

impl Eq for Token {}
