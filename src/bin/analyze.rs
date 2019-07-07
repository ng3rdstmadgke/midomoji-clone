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
use std::io::BufWriter;
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
    let mut reader: BufReader<Box<Read>> = if let Some(input) = options.get("input") {
        BufReader::new(Box::new(File::open(input).ok().unwrap()))
    } else {
        BufReader::new(Box::new(io::stdin()))
    };

    // writer
    let mut writer = BufWriter::new(io::stdout());

    analyze(dict_set, &mut reader, &mut writer);
}

fn analyze<R: Read, W: Write>(dict_set: DictionarySet<Token>, reader: &mut BufReader<R>, writer: &mut W) {
    let mut buf = String::new();
    while reader.read_line(&mut buf).unwrap() > 0 {
        let mut lattice = Lattice::build(&dict_set, buf.as_bytes());
        lattice.analyze(&dict_set);
        for node in lattice.get_result().iter().rev() {
            writer.write(node.surface).unwrap();
            writer.write(b"\n").unwrap();
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
                eprintln!("{}", include_str!("../resources/analyze.txt"));
                std::process::exit(1);
            } else if options.get("dict") == None {
                options.insert("dict".to_string(), arg);
            } else if arg == "-i" || arg == "--input" {
                key = Some("input".to_string());
            } else {
                eprintln!("不明なオプション: {}", arg);
                std::process::exit(1);
            }
        }
    }
    let required_opts = ["dict"];
    for k in required_opts.iter() { // k は std::borrow::Borrow<&str>
        if options.get(*k) == None {
            eprintln!("{}", include_str!("../resources/analyze.txt"));
            std::process::exit(1);
        }
    }
    options
}
