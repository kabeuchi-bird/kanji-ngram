# kanji_ngram

テキストコーパスから**漢字を含む文字 n-gram** を抽出し、出現回数・出現頻度とともに CSV に書き出すツールです。

---

## 動作環境

| OS | 備考 |
|---|---|
| Windows 10 / 11 | `run.bat` を使用 |
| Linux (x86_64) | `run.sh` を使用 |

コーパスファイルは **UTF-8** で保存されている必要があります。

---

## ファイル構成

```
kanji_ngram/
├── bin/
│   ├── kanji_ngram.exe   # Windows 用実行ファイル
│   └── kanji_ngram       # Linux 用実行ファイル
├── run.bat               # Windows 起動スクリプト
├── run.sh                # Linux 起動スクリプト
└── README.md             # このファイル
```

---

## 使い方

### Windows

`run.bat` をテキストエディタで開き、上部の設定欄を編集してダブルクリックで実行します。

```bat
set CORPUS=corpus.txt   ← コーパスファイルのパスを指定（必須）
set N=3                 ← n-gram のサイズ（省略時: 3）
set TOP_K=              ← 上位何件まで出力するか（省略時: 全件）
set ENCODING=utf-8-bom  ← 出力 CSV の文字コード（省略時: utf-8-bom）
```

### Linux

```bash
chmod +x run.sh         # 初回のみ実行権限を付与
./run.sh
```

`run.sh` をテキストエディタで開き、上部の設定欄を編集して実行します。

```bash
CORPUS="corpus.txt"     # コーパスファイルのパスを指定（必須）
N=3                     # n-gram のサイズ（省略時: 3）
TOP_K=""                # 上位何件まで出力するか（空欄で全件）
ENCODING="utf-8"        # 出力 CSV の文字コード
```

---

## コマンドライン直接実行

スクリプトを使わずに直接実行することもできます。

```
# 書式
kanji_ngram <コーパスファイル> [n] [top_k] [--encoding <enc>]

# 例: trigram（デフォルト）、全件出力
kanji_ngram corpus.txt

# 例: bigram、全件出力
kanji_ngram corpus.txt 2

# 例: trigram、上位 100 件のみ出力
kanji_ngram corpus.txt 3 100

# 例: trigram、UTF-8 BOM あり（Excel 向け）
kanji_ngram corpus.txt 3 --encoding utf-8-bom

# 例: trigram、Shift-JIS 出力
kanji_ngram corpus.txt 3 --encoding shift-jis
```

---

## 出力 CSV の文字コード

`--encoding` オプション（またはスクリプトの `ENCODING` 変数）で指定します。

| 値 | 説明 | 用途 |
|---|---|---|
| `utf-8` | UTF-8（BOM なし） | Linux / Mac、テキストエディタ全般 |
| `utf-8-bom` | UTF-8（BOM あり） | **Windows の Excel で直接開く場合（推奨）** |
| `shift-jis` | Shift-JIS（CP932） | 旧来の日本語 Windows 環境・レガシーシステム連携 |

> **Excel on Windows について**  
> Excel は BOM なしの UTF-8 CSV を開くと文字化けすることがあります。  
> `utf-8-bom` を使用すると Excel が自動的に UTF-8 と認識し、正しく表示されます。  
> `run.bat` のデフォルトは `utf-8-bom` に設定済みです。

---

## 出力

### ファイル名

| 条件 | ファイル名 |
|---|---|
| top_k 指定なし | `{コーパス名}_ngram{n}.csv` |
| top_k 指定あり | `{コーパス名}_ngram{n}_top{k}.csv` |

例: `corpus.txt` を n=3、top_k=100 で処理 → `corpus_ngram3_top100.csv`

出力 CSV はコーパスファイルと**同じディレクトリ**に作成されます。

### CSV フォーマット

```
n-gram,出現回数,出現頻度(%)
"日本語",142,3.2500
"語処理",98,2.2386
"処理を",75,1.7143
...
```

- 1 行目はヘッダ
- 出現回数の降順でソート（同数の場合は辞書順）
- 出現頻度の分母は「コーパス全体の n-gram 総出現数」
- n-gram フィールドはダブルクォートで囲まれます（RFC 4180 準拠）

### 処理対象

- 漢字を **1 文字以上含む** n-gram のみを抽出します
- ひらがな・カタカナ・記号のみで構成される n-gram は除外されます
- 改行・スペース・全角スペース・タブは除去してから処理します（行またぎあり）
- 対応する漢字の範囲: CJK 統合漢字・拡張 A〜H・互換漢字・互換漢字補助

### 単漢字の出現頻度を調べたい場合

`n=1` を指定することで、コーパス中の漢字1文字ずつの出現回数・頻度を集計できます。

```bash
kanji_ngram corpus.txt 1
```

n=1 のとき、ウィンドウは1文字になるため「漢字を含むか」と「漢字であるか」が一致し、漢字以外（かな・記号など）は自動的に除外されます。

---

## 実行例

```
前処理後の文字数: 204832

=== 完了 ===
n                   : 3
全 n-gram 総数        : 204830
漢字含む n-gram 種類数: 18472
漢字含む n-gram 総数  : 98341
出力件数            : 100
文字コード          : utf-8-bom
出力ファイル        : corpus_ngram3_top100.csv
```

進捗メッセージは標準エラー出力に表示されます（CSV には含まれません）。
