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

    if let Some(sub_command) = options.get("sub_command") {
        if sub_command == "build" {
            build(dict_set, &mut reader);
        } else if sub_command  == "analyze" {
            analyze(dict_set, &mut reader);
        } else if sub_command  == "search" {
            search(dict_set, &mut reader);
        } else if sub_command  == "prefix-search" {
            prefix_search(dict_set, &mut reader);
        } else {
            eprintln!("不明なサブコマンド: {}", sub_command);
            std::process::exit(1);
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
    let _script = args.next().unwrap();
    let mut key: Option<String> = None;
    for arg in args {
        if let Some(k) = key {
            options.insert(k.clone(), arg.to_string());
            key = None;
        } else {
            if arg == "-h" || arg == "--help" {
                eprintln!("{}", include_str!("../resources/debug.txt"));
                std::process::exit(1);
            } else if options.get("dict") == None {
                options.insert("dict".to_string(), arg);
            } else if options.get("sub_command") == None {
                options.insert("sub_command".to_string(), arg);
            } else {
                eprintln!("不明なオプション: {}", arg);
                std::process::exit(1);
            }
        }
    }
    let required_opts = ["dict", "sub_command"];
    for k in required_opts.iter() { // k は std::borrow::Borrow<&str>
        if options.get(*k) == None {
                eprintln!("{}", include_str!("../resources/debug.txt"));
                std::process::exit(1);
        }
    }
    options
}
