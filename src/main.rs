extern crate csv;

use midomoji_clone::dictionary::*;
use midomoji_clone::dictionary::trie::Trie;
use midomoji_clone::dictionary::matrix_builder::MatrixBuilder;
use midomoji_clone::token::Token;
use midomoji_clone::config::*;
use midomoji_clone::util::*;

use std::env;
use std::process;
use std::io::prelude::*;
use std::fs::File;
use std::io::BufReader;
use memmap::*;
//use std::io::{Error, ErrorKind};
//use std::io;

fn main() {
    let config: Config =  match Config::new(env::args()) {
        Ok(config) => config,
        Err(msg)   => {
            eprintln!("{}", msg);
            process::exit(1);
        },
    };
match config.mode {
        Some(Mode::Tokenize { dict }) => {
            println!("tokenize");
        },
        Some(Mode::Build { lex, matrix, output }) => {
            build(lex, matrix, output);
            println!("build complete");
        },
        Some(Mode::Test { lex, matrix, dict }) => {
            test(lex, matrix, dict);
            println!("test complete");
        },
        Some(Mode::Bench { lex }) => {
            bench(lex);
            println!("test complete");
        },
        None => {
            eprintln!("mode not found");
            process::exit(1);
        }
    }
}

fn bench(lex: String) {
    let mut timer = Timer::new();
    // 形態素辞書構築
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
    trie.to_double_array();
    println!("build double_array complete");
    timer.stop();
    timer.print();
}

fn test(lex: String, matrix: String, dict: String) {
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

fn build(lex: String, matrix: String, output: String) {
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
