use crate::dictionary::bit_cache::BitCache;

struct Node<T> {
    key   : u8,
    values: Vec<T>,
    nexts : Vec<Node<T>>,
}

pub struct Trie<T> {
    root: Node<T>,
}

impl<T> Trie<T> {
    pub fn new() -> Trie<T> {
        Trie { root: Node { key: 0, values: Vec::new(), nexts: Vec::new() }}
    }

    /// trieにノードを追加する
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
        node.values.push(value);
    }

    /// 末尾の要素を正しい位置まで前方に移動する
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
        let ret = trie.get(&s).unwrap();
        assert_eq!(0, ret[0]);
        assert_eq!(1, ret[1]);
        let s = String::from("cba");
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
        assert_eq!(10, trie.get(&s1).unwrap()[0]);
        assert_eq!(11, trie.get(&s2).unwrap()[0]);
        assert_eq!(12, trie.get(&s3).unwrap()[0]);
        assert_eq!(13, trie.get(&s4).unwrap()[0]);
        assert_eq!(14, trie.get(&s5).unwrap()[0]);
    }
}
