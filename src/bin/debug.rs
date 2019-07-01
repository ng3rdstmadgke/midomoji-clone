use midomoji_clone::dictionary::*;
use midomoji_clone::token::Token;
use midomoji_clone::lattice::Lattice;

use std::env;
use std::env::Args;
use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::BufReader;
//use std::io::BufWriter;
use memmap::*;

fn main() {
    let options = parse_args(env::args());
    // 辞書構築
    let dict_file: File = File::open(options.get("dict").unwrap()).unwrap();
    let mmap: Mmap = unsafe {
        MmapOptions::new().map(&dict_file).ok().unwrap()
    };
    let dict_set: DictionarySet<Token> = DictionarySet::new(&mmap);

    // reader
    let mut reader: BufReader<Box<Read>> = BufReader::new(Box::new(io::stdin()));

    // writer
    //let mut writer = BufWriter::new(io::stdout());

    if let Some(subcommand) = options.get("subcommand") {
        if subcommand == "build" {
            build(dict_set, &mut reader);
        } else if subcommand  == "analyze" {
            analyze(dict_set, &mut reader);
        } else if subcommand  == "search" {
            search(dict_set, &mut reader);
        } else if subcommand  == "prefix_search" {
            prefix_search(dict_set, &mut reader);
        }

    }
}

fn build<R: Read>(dict_set: DictionarySet<Token>, reader: &mut BufReader<R>) {
    let mut buf = String::new();
    while reader.read_line(&mut buf).unwrap() > 0 {
        let lattice = Lattice::build(&dict_set, buf.trim().as_bytes());
        lattice.debug();
        buf.clear();
    }
}


fn analyze<R: Read>(dict_set: DictionarySet<Token>, reader: &mut BufReader<R>) {
    let mut buf = String::new();
    while reader.read_line(&mut buf).unwrap() > 0 {
        let mut lattice = Lattice::build(&dict_set, buf.trim().as_bytes());
        lattice.analyze(&dict_set);
        lattice.debug();
        println!("=== === === result === === ===");
        for node in lattice.get_result().iter().rev() {
            println!("{}", node);
        }
        buf.clear();
    }
}

fn search<R: Read>(dict_set: DictionarySet<Token>, reader: &mut BufReader<R>) {
    let mut buf = String::new();
    while reader.read_line(&mut buf).unwrap() > 0 {
        let line = buf.trim();
        println!("{}", line);
        if let Some(tokens) = dict_set.get_trie(line) {
            for (i, token) in tokens.iter().enumerate() {
                println!("|    {}: {:?}", i, token);
            }
        }
        buf.clear();

    }
}

fn prefix_search<R: Read>(dict_set: DictionarySet<Token>, reader: &mut BufReader<R>) {
    let mut buf = String::new();
    while reader.read_line(&mut buf).unwrap() > 0 {
        let line = buf.trim();
        for (surface, tokens) in dict_set.prefix_search(line) {
            println!("{}", surface);
            for (i, token) in tokens.iter().enumerate() {
                println!("|    {}: {:?}", i, token);
            }

        }
        buf.clear();

    }
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
            if options.get("dict") == None {
                options.insert("dict".to_string(), arg);
            } else if options.get("subcommand") == None {
                options.insert("subcommand".to_string(), arg);
            } else {
                panic!("不明なオプション: {}", arg);
            }
        }
    }
    let required_opts = ["dict", "subcommand"];
    for k in required_opts.iter() { // k は std::borrow::Borrow<&str>
        if options.get(*k) == None {
            panic!("usage: {} <DICT_PATH> <build|analyze|search|prefix_search>", script);
        }
    }
    options
}
