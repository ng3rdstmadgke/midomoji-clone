extern crate csv;

use midomoji_clone::token::Token;
use midomoji_clone::dictionary::*;
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
        None => {
            eprintln!("mode not found");
            process::exit(1);
        }
    }
}

fn test(lex: String, matrix: String, dict: String) {
    // 辞書読み込み
    let file: File = File::open(&dict).ok().unwrap();
    let mmap: Mmap = unsafe {
        MmapOptions::new().map(&file).ok().unwrap()
    };
    let dict_set: DictionarySet<Token> = DictionarySet::new(&mmap);
    println!("load dictionary complete");

    // matrix test
    let time = Timer::start();
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
    println!("{}", time.end());

    // trie test
    let time = Timer::start();
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
    println!("{}", time.end());
}

// Err(Error::new(ErrorKind::InvalidData, "invalid format. left_max, right_max not found."));
fn build(lex: String, matrix: String, output: String) {
    let mut builder: DictionaryBuilder<Token> = DictionaryBuilder::new(0, 0);
    
    // header読み込み
    let time = Timer::start();
    {
        let matrix_reader: BufReader<File> = BufReader::new(File::open(&matrix).ok().unwrap());
        for result in matrix_reader.lines() {
            let line: String = result.ok().unwrap();
            let mut record = line.trim().split_whitespace();
            let left_max: usize  = record.next().unwrap().parse::<usize>().ok().unwrap();
            let right_max: usize = record.next().unwrap().parse::<usize>().ok().unwrap();
            builder = DictionaryBuilder::new(left_max, right_max);
            break;
        }
    }
    println!("build header complete");
    println!("{}", time.end());

    // matrix構築
    let time = Timer::start();
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
            builder.set_matrix(left_id, right_id, cost);
        }
    }
    println!("build matrix complete");
    println!("{}", time.end());

    // 形態素辞書構築
    let time = Timer::start();
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
            builder.set_trie(lex, token);
        }
    }
    println!("build trie complete");
    println!("{}", time.end());

    // 辞書の書き込み
    let time = Timer::start();
    builder.serialize(&output).ok().unwrap();
    println!("build dictionary complete");
    println!("{}", time.end());
}
