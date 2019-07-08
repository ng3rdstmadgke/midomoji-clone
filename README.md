# midomoji-clone
midomojiのrust版

[![Build Status](https://travis-ci.org/ng3rdstmadgke/midomoji-clone.svg?branch=master)](https://travis-ci.org/ng3rdstmadgke/midomoji-clone)

# 使い方

```bash
# 現代書き言葉UniDic をダウンロード(https://unidic.ninjal.ac.jp/)
# 解凍
unzip unidic-csj-2.3.0.zip

# 辞書の構築(uni.dic)
cd midomoji-clone
cargo run --bin build-dict --release -- ../unidic-cwj-2.3.0/lex.csv ../unidic-cwj-2.3.0/matrix.def uni.dic

# 解析
echo -n "吾輩は猫である。" | cargo run --bin analyze --release -- uni.dic
```