/// 連接コスト表を構築する構造体
#[allow(dead_code)]
pub struct MatrixBuilder {
    matrix: Vec<u16>,
    left_max: usize,
    right_max: usize,
}

impl MatrixBuilder {
    pub fn new(left_max: usize, right_max: usize) -> MatrixBuilder {
        MatrixBuilder { matrix: vec![0; left_max * right_max], left_max, right_max }
    }

    pub fn get_matrix(&self) -> &[u16] {
        &self.matrix
    }

    pub fn get_left_max(&self) -> usize {
        self.left_max
    }

    pub fn get_right_max(&self) -> usize {
        self.right_max
    }

    /// 連接コスト表からコストを引く
    ///
    /// # Arguments
    ///
    /// * `left_id`  - 左文脈ID
    /// * `right_id` - 右文脈ID
    pub fn get(&self, left_id: usize, right_id: usize) -> u16 {
        self.matrix[(left_id * self.right_max) + right_id]
    }

    /// 連接コスト表に値をセットする
    ///
    /// # Arguments
    ///
    /// * `left_id`  - 左文脈ID
    /// * `right_id` - 右文脈ID
    /// * `cost`     - 連接コスト
    pub fn set(&mut self, left_id: usize, right_id: usize, cost: u16) {
        self.matrix[(left_id * self.right_max) + right_id] = cost;
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matrix_1() {
        let mut matrix = MatrixBuilder::new(100, 100);
        for l in 0u16..100 {
            for r in 0u16..100 {
                matrix.set(l as usize, r as usize, l * 100 + r);
            }
        }

        for l in 0u16..100 {
            for r in 0u16..100 {
                assert_eq!(l * 100 + r, matrix.get(l as usize, r as usize));
            }
        }
    }
}
