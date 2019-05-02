struct Node<K, V> where K: Copy + Ord + Eq {
    key   : Option<K>,
    values: Vec<V>,
    nexts : Vec<Node<K, V>>,
}

pub struct Trie<K, V> where K: Copy + Ord + Eq {
    root: Node<K, V>,
}

impl<K, V> Trie<K, V> where K: Copy + Ord + Eq {
    pub fn new() -> Trie<K, V> {
        Trie { root: Node { key: None, values: Vec::new(), nexts: Vec::new() }}
    }

    /// trieにノードを追加する
    pub fn set(&mut self, key: &[K], value: V) {
        let mut node = &mut self.root;
        for &k in key {
           match Self::binary_search(k, &node.nexts) {
               None    => {
                   node.nexts.push(Node { key: Some(k), values: Vec::new(), nexts: Vec::new() });
                   let i = Self::sort(&mut node.nexts);
                   node = &mut node.nexts[i];
               },
               Some(i) => {
                   node = &mut node.nexts[i];
               },
           }
        }
        node.values.push(value);
    }

    /// 末尾の要素を正しい位置まで前方に移動する
    fn sort(nodes: &mut [Node<K, V>]) -> usize {
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
    pub fn get(&self, key: &[K]) -> Option<&[V]> {
        let mut node = &self.root;
        for &k in key {
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

    /// nodesからkeyを持つNode<K, V>を二分探索で探索する
    fn binary_search(key: K, nodes: &[Node<K, V>]) -> Option<usize> {
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
            let target = nodes[pivot].key.unwrap();
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trie_1() {
        let mut trie: Trie<u8, i32> = Trie::new();
        let s = String::from("abc");
        trie.set(s.as_bytes(), 0);
        trie.set(s.as_bytes(), 1);
        let ret = trie.get(s.as_bytes()).unwrap();
        assert_eq!(0, ret[0]);
        assert_eq!(1, ret[1]);
        let s = String::from("cba");
        assert_eq!(None, trie.get(s.as_bytes()));
    }

    #[test]
    fn test_trie_2() {
        let mut trie: Trie<u8, String> = Trie::new();
        let s1 = String::from("abc");
        let s2 = String::from("abd");
        let s3 = String::from("zyx");
        let s4 = String::from("zwx");
        trie.set(s1.as_bytes(), String::from("abc"));
        trie.set(s2.as_bytes(), String::from("abd"));
        trie.set(s3.as_bytes(), String::from("zyx"));
        trie.set(s4.as_bytes(), String::from("zwx"));
        trie.set(s1.as_bytes(), String::from("abc"));
        assert_eq!(s1, trie.get(s1.as_bytes()).unwrap()[0]);
        assert_eq!(s1, trie.get(s1.as_bytes()).unwrap()[1]);
        assert_eq!(s1, trie.get(s1.as_bytes()).unwrap()[0]);
        assert_eq!(s2, trie.get(s2.as_bytes()).unwrap()[0]);
        assert_eq!(s3, trie.get(s3.as_bytes()).unwrap()[0]);
        assert_eq!(s4, trie.get(s4.as_bytes()).unwrap()[0]);
    }

    #[test]
    fn test_trie_3() {
        let mut trie: Trie<u8, u32> = Trie::new();
        let s1 = String::from("あいうえお");
        let s2 = String::from("あいえうお");
        let s3 = String::from("漢字");
        let s4 = String::from("平仮名");
        let s5 = String::from("片仮名");
        trie.set(s1.as_bytes(), 10);
        trie.set(s2.as_bytes(), 11);
        trie.set(s3.as_bytes(), 12);
        trie.set(s4.as_bytes(), 13);
        trie.set(s5.as_bytes(), 14);
        assert_eq!(10, trie.get(s1.as_bytes()).unwrap()[0]);
        assert_eq!(11, trie.get(s2.as_bytes()).unwrap()[0]);
        assert_eq!(12, trie.get(s3.as_bytes()).unwrap()[0]);
        assert_eq!(13, trie.get(s4.as_bytes()).unwrap()[0]);
        assert_eq!(14, trie.get(s5.as_bytes()).unwrap()[0]);
    }
}
