/// midomoji-cloneの動作モード
///
/// - Build: 辞書構築モード
///   - lex: 形態素辞書ファイル
///   - matrix: 連接コストファイル
///   - output: 出力ファイル
/// - Tokenize: 解析モード
#[derive(Debug)]
pub enum Mode {
    Tokenize {
        dict: String,
    },
    Build {
        lex   : String,
        matrix: String,
        output: String,
    },
    Test {
        lex   : String,
        matrix: String,
        dict  : String,
    },
    Bench {
        lex   : String,
    },
}

pub struct Config {
    pub mode: Option<Mode>,
}

/// コマンドライン引数の構造体
///
/// --build <LEX_PATH> <MATRIX_PATH> <OUTPUT_PATH>
/// --tokenize
impl Config {
    pub fn new(mut args: std::env::Args) -> Result<Config, &'static str> {
        let config = Config {
            mode: None,
        };
        args.next();
        Config::parse(args, config)
    }

    fn parse (mut args: std::env::Args, mut config: Config) -> Result<Config, &'static str> {
        match args.next() {
            Some(ref arg)  => {
                let arg_str = arg.as_str();
                if arg_str == "--tokenize" {
                    let dict    = args.next().ok_or("<LEX_PATH> not found")?;
                    config.mode = Some(Mode::Tokenize { dict });
                    Self::parse(args, config)
                } else if arg_str == "--build" {
                    let lex    = args.next().ok_or("<LEX_PATH> not found")?;
                    let matrix = args.next().ok_or("<MATRIX_PATH> not found")?;
                    let output = args.next().ok_or("<OUTPUT_PATH> not found")?;
                    config.mode = Some(Mode::Build { lex, matrix, output });
                    Self::parse(args, config)
                } else if arg_str == "--test" {
                    let lex    = args.next().ok_or("<LEX_PATH> not found")?;
                    let matrix = args.next().ok_or("<MATRIX_PATH> not found")?;
                    let dict   = args.next().ok_or("<DICT_PATH> not found")?;
                    config.mode = Some(Mode::Test { lex, matrix, dict });
                    Self::parse(args, config)
                } else if arg_str == "--bench" {
                    let lex = args.next().ok_or("<LEX_PATH> not found")?;
                    config.mode = Some(Mode::Bench { lex });
                    Self::parse(args, config)
                } else {
                    Err("invalid args")
                }
            },
            None => Ok(config)
        }
    }
}
