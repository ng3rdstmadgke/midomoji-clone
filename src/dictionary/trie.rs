use crate::dictionary::bit_cache::BitCache;

struct Node<T: Clone> {
    key   : u8,
    values: Vec<T>,
    nexts : Vec<Node<T>>,
}

pub struct Trie<T: Clone> {
    root: Node<T>,
}

impl<T: Clone> Trie<T> {
    pub fn new() -> Trie<T> {
        Trie { root: Node { key: 0, values: Vec::new(), nexts: Vec::new() }}
    }

    /// trieにノードを追加する
    /// 一つのkeyにつき256個までの値を登録できる
    /// 超えた場合はpanic
    ///
    /// # Arguments
    ///
    /// * `key`   - 追加するキー
    /// * `value` - キーに対応する値
    pub fn set(&mut self, key: &str, value: T) {
        let mut node = &mut self.root;
        for &k in key.as_bytes() {
           match Self::binary_search(k, &node.nexts) {
               None    => {
                   node.nexts.push(Node { key: k, values: Vec::new(), nexts: Vec::new() });
                   let i = Self::sort(&mut node.nexts);
                   node = &mut node.nexts[i];
               },
               Some(i) => {
                   node = &mut node.nexts[i];
               },
           }
        }
        if node.values.len() < 256 {
            node.values.push(value);
        } else {
            panic!("登録できる値は1つのkeyに256個までです。")
        }
    }

    /// 末尾の要素を昇順で正しい位置まで前方に移動する
    ///
    /// # Arguments
    ///
    /// * `nodes` - ソート対象のノードの配列
    fn sort(nodes: &mut [Node<T>]) -> usize {
        for i in (1..nodes.len()).rev() {
            if nodes[i].key < nodes[i - 1].key {
                nodes.swap(i, i - 1);
            } else {
                return i;
            }
        }
        0
    }

    /// trieを探索する
    /// keyに対応する値が見つかったら値のスライスを返す
    ///
    /// # Arguments
    ///
    /// * `key` - 探索するkey
    pub fn get(&self, key: &str) -> Option<&[T]> {
        let mut node = &self.root;
        for &k in key.as_bytes() {
           match Self::binary_search(k, &node.nexts) {
               None    => return None ,
               Some(i) => node = &node.nexts[i],
           }
        }
        if node.values.is_empty() {
            None
        } else {
            Some(&node.values)
        }
    }

    /// nodesからkeyを持つNode<K, T>を二分探索で探索する
    /// 見つかった場合はそのindexを返す
    ///
    /// # Arguments
    ///
    /// * `key`   - 探索するkey
    /// * `nodes` - 探索対象のノードの配列
    fn binary_search(key: u8, nodes: &[Node<T>]) -> Option<usize> {
        if nodes.is_empty() {
            return None;
        }
        let mut s = 0;
        let mut e = nodes.len();
        loop {
            if s >= e {
                break;
            }
            let pivot = (s + e) / 2;
            let target = nodes[pivot].key;
            if key < target {
                e = pivot;
            } else if key > target {
                s = pivot + 1;
            } else {
                return Some(pivot);
            }
        }
        None
    }

    /// トライ木をダブル配列に変換する
    ///
    /// # Arguments
    ///
    /// * `len` - ダブル配列の初期サイズ
    pub fn to_double_array(self, mut len: usize) -> (Vec<u32>, Vec<u32>, Vec<T>) {
        let max_key = u8::max_value() as usize + 1;      // keyが取りうる値のパターン
        len = if max_key >= len { max_key } else { len };
        let mut base_arr: Vec<u32> = vec![0; len];
        let mut check_arr: Vec<u32> = vec![0; len];
        let mut data_arr: Vec<T> = vec![];
        let mut bit_cache: BitCache = BitCache::new();
        bit_cache.set(0);
        bit_cache.set(1);
        let mut stack: Vec<(usize, Node<T>)> = vec![];
        if !self.root.nexts.is_empty() {
            stack.push((1, self.root));
        }
        while !stack.is_empty() {
            let (curr_idx, node) = stack.pop().unwrap();
            // base値を求める
            let base: usize = if node.values.is_empty() {
                Self::find_base(&node.nexts, &bit_cache, false)
            } else {
                let base = Self::find_base(&node.nexts, &bit_cache, true);
                bit_cache.set(base);
                // valuesの格納: baseには「24bit: dataのindex, 8bit: 長さ」を格納し、dataには末尾にvaluesを追加する
                base_arr[base]  = ((data_arr.len() << 8) | node.values.len() & 0b11111111) as u32;
                check_arr[base] = curr_idx as u32;
                data_arr.extend_from_slice(&node.values);
                base
            };

            // base値をセット
            base_arr[curr_idx] = base as u32;

            // 配列の長さが足りなければ配列を拡張
            if base + max_key >= len {
                len = len * 2;
                base_arr.resize(len, 0);
                check_arr.resize(len, 0);
            }

            // 新しいノードをダブル配列に登録
            for n in node.nexts {
                let i = base + (n.key as usize);
                bit_cache.set(i);
                check_arr[i] = curr_idx as u32;
                stack.push((i, n));
            }
        }
        // 配列のリサイズ
        let new_len = match bit_cache.last_index_of_one() {
            None          => max_key,
            Some(new_len) => new_len + max_key,
        };
        base_arr.resize(new_len, 0);
        check_arr.resize(new_len, 0);
        (base_arr, check_arr, data_arr)
    }

    /// 新しいbase値を探索するメソッド
    ///
    /// # Arguments
    ///
    /// * `nodes`     - 追加対象のノード
    /// * `bit_cache` - BitCacheのインスタンス
    /// * `with_zero` - key=0のノードも考慮してbase値を探す
    fn find_base(nodes: &[Node<T>], bit_cache: &BitCache, with_zero: bool) -> usize {
        if !with_zero && nodes.is_empty() {
                panic!("探索すべきノードがありません");
        }
        let first_key = if with_zero { 0 } else { nodes[0].key as usize };
        let mut offset = first_key;
        'outer: loop {
            let empty_idx = bit_cache.find_empty_idx(offset);
            let new_base =  empty_idx - first_key;
            // すべてのノードが重複せずに配置できるかをチェック
            'inner: for next in nodes {
                if bit_cache.get(new_base + next.key as usize) != 0 {
                    // 空じゃなかった場合はnew_baseを探すとこからやり直し
                    offset = empty_idx + 1;
                    continue 'outer;
                }
            }
            return new_base;
        }
    }
}

/// ダブル配列をデバッグ目的で表示するための関数
#[allow(dead_code)]
fn debug_double_array<T: std::fmt::Debug>(base_arr: &[u32], check_arr: &[u32], data_arr: &[T]) {
    println!("size: base={}, check={}, data={}", base_arr.len(), check_arr.len(), data_arr.len());
    println!("{:-10} | {:-10} | {:-10} |", "index", "base", "check");
    println!("{:-10} | {:-10} | {:-10} |", 0, base_arr[0], check_arr[0]);
    println!("{:-10} | {:-10} | {:-10} |", 1, base_arr[1], check_arr[1]);
    for i in 2..base_arr.len() {
        let check = check_arr[i];
        if  check != 0 {
            if i == base_arr[check as usize] as usize {
                let data_idx = (base_arr[i] >> 8) as usize;
                let data_len = (base_arr[i] & 0b11111111) as usize;
                println!(
                    "{:-10} | {:-10} | {:-10} | {:?}",
                    i,
                    base_arr[i],
                    check_arr[i],
                    &data_arr[data_idx..(data_idx + data_len)],
                );
            } else {
                println!(
                    "{:-10} | {:-10} | {:-10} |",
                    i,
                    base_arr[i],
                    check_arr[i],
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trie_1() {
        let mut trie: Trie<i32> = Trie::new();
        let s = String::from("abc");
        trie.set(&s, 0);
        trie.set(&s, 1);
        // 登録されたkeyと値が一致している
        assert_eq!(0, trie.get(&s).unwrap()[0]);
        assert_eq!(1, trie.get(&s).unwrap()[1]);
        let s = String::from("cba");
        // 登録されていないkeyはNoneを返す
        assert_eq!(None, trie.get(&s));
    }

    #[test]
    fn test_trie_2() {
        let mut trie: Trie<String> = Trie::new();
        let s1 = String::from("abc");
        let s2 = String::from("abd");
        let s3 = String::from("zyx");
        let s4 = String::from("zwx");
        trie.set(&s1, String::from("abc"));
        trie.set(&s2, String::from("abd"));
        trie.set(&s3, String::from("zyx"));
        trie.set(&s4, String::from("zwx"));
        trie.set(&s1, String::from("abc"));
        // 登録されたkeyと値が一致している
        assert_eq!(s1, trie.get(&s1).unwrap()[0]);
        assert_eq!(s1, trie.get(&s1).unwrap()[1]);
        assert_eq!(s1, trie.get(&s1).unwrap()[0]);
        assert_eq!(s2, trie.get(&s2).unwrap()[0]);
        assert_eq!(s3, trie.get(&s3).unwrap()[0]);
        assert_eq!(s4, trie.get(&s4).unwrap()[0]);
    }

    #[test]
    fn test_trie_3() {
        let mut trie: Trie<u32> = Trie::new();
        let s1 = String::from("あいうえお");
        let s2 = String::from("あいえうお");
        let s3 = String::from("漢字");
        let s4 = String::from("平仮名");
        let s5 = String::from("片仮名");
        trie.set(&s1, 10);
        trie.set(&s2, 11);
        trie.set(&s3, 12);
        trie.set(&s4, 13);
        trie.set(&s5, 14);
        // 登録されたkeyと値が一致している
        assert_eq!(10, trie.get(&s1).unwrap()[0]);
        assert_eq!(11, trie.get(&s2).unwrap()[0]);
        assert_eq!(12, trie.get(&s3).unwrap()[0]);
        assert_eq!(13, trie.get(&s4).unwrap()[0]);
        assert_eq!(14, trie.get(&s5).unwrap()[0]);
    }

    #[test]
    fn test_find_base_1() {
        let nodes: Vec<Node<u32>> = vec![
            Node::<u32> { key: 1, values: vec![], nexts: vec![] },
            Node::<u32> { key: 2, values: vec![], nexts: vec![] },
            Node::<u32> { key: 5, values: vec![], nexts: vec![] },
        ];
        let mut bit_cache = BitCache::new();

        // 0~1が埋まっている。1, 2, 5を登録できるbase値は2
        bit_cache.set(0);
        bit_cache.set(1);
        assert_eq!(1, Trie::find_base(&nodes, &bit_cache, false));

        // 0~1, 5~62が埋まっている。1, 2, 5を登録できるbase値は63
        for i in (5..63) { bit_cache.set(i); }
        assert_eq!(62, Trie::find_base(&nodes, &bit_cache, false));

        // 0~1, 5~65535が埋まっている。1, 2, 5を登録できるbase値は65536
        for i in (63..65536) { bit_cache.set(i); }
        assert_eq!(65535, Trie::find_base(&nodes, &bit_cache, false));
    }

    #[test]
    fn test_find_base_2() {
        let nodes: Vec<Node<u32>> = vec![
            Node::<u32> { key: 1, values: vec![], nexts: vec![] },
            Node::<u32> { key: 2, values: vec![], nexts: vec![] },
            Node::<u32> { key: 5, values: vec![], nexts: vec![] },
        ];
        let mut bit_cache = BitCache::new();

        // 0~1が埋まっている。0, 1, 2, 5を登録できるbase値は2
        bit_cache.set(0);
        bit_cache.set(1);
        assert_eq!(2, Trie::find_base(&nodes, &bit_cache, true));

        // 0~1, 5~62が埋まっている。0, 1, 2, 5を登録できるbase値は63
        for i in (5..63) { bit_cache.set(i); }
        assert_eq!(63, Trie::find_base(&nodes, &bit_cache, true));

        // 0~1, 5~65535が埋まっている。0, 1, 2, 5を登録できるbase値は65536
        for i in (63..65536) { bit_cache.set(i); }
        assert_eq!(65536, Trie::find_base(&nodes, &bit_cache, true));
    }

    #[test]
    #[should_panic("探索すべきノードがありません")]
    fn test_find_base_3() {
        let nodes: Vec<Node<u32>> = vec![];
        let mut bit_cache = BitCache::new();
        // nodesが空でwith_zero=falseの場合は、base値を求められないのでpanic
        Trie::find_base(&nodes, &bit_cache, false);
    }

    #[test]
    fn test_to_double_array_1() {
        let mut trie: Trie<u32> = Trie::new();
        let s1 = String::from("abc");
        let s2 = String::from("ac");
        let s3 = String::from("b");
        let s4 = String::from("bd");
        let s5 = String::from("bdc");
        trie.set(&s1, 1);
        trie.set(&s1, 2);
        trie.set(&s2, 3);
        trie.set(&s3, 4);
        trie.set(&s4, 5);
        trie.set(&s5, 6);
        let (base_arr, check_arr, data_arr) = trie.to_double_array(255);
        //debug_double_array(&base_arr, &check_arr, &data_arr);
        // 登録されていて、data_arrに値が存在するkeyは対応する値を返す
        assert_eq!([1, 2], find(&s1, &base_arr, &check_arr, &data_arr).unwrap());
        assert_eq!([3], find(&s2, &base_arr, &check_arr, &data_arr).unwrap());
        assert_eq!([4], find(&s3, &base_arr, &check_arr, &data_arr).unwrap());
        assert_eq!([5], find(&s4, &base_arr, &check_arr, &data_arr).unwrap());
        assert_eq!([6], find(&s5, &base_arr, &check_arr, &data_arr).unwrap());
        // 登録されているが、data_arrに値が存在しないkeyはNoneを返す
        assert_eq!(None, find("ab", &base_arr, &check_arr, &data_arr));
    }

    #[test]
    #[should_panic (expected = "(idx=1, base=0, check=0)から(idx=97, base=0, check=0)に遷移できません。(key=abc, i=0, byte=97)")]
    fn test_to_double_array_2() {
        let mut trie: Trie<u32> = Trie::new();
        let (base_arr, check_arr, data_arr) = trie.to_double_array(255);
        let s1 = String::from("abc");
        // 遷移できない場合はpanicする
        find(&s1, &base_arr, &check_arr, &data_arr).unwrap();
    }

    #[test]
    fn test_to_double_array_3() {
        // マルチバイト文字のテスト
        let mut trie: Trie<u32> = Trie::new();
        let s1 = String::from("おすしとビール");
        let s2 = String::from("お寿司とビール");
        let s3 = String::from("🍣🍺");
        trie.set(&s1, 1);
        trie.set(&s1, 2);
        trie.set(&s2, 3);
        trie.set(&s3, 4);
        let (base_arr, check_arr, data_arr) = trie.to_double_array(255);
        // 登録されていて、data_arrに値が存在するkeyは対応する値を返す
        assert_eq!([1, 2], find(&s1, &base_arr, &check_arr, &data_arr).unwrap());
        assert_eq!([3], find(&s2, &base_arr, &check_arr, &data_arr).unwrap());
        assert_eq!([4], find(&s3, &base_arr, &check_arr, &data_arr).unwrap());
        // 登録されているが、data_arrに値が存在しないkeyはNoneを返す
        assert_eq!(None, find("お寿", &base_arr, &check_arr, &data_arr));
    }


    /// ダブル配列から指定されたkeyを探索する関数
    /// 途中で遷移できなくなった場合はpanicする
    /// 遷移はできたが、data_arrに値が存在しない場合はNoneを返す
    /// 遷移ができて、data_arrに値が存在する場合はdata_arrのスライスを返す
    ///
    /// # Arguments
    ///
    /// * `key`       - 探索対象の文字列
    /// * `base_arr`  - base配列
    /// * `check_arr` - check配列
    /// * `data_arr`  - data配列
    fn find<'a, T>(key: &str, base_arr: &[u32], check_arr: &[u32], data_arr: &'a [T]) -> Option<&'a [T]> {
        let mut idx  = 1;
        let mut base = base_arr[idx] as usize;

        for (i, &byte) in key.as_bytes().iter().enumerate() {
            let next_idx = base + (byte as usize);
            if  check_arr[next_idx] as usize == idx {
                idx  = next_idx;
                base = base_arr[idx] as usize;
            } else {
                panic!(
                    "(idx={}, base={}, check={})から(idx={}, base={}, check={})に遷移できません。(key={}, i={}, byte={})",
                    idx     , base_arr[idx]     , check_arr[idx],
                    next_idx, base_arr[next_idx], check_arr[next_idx],
                    key     , i                 , byte,
                );
            }
        }
        if check_arr[base] as usize == idx {
            let data_idx = (base_arr[base] >> 8) as usize;
            let data_len = (base_arr[base] & 0b11111111) as usize;
            Some(&data_arr[data_idx..(data_idx + data_len)])
        } else {
            None
        }
    }
}
