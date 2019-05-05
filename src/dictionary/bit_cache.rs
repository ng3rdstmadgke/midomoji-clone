/// 空いているindexをbitで管理する
/// 0: 空, 1: 空でない
pub struct BitCache {
    cache: Vec<i64>,
}

impl BitCache {
    pub fn new() -> BitCache {
        BitCache { cache: vec![0; 1024] }
    }

    /// 指定されたインデックスを取得する
    /// 空なら0, 空でないなら0以外
    ///
    /// # Arguments
    ///
    /// * `idx`- 調べたいindex
    pub fn get(&self, idx: usize) -> usize {
        let arr_idx: usize = idx >> 6;       // idx / 64
        let bit_idx: usize = idx & 0b111111; // idx % 64
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
        let arr_idx: usize = idx >> 6;       // idx / 64
        let bit_idx: usize = idx & 0b111111; // idx % 64
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
        let arr_idx: usize = offset >> 6;       // idx / 64
        let bit_idx: usize = offset & 0b111111; // idx % 64
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
                return cnt * 64 + zeros;
            }
            mask = -1;
            cnt += 1;
        }
        self.cache.len() * 64
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
}
