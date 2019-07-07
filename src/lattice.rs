use crate::token::Token;
use crate::dictionary::DictionarySet;

// TODO: NBestを実装する
pub struct Lattice<'a> {
    start   : Vec<Vec<LatticeNode<'a>>>,
    end     : Vec<Vec<(usize, usize)>>,
}

impl<'a> Lattice<'a> {
    pub fn new(size: usize) -> Lattice<'a> {
        // TODO: start, endを使いまわして、malloc の回数を減らしたい
        let mut ret = Lattice {
            start   : Vec::with_capacity(size),
            end     : Vec::with_capacity(size),
        };
        ret.start.resize_with(size, Default::default);
        ret.end.resize_with(size, Default::default);
        ret
    }

    /**
     * ラティス構造に複数のトークンをセットする
     */
    fn set_tokens(&mut self, idx_start: usize, idx_end: usize, surface: &'a [u8], tokens: &[Token]) {
        let idx_start = idx_start + 1;
        let idx_end   = idx_end + 1;
        for &token in tokens {
            self.start[idx_start].push(LatticeNode::new(surface, token));
            self.end[idx_end].push((idx_start, self.start[idx_start].len() - 1));
        }
    }

    /**
     * ラティス構造にトークンをセットする
     */
    fn set_token(&mut self, idx_start: usize, idx_end: usize, surface: &'a [u8], token: Token) {
        let idx_start = idx_start + 1;
        let idx_end   = idx_end + 1;
        self.start[idx_start].push(LatticeNode::new(surface, token));
        self.end[idx_end].push((idx_start, self.start[idx_start].len() - 1));
    }

    /**
     * eos, bosノードをセットする
     */
    fn set_bos_eos(&mut self, surface: &'a [u8], eos_bos: Token) {
        // bosノード
        self.start[0].push(LatticeNode {
            token     : eos_bos,
            surface   : surface,
            total_cost: 0,
            prev      : None,
        });
        self.end[1].push((0, 0));

        // eosノード: startにだけ登録すればいい
        let idx_eos = self.start.len() - 1;
        self.start[idx_eos].push(LatticeNode {
            token     : eos_bos,
            surface   : surface,
            total_cost: i32::max_value(),
            prev      : None,
        });
    }

    /// ラティス構造を構築する
    ///
    /// # Arguments
    ///
    /// * `bytes`    - 解析する文字列(バイト列)
    pub fn build(dict_set: &DictionarySet<'a, Token>, bytes: &'a [u8]) -> Lattice<'a> {
        // ラティス構造の初期化
        let mut lattice = Self::new(bytes.len() + 2);
        // ダブル配列での探索開始index
        let mut idx = 1;
        // idxにおけるbase値
        let mut base = dict_set.base_arr[idx] as usize;

        // BOS・EOSの登録
        lattice.set_bos_eos(&bytes[0..0], Token::eos_bos());
        // 未知語ノードはとりあえずデフォルトで挿入

        for (i, &byte) in bytes.iter().enumerate() { // スタート位置
            // 未知語の終了ノードを求める
            // TODO: graphemeを考慮する(https://doc.rust-lang.org/1.3.0/std/str/struct.GraphemeIndices.html)
            let unknown_end_idx = if (byte & 0b11111000) == 0b11110000 { // 4byte文字
                i + 4
            } else if (byte & 0b11110000) == 0b11100000 { // 3byte文字
                i + 3
            } else if (byte & 0b11100000) == 0b11000000 { // 2byte文字
                i + 2
            } else if (byte & 0b11000000) == 0b10000000 { // スタート位置がutf8の文字として適切でなければスキップ
                continue;
            } else { // if (byte & 0b10000000) == 0b00000000 // 1byte文字
                i + 1
            };
            // println!("{:?}", std::str::from_utf8(&bytes[i..unknown_end_idx]));
            // 未知語ノードの登録
            lattice.set_token(i, unknown_end_idx, &bytes[i..unknown_end_idx], Token::unknown());

            // 通常ノードの登録
            for (j, &byte) in bytes[i..].iter().enumerate() { // スタート位置から文字をループ
                // 次のノードに遷移
                let next_idx = base + (byte as usize);
                if dict_set.check_arr[next_idx] as usize != idx {
                    break;
                }
                idx = next_idx;
                base = dict_set.base_arr[idx] as usize;

                // 値があればlatticeにセット
                let value_idx = base + (u8::max_value() as usize);
                if dict_set.check_arr[value_idx] as usize == idx {
                    let start_idx = i;         // 包含
                    let end_idx   = i + j + 1; // 排他
                    let data_idx = (dict_set.base_arr[value_idx] >> 8) as usize;
                    let data_len = (dict_set.base_arr[value_idx] & 0b11111111) as usize;
                    lattice.set_tokens(
                        start_idx,
                        end_idx,
                        &bytes[start_idx..end_idx],
                        &dict_set.data_arr[data_idx..(data_idx + data_len)]
                    );
                }
            }
            // idxとbaseを初期値に戻す
            idx  = 1;
            base = dict_set.base_arr[idx] as usize;
        }
        lattice
    }

    /**
     * ラティス構造を解析する
     * 文字列の前方から解析していく
     */
    pub fn analyze(&mut self, dict_set: &DictionarySet<'a, Token>) {
        for si in 1..self.start.len() {
            for sj in 0..self.start[si].len() {
                // 右側ノードの左文脈IDと生起コスト
                let left_id  = self.start[si][sj].token.left_id as usize;
                let cost     = self.start[si][sj].token.cost as i32;
                // 直前につながるノードを求める
                for &(ei, ej) in self.end[si].iter() {
                    // 左側ノードの右文脈IDとトータルコスト
                    let right_id  = self.start[ei][ej].token.left_id as usize;
                    let prev_cost = self.start[ei][ej].total_cost;
                    // 接続コスト
                    let conn_cost = dict_set.get_matrix(left_id, right_id) as i32;
                    let total_cost = prev_cost + cost  + conn_cost;
                    if total_cost < self.start[si][sj].total_cost {
                        self.start[si][sj].total_cost = total_cost;
                        self.start[si][sj].prev       = Some((ei, ej));
                    }
                }
            }
        }
    }


    /**
     * 解析結果を配列にまとめる。
     * TODO: イテレータにしたい
     */
    pub fn get_result(&self) -> Vec<&LatticeNode<'a>> {
        // eosノードから前方のノードをさかのぼっていく
        let mut result: Vec<&LatticeNode<'a>> = vec![];
        let mut node = &self.start[self.start.len() - 1][0]; // eosノード
        loop {
            result.push(node);
            if let Some((i, j)) = node.prev {
                node = &self.start[i][j];
            } else {
                break;
            }
        }
        result
    }

    pub fn debug(&self) {
        for i in 0..self.start.len() {
            println!("index: {}", i);
            for (j, node) in self.start[i].iter().enumerate() {
                if j == 0 {
                    println!("=== === === start === === ===");
                }
                println!("|    {}", node);
            }
            for (j, &(ei, ej)) in self.end[i].iter().enumerate() {
                if j == 0 {
                    println!("=== === === end === === ===");
                }
                println!("|    {}", self.start[ei][ej]);

            }
        }
    }
}

#[derive(Debug)]
pub struct LatticeNode<'a> {
    pub token     : Token,
    pub surface   : &'a [u8],
    pub total_cost: i32,
    pub prev      : Option<(usize, usize)>,
}

impl<'a> LatticeNode<'a> {
    pub fn new(surface: &'a [u8], token: Token) -> LatticeNode<'a> {
        LatticeNode {
            token     : token,
            surface   : surface,
            total_cost: i32::max_value(),
            prev      : Some((0, 0)),
        }
    }
}

impl<'a> std::fmt::Display for LatticeNode<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "(surface: {:?}, left_id: {}, right_id: {}, cost: {}, total_cost: {}, prev: {:?})",
            std::str::from_utf8(self.surface),
            self.token.left_id,
            self.token.right_id,
            self.token.cost,
            self.total_cost,
            self.prev,
        )
    }

}


