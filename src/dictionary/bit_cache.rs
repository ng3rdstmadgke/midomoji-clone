/// 空いているindexをbitで管理する
/// 0: 空, 1: 空でない
pub struct BitCache {
    cache: Vec<i64>,
}

impl BitCache {
    const BIT_MASK: usize = 0b111111; // 63
    const BIT_LEN: usize = 64;
    const BIT_CNT: usize = 6;

    pub fn new() -> BitCache {
        BitCache { cache: vec![0; 32768] }
    }

    /// 指定されたインデックスを取得する
    /// 空なら0, 空でないなら0以外
    ///
    /// # Arguments
    ///
    /// * `idx`- 調べたいindex
    pub fn get(&self, idx: usize) -> usize {
        let arr_idx: usize = idx >> Self::BIT_CNT; // idx / Self::BIT_LEN
        let bit_idx: usize = idx & Self::BIT_MASK; // idx % Self::BIT_LEN
        if arr_idx < self.cache.len() {
            (self.cache[arr_idx] as usize) & (1 << bit_idx)
        } else {
            0
        }
    }

    /// 指定されたインデックスのビットを立てる
    ///
    /// # Arguments
    ///
    /// * `idx`- ビットを立てたいindex
    pub fn set(&mut self, idx: usize) {
        let arr_idx: usize = idx >> Self::BIT_CNT; // idx / Self::BIT_LEN
        let bit_idx: usize = idx & Self::BIT_MASK; // idx % Self::BIT_LEN
        if arr_idx >= self.cache.len() {
            self.cache.resize(arr_idx * 2, 0);
        }
        self.cache[arr_idx] |= 1 << bit_idx;
    }

    /// offset以降の最初の空いているインデックスを返す
    ///
    /// # Arguments
    ///
    /// * `offset`- 探索開始インデックス
    pub fn find_empty_idx(&self, offset: usize) -> usize {
        let arr_idx: usize = offset >> Self::BIT_CNT; // idx / Self::BIT_LEN
        let bit_idx: usize = offset & Self::BIT_MASK; // idx % Self::BIT_LEN
        if arr_idx >= self.cache.len() {
            return offset;
        }
        // offset よりも前のビットを0埋めするためのマスク
        let mut mask: i64 = -1 << bit_idx;
        let mut cnt = arr_idx;
        for &e in &self.cache[arr_idx..] {
            // bit反転しているので、0が要素あり、1が空
            let bits = (e ^ -1) & mask;
            if bits != 0 {
                // 右から連続した0の個数を数える。0の個数が空のindexとなる
                let zeros = bits.trailing_zeros() as usize;
                return cnt * Self::BIT_LEN + zeros;
            }
            mask = -1;
            cnt += 1;
        }
        self.cache.len() * Self::BIT_LEN
    }

    /// cacheの中で一番最後に現れる1のindexを返す
    pub fn last_index_of_one(&self) -> Option<usize> {
        for (i, &bits) in self.cache.iter().enumerate().rev() {
            if bits != 0 {
                // 左から連続した0の個数を数える。
                // (Self::BIT_LEN - (0の個数 + 1)) はビット内での1のindex
                let zeros = bits.leading_zeros() as usize;
                return Some((i * Self::BIT_LEN) + (Self::BIT_LEN - (zeros + 1)));
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_get() {
        let mut bit_cache = BitCache::new();
        bit_cache.set(0);
        bit_cache.set(100);
        bit_cache.set(100000);
        // セットしたindexが登録されている
        assert_eq!(false, bit_cache.get(0) == 0);
        assert_eq!(false, bit_cache.get(100) == 0);
        assert_eq!(false, bit_cache.get(100000) == 0);
        // セットしていないindexは登録されていない
        assert_eq!(true, bit_cache.get(1000000) == 0);
    }

    #[test]
    fn test_find_empty_idx_1() {
        let mut bit_cache = BitCache::new();
        for i in (1..65) {
            bit_cache.set(i);
        }
        for i in (66..300) {
            bit_cache.set(i);
        }
        // 0番目は空いているので0
        assert_eq!(0     , bit_cache.find_empty_idx(0));
        // 1~64番目までは埋まっているので65
        assert_eq!(65    , bit_cache.find_empty_idx(1));
        // 66~299番目までは埋まっているので300
        assert_eq!(300   , bit_cache.find_empty_idx(66));
        // 100000番目は配列に存在しないので空いていることになる
        assert_eq!(100000, bit_cache.find_empty_idx(100000));
    }

    #[test]
    fn test_find_empty_idx_2() {
        let mut bit_cache = BitCache::new();
        for i in (0..65536) {
            bit_cache.set(i);
        }
        assert_eq!(65536, bit_cache.find_empty_idx(65535));
    }

    #[test]
    fn test_last_index_of_one() {
        let mut bit_cache = BitCache::new();
        assert_eq!(None, bit_cache.last_index_of_one());
        bit_cache.set(0);
        assert_eq!(Some(0), bit_cache.last_index_of_one());
        bit_cache.set(63);
        assert_eq!(Some(63), bit_cache.last_index_of_one());
        bit_cache.set(300);
        assert_eq!(Some(300), bit_cache.last_index_of_one());
    }
}
