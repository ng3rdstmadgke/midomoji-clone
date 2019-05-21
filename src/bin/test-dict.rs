extern crate csv;

use midomoji_clone::dictionary::*;
use midomoji_clone::token::Token;
use midomoji_clone::util::*;

use std::env;
use std::env::Args;
use std::io::prelude::*;
use std::fs::File;
use std::io::BufReader;
use std::collections::HashMap;
use memmap::*;

fn main() {
    let options = parse_args(env::args());
    let lex    = options.get("lex").unwrap();
    let matrix = options.get("matrix").unwrap();
    let dict   = options.get("dict").unwrap();
    test(lex, matrix, dict);
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
            } else if options.get("dict") == None {
                options.insert("dict".to_string(), arg);
            } else {
                panic!("不明なオプション: {}", arg);
            }
        }
    }
    let required_opts = ["lex", "matrix", "dict"];
    for k in required_opts.iter() { // k は std::borrow::Borrow<&str>
        if options.get(*k) == None {
            panic!("usage: {} <LEX_PATH> <MATRIX_PATH> <DICT_PATH>", script);
        }
    }
    options
}

fn test(lex: &str, matrix: &str, dict: &str) {
    let mut timer = Timer::new();
    // 辞書読み込み
    timer.start();
    let file: File = File::open(&dict).ok().unwrap();
    let mmap: Mmap = unsafe {
        MmapOptions::new().map(&file).ok().unwrap()
    };
    let dict_set: DictionarySet<Token> = DictionarySet::new(&mmap);
    println!("load dictionary complete");
    timer.stop();
    timer.print();

    // matrix test
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
            if dict_set.get_matrix(left_id, right_id) != cost {
                panic!(
                    "matrix: left_id={}, right_id={}, cost(file)={}, cost(index)={}",
                    left_id,
                    right_id,
                    cost,
                    dict_set.get_matrix(left_id, right_id),
                );
            }
        }
    }
    println!("test matrix complete");
    timer.stop();
    timer.print();

    // trie test
    timer.reset();
    timer.start();
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
            let values = dict_set.get_trie(&lex);
            if values.is_some() {
                let mut exists = false;
                for v in values.unwrap() {
                    if token == *v {
                        exists = true;
                        break;
                    }
                }
                if !exists {
                    panic!("trie: lex={}, token={:?}", lex, token);
                }
            } else {
                panic!("trie: lex={}, token={:?}", lex, token);
            }
        }
    }
    println!("test trie complete");
    timer.stop();
    timer.print();
}
