extern crate midomoji_clone;

use midomoji_clone::dictionary::*;
use midomoji_clone::token::Token;
use midomoji_clone::lattice::Lattice;

use std::io::prelude::*;
use std::io::BufReader;
use std::io::BufWriter;
use std::fs::File;
use memmap::*;

#[test]
fn test_lattice() {
    // 辞書構築
    // 辞書ファイルは予め↓で作っておく
    // cargo run --bin build-dict --release -- tests/lattice_test/lex.csv tests/lattice_test/matrix.def tests/lattice_test/ipa.dic
    let dict_file: File = File::open("tests/lattice_test/ipa.dic").unwrap();
    let mmap: Mmap = unsafe {
        MmapOptions::new().map(&dict_file).ok().unwrap()
    };
    let dict_set: DictionarySet<Token> = DictionarySet::new(&mmap);

    // reader
    let mut reader: BufReader<File> = BufReader::new(File::open("tests/lattice_test/input.tsv").unwrap());

    for line in reader.lines() {
        let line = line.unwrap();
        let mut split = line.trim().split("\t");
        // クエリ文字列
        let query: &str    = split.next().unwrap();
        // 結果文字列
        let result: &str = split.next().unwrap();

        // 期待するトークンを格納する配列
        let mut expected: Vec<(String, Token)> = Vec::new();
        for e in result.split(";") {
            let mut token_str = e.split(",");
            let surface  = token_str.next().unwrap().to_string();
            let left_id  = token_str.next().unwrap().parse::<u16>().unwrap();
            let right_id = token_str.next().unwrap().parse::<u16>().unwrap();
            let cost     = token_str.next().unwrap().parse::<i16>().unwrap();
            let expected_token = Token { left_id, right_id, cost }; // 期待するトークン
            expected.push((surface, expected_token));
        }

        // ラティス構造構築
        let mut lattice = Lattice::build(&dict_set, query.as_bytes());
        lattice.analyze(&dict_set);
        let actual = lattice.get_result();

        // 解析結果が期待する値になっているかを検証
        for (i, node) in actual[1..(actual.len() - 1)].iter().rev().enumerate() {
            // 解析結果のトークン
            let actual_token = (std::str::from_utf8(node.surface).unwrap().to_string(), node.token);
            assert_eq!(actual_token, expected[i]);
        }
    }


}

