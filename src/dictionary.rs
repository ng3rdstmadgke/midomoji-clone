pub mod trie;
pub mod matrix_builder;
mod bit_cache;

use self::matrix_builder::MatrixBuilder;

use std::fmt::Debug;
use std::slice;
use std::mem;
use std::io;
use std::io::prelude::*;
use std::fs::File;
use std::ptr;

#[allow(dead_code)]
pub struct DictionaryHeader {
    base_idx        : usize,
    check_idx       : usize,
    data_idx        : usize,
    matrix_idx      : usize,
    base_len        : usize,
    check_len       : usize,
    data_len        : usize,
    matrix_len      : usize,
    matrix_left_max : usize,
    matrix_right_max: usize,
}


pub struct DictionarySet<'a, T: Copy + Debug> {
    header   : DictionaryHeader,
    pub base_arr : &'a [u32],
    pub check_arr: &'a [u32],
    pub data_arr : &'a [T],
    pub matrix   : &'a [i16],
}

impl<'a, T: Copy + Debug> DictionarySet<'a, T> {
    /**
     * byte列を辞書として読み込む
     */
    pub fn new(bytes: &[u8]) -> DictionarySet<'a, T> {
        // header
        let header: DictionaryHeader = unsafe {
            ptr::read(bytes.as_ptr() as *const DictionaryHeader)
        };

        // base_arr
        let base_arr: &'a [u32] = unsafe {
            slice::from_raw_parts(
                bytes[header.base_idx..].as_ptr() as *const u32,
                header.base_len
            )
        };

        // check_arr
        let check_arr: &'a [u32] = unsafe {
            slice::from_raw_parts(
                bytes[header.check_idx..].as_ptr() as *const u32,
                header.check_len
            )
        };

        // data_arr
        let data_arr: &'a [T] = unsafe {
            slice::from_raw_parts(
                bytes[header.data_idx..].as_ptr() as *const T,
                header.data_len
            )
        };

        // matrix
        let matrix: &'a [i16] = unsafe {
            slice::from_raw_parts(
                bytes[header.matrix_idx..].as_ptr() as *const i16,
                header.matrix_len
            )
        };

        DictionarySet { header, base_arr, check_arr, data_arr, matrix }
    }

    /// ダブル配列から指定されたkeyを探索する関数
    /// 途中で遷移できなくなった場合、data_arrに値が存在しない場合はNoneを返す
    /// 遷移ができて、data_arrに値が存在する場合はdata_arrのスライスを返す
    /// デバッグ用
    ///
    /// # Arguments
    ///
    /// * `key`       - 探索対象の文字列
    pub fn get_trie(&self, key: &str) -> Option<&'a [T]> {
        let mut idx  = 1;
        let mut base = self.base_arr[idx] as usize;

        for &byte in key.as_bytes() {
            let next_idx = base + (byte as usize);
            if  self.check_arr[next_idx] as usize != idx {
                return None;
            }
            idx  = next_idx;
            base = self.base_arr[idx] as usize;
        }
        let value_idx = base + (u8::max_value() as usize);
        if self.check_arr[value_idx] as usize == idx {
            let data_idx = (self.base_arr[value_idx] >> 8) as usize;
            let data_len = (self.base_arr[value_idx] & 0b11111111) as usize;
            Some(&self.data_arr[data_idx..(data_idx + data_len)])
        } else {
            None
        }
    }

    /// ダブル配列で共通接頭辞検索を行う
    /// デバッグ用
    ///
    /// # Arguments
    ///
    /// * `key`       - 探索対象の文字列
    pub fn prefix_search(&self, key: &'a str) -> Vec<(&'a str, &'a[T])> {
        let mut ret: Vec<(&str, &[T])> = Vec::new();
        let mut idx = 1;
        let mut base = self.base_arr[idx] as usize;

        for (i, &byte) in key.as_bytes().iter().enumerate() {
            // 次のノードに遷移
            let next_idx = base + (byte as usize);
            if self.check_arr[next_idx] as usize != idx {
                break;
            }
            idx = next_idx;
            base = self.base_arr[idx] as usize;
            // value があれば戻り値の配列に追加
            let value_idx = base + (u8::max_value() as usize);
            if self.check_arr[value_idx] as usize == idx {
                let data_idx = (self.base_arr[value_idx] >> 8) as usize;
                let data_len = (self.base_arr[value_idx] & 0b11111111) as usize;
                ret.push((&key[0..(i + 1)], &self.data_arr[data_idx..(data_idx + data_len)]));
            }
        }
        ret
    }

    /// ダブル配列から指定されたkeyを探索する関数
    /// 途中で遷移できなくなった場合、data_arrに値が存在しない場合はNoneを返す
    /// 遷移ができて、data_arrに値が存在する場合はdata_arrのスライスを返す
    ///
    /// # Arguments
    ///
    /// * `key`       - 探索対象の文字列
    pub fn get_matrix(&self, left_id: usize, right_id: usize) -> i16 {
        let unknown_id = u16::max_value() as usize;
        if left_id == unknown_id || right_id == unknown_id {
            // アドホックな未知語の連接コストの対応。
            // TODO: 未知語の設定ファイルを読み込めるようにする
            return i16::max_value();
        }
        self.matrix[(left_id * self.header.matrix_right_max) + right_id]
    }

    /// ダブル配列、連接コスト表をバイト列としてファイルに書き込む
    ///
    /// # Arguments
    ///
    /// * `output_path` - 出力するファイル
    pub fn serialize(base_arr: &[u32], check_arr: &[u32], data_arr: &[T], matrix: MatrixBuilder, output_path: &str) -> io::Result<()> {
        // base_arr
        let base_bytes: &[u8] = unsafe {
            slice::from_raw_parts(
                base_arr.as_ptr() as *const u8,
                mem::size_of::<u32>() * base_arr.len()
            )
        };
        // check_arr
        let check_bytes: &[u8] = unsafe {
            slice::from_raw_parts(
                check_arr.as_ptr() as *const u8,
                mem::size_of::<u32>() * check_arr.len()
            )
        };
        // data_arr
        let data_bytes: &[u8] = unsafe {
            slice::from_raw_parts(
                data_arr.as_ptr() as *const u8,
                mem::size_of::<T>() * data_arr.len()
            )
        };
        // matrix
        let matrix_bytes: &[u8] = unsafe {
            slice::from_raw_parts(
                matrix.get_matrix().as_ptr() as *const u8,
                mem::size_of::<u16>() * matrix.get_matrix().len()
            )
        };
        // dictionary_header
        let header_size: usize = mem::size_of::<DictionaryHeader>();
        let header = DictionaryHeader {
            base_idx        : header_size,
            check_idx       : header_size + base_bytes.len(),
            data_idx        : header_size + base_bytes.len() + check_bytes.len(),
            matrix_idx      : header_size + base_bytes.len() + check_bytes.len() + data_bytes.len(),
            base_len        : base_arr.len(),
            check_len       : check_arr.len(),
            data_len        : data_arr.len(),
            matrix_len      : matrix.get_matrix().len(),
            matrix_left_max : matrix.get_left_max(),
            matrix_right_max: matrix.get_right_max(),
        };
        let header_bytes: &[u8] = unsafe {
            slice::from_raw_parts(
                &header as *const DictionaryHeader as *const u8,
                header_size,
            )
        };

        let mut f = File::create(output_path)?;
        f.write_all(header_bytes)?;
        f.write_all(base_bytes)?;
        f.write_all(check_bytes)?;
        f.write_all(data_bytes)?;
        f.write_all(matrix_bytes)?;
        f.flush()?;
        Ok(())
    }

    /// ダブル配列をデバッグ目的で表示するための関数
    pub fn debug_double_array(&self, len: usize) {
        let base_arr = self.base_arr;
        let check_arr = self.check_arr;
        let data_arr = self.data_arr;
        println!("size: base={}, check={}, data={}", base_arr.len(), check_arr.len(), data_arr.len());
        println!("{:-10} | {:-10} | {:-10} |", "index", "base", "check");
        println!("{:-10} | {:-10} | {:-10} |", 0, base_arr[0], check_arr[0]);
        println!("{:-10} | {:-10} | {:-10} |", 1, base_arr[1], check_arr[1]);
        for i in 2..len {
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
}

/*
use std::iter::Iterator;
struct PrefixSearchIter<'a, T> {
    idx      : usize,
    base_arr : &'a [u32],
    check_arr: &'a [u32],
    data_arr : &'a [T],
}

impl<'a, T> Iterator for PrefixSearchIter<'a, T>  {
    type Item =  &'a [T];

    fn next(&mut self) -> Option<&'a [T]> {
        Some(&self.data_arr[0..1])
    }
}
*/

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dictionary_set_new() {
        // base_arr
        let base_arr: Vec<u32> = vec![1,2,3,4,5];
        let base_bytes: &[u8] = unsafe {
            slice::from_raw_parts( base_arr.as_ptr() as *const u8, mem::size_of::<u32>() * base_arr.len())
        };

        // check_arr
        let check_arr: Vec<u32> = vec![10,20,30,40,50];
        let check_bytes: &[u8] = unsafe {
            slice::from_raw_parts( check_arr.as_ptr() as *const u8, mem::size_of::<u32>() * check_arr.len())
        };

        // data_arr
        let data_arr: Vec<u32> = vec![100,200,300,400,500];
        let data_bytes: &[u8] = unsafe {
            slice::from_raw_parts( data_arr.as_ptr() as *const u8, mem::size_of::<u32>() * data_arr.len())
        };

        // matrix
        let matrix: Vec<u16> = vec![1000,2000,3000,4000];
        let matrix_bytes: &[u8] = unsafe {
            slice::from_raw_parts( matrix.as_ptr() as *const u8, mem::size_of::<u16>() * matrix.len())
        };

        // dictionary_header
        let header_size: usize = mem::size_of::<DictionaryHeader>();
        let header = DictionaryHeader {
            base_idx        : header_size,
            check_idx       : header_size + base_bytes.len(),
            data_idx        : header_size + base_bytes.len() + check_bytes.len(),
            matrix_idx      : header_size + base_bytes.len() + check_bytes.len() + data_bytes.len(),
            base_len        : base_arr.len(),
            check_len       : check_arr.len(),
            data_len        : data_arr.len(),
            matrix_len      : matrix.len(),
            matrix_left_max : 1,
            matrix_right_max: 2,
        };
        let header_bytes: &[u8] = unsafe {
            slice::from_raw_parts( &header as *const DictionaryHeader as *const u8, header_size)
        };

        let mut bytes: Vec<u8> = vec![];
        bytes.extend_from_slice(header_bytes);
        bytes.extend_from_slice(base_bytes);
        bytes.extend_from_slice(check_bytes);
        bytes.extend_from_slice(data_bytes);
        bytes.extend_from_slice(matrix_bytes);
        let dict_set: DictionarySet<u32> = DictionarySet::new(&bytes);
        assert_eq!([1,2,3,4,5]          , dict_set.base_arr);
        assert_eq!([10,20,30,40,50]     , dict_set.check_arr);
        assert_eq!([100,200,300,400,500], dict_set.data_arr);
        assert_eq!([1000,2000,3000,4000], dict_set.matrix);
        assert_eq!(1, dict_set.header.matrix_left_max);
        assert_eq!(2, dict_set.header.matrix_right_max);
    }
}
