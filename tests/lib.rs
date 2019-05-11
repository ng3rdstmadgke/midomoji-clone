extern crate midomoji_clone;

use midomoji_clone::dictionary::*;
use midomoji_clone::dictionary::trie::Trie;
use midomoji_clone::dictionary::matrix_builder::MatrixBuilder;
use memmap::*;
use std::fs::File;

#[test]
fn test_build_load_dictionary() {
    // --- --- --- 構築 --- --- ---
    // 連接コスト登録
    let mut matrix_builder = MatrixBuilder::new(100, 100);
    for l in 0..100 {
        for r in 0..100 {
            matrix_builder.set(l, r, (l as i16) * 100 + (r as i16));
        }
    }
    // 単語登録
    let mut trie: Trie<usize> = Trie::new();
    let words: Vec<String> = vec![
        String::from("abc"),
        String::from("abc"),
        String::from("abd"),
        String::from("ac"),
        String::from("acd"),
        String::from("おすしビール"),
        String::from("お寿司ビール"),
        String::from("🍣🍺"),
    ];
    for (i, w) in words.into_iter().enumerate() {
        trie.set(&w, i);
    }

    // ダブル配列構築
    let (base_arr, check_arr, data_arr) = trie.to_double_array();

    // 辞書書き込み
    DictionarySet::serialize(&base_arr, &check_arr, &data_arr, matrix_builder, "tests/test.dic").ok().unwrap();

    // --- --- --- 読み込み --- --- ---
    // 辞書読み込み
    let file: File = File::open("tests/test.dic").ok().unwrap();
    let mmap: Mmap = unsafe {
        MmapOptions::new().map(&file).ok().unwrap()
    };
    let dict_set: DictionarySet<usize> = DictionarySet::new(&mmap);

    // ダブル配列の探索
    assert_eq!([0, 1], dict_set.get_trie("abc").unwrap());
    assert_eq!([2]   , dict_set.get_trie("abd").unwrap());
    assert_eq!([3]   , dict_set.get_trie("ac").unwrap());
    assert_eq!([4]   , dict_set.get_trie("acd").unwrap());
    assert_eq!([5]   , dict_set.get_trie("おすしビール").unwrap());
    assert_eq!([6]   , dict_set.get_trie("お寿司ビール").unwrap());
    assert_eq!([7]   , dict_set.get_trie("🍣🍺").unwrap());
    assert_eq!(None, dict_set.get_trie("ahoge"));
    assert_eq!(None, dict_set.get_trie("お寿"));

    // 連接コスト表の探索
    for l in 0..100 {
        for r in 0..100 {
            let cost = (l as i16) * 100 + (r as i16);
            assert_eq!(cost, dict_set.get_matrix(l, r));
        }
    }

    std::fs::remove_file("tests/test.dic").ok().unwrap();
}
