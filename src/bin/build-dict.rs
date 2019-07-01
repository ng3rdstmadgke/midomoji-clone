extern crate csv;

use midomoji_clone::dictionary::*;
use midomoji_clone::dictionary::trie::Trie;
use midomoji_clone::dictionary::matrix_builder::MatrixBuilder;
use midomoji_clone::token::Token;
use midomoji_clone::util::*;

use std::env;
use std::env::Args;
use std::io::prelude::*;
use std::fs::File;
use std::io::BufReader;
use std::collections::HashMap;

fn main() {
    let options = parse_args(env::args());
    let lex    = options.get("lex").unwrap();
    let matrix = options.get("matrix").unwrap();
    let output = options.get("output").unwrap();
    build(lex, matrix, output);
}

fn parse_args(mut args: Args) -> HashMap<String, String> {
    let mut options = HashMap::new();
    let script = args.next().unwrap();
    let mut key: Option<String> = None;
    for arg in args {
        if let Some(k) = key {
            options.insert(k.clone(), arg.to_string());
            key = None;
        } else {
            if options.get("lex") == None {
                options.insert("lex".to_string(), arg);
            } else if options.get("matrix") == None {
                options.insert("matrix".to_string(), arg);
            } else if options.get("output") == None {
                options.insert("output".to_string(), arg);
            } else {
                panic!("不明なオプション: {}", arg);
            }
        }
    }
    let required_opts = ["lex", "matrix", "output"];
    for k in required_opts.iter() { // k は std::borrow::Borrow<&str>
        if options.get(*k) == None {
            panic!("usage: {} <LEX_PATH> <MATRIX_PATH> <OUTPUT_PATH>", script);
        }
    }
    options
}

fn build(lex: &str, matrix: &str, output: &str) {
    let mut timer = Timer::new();
    // Err(Error::new(ErrorKind::InvalidData, "invalid format. left_max, right_max not found."));
    let mut matrix_builder = MatrixBuilder::new(0, 0);
    // header読み込み
    timer.start();
    {
        let matrix_reader: BufReader<File> = BufReader::new(File::open(&matrix).ok().unwrap());
        for result in matrix_reader.lines() {
            let line: String = result.ok().unwrap();
            let mut record = line.trim().split_whitespace();
            let left_max: usize  = record.next().unwrap().parse::<usize>().ok().unwrap();
            let right_max: usize = record.next().unwrap().parse::<usize>().ok().unwrap();
            matrix_builder = MatrixBuilder::new(left_max, right_max);
            break;
        }
    }
    println!("build header complete");
    timer.stop();
    timer.print();

    // matrix構築
    timer.reset();
    timer.start();
    {
        let matrix_reader: BufReader<File> = BufReader::new(File::open(&matrix).ok().unwrap());
        let mut lines = matrix_reader.lines();
        lines.next();
        for result in lines {
            let line: String = result.ok().unwrap();
            let mut record = line.trim().split_whitespace();
            let left_id: usize  = record.next().unwrap().parse::<usize>().ok().unwrap();
            let right_id: usize = record.next().unwrap().parse::<usize>().ok().unwrap();
            let cost: i16       = record.next().unwrap().parse::<i16>().ok().unwrap();
            matrix_builder.set(left_id, right_id, cost);
        }
    }
    println!("build matrix complete");
    timer.stop();
    timer.print();

    // 形態素辞書構築
    // TODO: なぜか長さ0のトークンが登録されてしまう
    timer.reset();
    timer.start();
    let mut trie: Trie<Token> = Trie::new();
    {
        let mut lex_reader = csv::Reader::from_reader(File::open(lex).ok().unwrap());
        for result in lex_reader.records() {
            let record = result.ok().unwrap();
            let lex = &record[0];
            let token = Token {
                left_id : record[1].parse::<u16>().ok().unwrap(),
                right_id: record[2].parse::<u16>().ok().unwrap(),
                cost    : record[3].parse::<i16>().ok().unwrap(),
            };
            trie.set(lex, token);
        }
    }
    println!("build trie complete");
    timer.stop();
    timer.print();

    timer.reset();
    timer.start();
    let (base_arr, check_arr, data_arr) = trie.to_double_array();
    println!("build double_array complete");
    timer.stop();
    timer.print();

    // 辞書の書き込み
    timer.reset();
    timer.start();
    DictionarySet::serialize(&base_arr, &check_arr, &data_arr, matrix_builder, &output).ok().unwrap();
    println!("serialize dictionary complete");
    timer.stop();
    timer.print();
}
