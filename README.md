# midomoji-clone
midomojiのrust版

[![Build Status](https://travis-ci.org/ng3rdstmadgke/midomoji-clone.svg?branch=master)](https://travis-ci.org/ng3rdstmadgke/midomoji-clone)

# 使い方

```bash
# 現代書き言葉UniDic をダウンロード(https://clrd.ninjal.ac.jp/unidic/)
$ wget https://clrd.ninjal.ac.jp/unidic_archive/cwj/2.3.0/unidic-cwj-2.3.0.zip

# 解凍
$ unzip unidic-csj-2.3.0.zip

# クローン
$ git clone git@github.com:ng3rdstmadgke/midomoji-clone.git
$ cd midomoji-clone

# ビルド
$ cargo build --release

# 辞書の構築(uni.dic)
$ ./target/release/build-dict ../unidic-cwj-2.3.0/lex.csv ../unidic-cwj-2.3.0/matrix.def uni.dic

# 解析
$ echo -n "吾輩は猫である。" | ./target/release/analyze uni.dic

吾輩
は
猫
で
ある
。
```
