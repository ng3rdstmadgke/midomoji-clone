extern crate csv;

use midomoji_clone::dictionary::trie::Trie;
use midomoji_clone::token::Token;
use midomoji_clone::util::*;

use std::env;
use std::env::Args;
use std::fs::File;
use std::collections::HashMap;

fn main() {
    let options = parse_args(env::args());
    let sub_command = options.get("sub_command").unwrap();
    let lex         = options.get("lex").unwrap();
    if sub_command == "double_array" {
        build_double_array(lex);
    }
}

fn parse_args(mut args: Args) -> HashMap<String, String> {
    let mut options = HashMap::new();
    let script = args.next().unwrap();
    let sub_command = args.next().expect(&format!("usage: {} <SUB_COMMAND> <ARGS...>", script));
    if sub_command == "double_array" {
        parse_args_double_array(&script, args, &mut options);
    } else {
        panic!("不明なサブコマンド: {}", sub_command);
    }
    options
}

fn parse_args_double_array(script: &str, args: Args, options: &mut HashMap<String, String>) {
    options.insert("sub_command".to_string(), "double_array".to_string());
    let mut key: Option<String> = None;
    for arg in args {
        if let Some(k) = key {
            options.insert(k.clone(), arg.to_string());
            key = None;
        } else {
            if options.get("lex") == None {
                options.insert("lex".to_string(), arg);
            } else {
                panic!("不明なオプション: {}", arg);
            }
        }
    }
    let required_opts = ["lex"];
    for k in required_opts.iter() { // k は std::borrow::Borrow<&str>
        if options.get(*k) == None {
            panic!("usage: {} double_array <LEX_PATH> [options]", script);
        }
    }
}


fn build_double_array(lex: &str) {
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
