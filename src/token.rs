#[derive(Debug, Clone, Copy)]
pub struct Token {
    pub left_id : u16,
    pub right_id: u16,
    pub cost    : i16,
}

impl Token {
    /**
     * EOS/BOSトークンを生成する。
     * とりあえず決め打ちで値を入れておく。
     */
    pub fn eos_bos() -> Self {
        Token {
            left_id : 0,
            right_id: 0,
            cost    : 0,
        }
    }

    /**
     * UNKNOWNトークンを生成する。
     * とりあえず決め打ちで値を入れておく。
     */
    pub fn unknown() -> Self {
        Token {
            left_id : u16::max_value(),
            right_id: u16::max_value(),
            cost    : i16::max_value(),
        }
    }
}

impl PartialEq for Token {
    fn eq(&self, other: &Token) -> bool {
        self.left_id == other.left_id && 
        self.right_id == other.right_id &&
        self.cost == other.cost
    }
}

impl Eq for Token {}
