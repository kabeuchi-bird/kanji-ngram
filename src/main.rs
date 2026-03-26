use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::BufWriter;
use std::path::Path;
use std::process;

// ── 漢字判定 ────────────────────────────────────────────────────────────────
// CJK 統合漢字・拡張 A–H・互換漢字・互換漢字補助をすべてカバー
fn is_kanji(c: char) -> bool {
    matches!(c,
        '\u{3400}'..='\u{4DBF}'   // CJK 拡張 A
        | '\u{4E00}'..='\u{9FFF}'  // CJK 統合漢字
        | '\u{F900}'..='\u{FAFF}'  // CJK 互換漢字
        | '\u{20000}'..='\u{2A6DF}'// CJK 拡張 B
        | '\u{2A700}'..='\u{2B73F}'// CJK 拡張 C
        | '\u{2B740}'..='\u{2B81F}'// CJK 拡張 D
        | '\u{2B820}'..='\u{2CEAF}'// CJK 拡張 E
        | '\u{2CEB0}'..='\u{2EBEF}'// CJK 拡張 F
        | '\u{2F800}'..='\u{2FA1F}'// CJK 互換漢字補助
        | '\u{30000}'..='\u{3134F}'// CJK 拡張 G
        | '\u{31350}'..='\u{323AF}'// CJK 拡張 H（Unicode 15.0）
    )
}

// ── 除去する文字（改行・空白）────────────────────────────────────────────────
fn is_removable(c: char) -> bool {
    c.is_whitespace() // ASCII スペース・タブ・改行 + 全角スペース等を一括処理
}

// ── n-gram カウント（ロジック本体） ───────────────────────────────────────────
fn count_ngrams(chars: &[char], n: usize) -> HashMap<String, u64> {
    let window_count = chars.len().saturating_sub(n - 1);
    let mut counts: HashMap<String, u64> = HashMap::with_capacity(window_count / 4 + 1);

    // key バッファを外に出してアロケーション回数を削減
    let mut key_buf = String::with_capacity(n * 4);

    for i in 0..window_count {
        let window = &chars[i..i + n];
        if window.iter().any(|&c| is_kanji(c)) {
            key_buf.clear();
            key_buf.extend(window.iter().copied());

            match counts.get_mut(key_buf.as_str()) {
                Some(count) => *count += 1,
                None => {
                    let owned = std::mem::replace(
                        &mut key_buf,
                        String::with_capacity(n * 4),
                    );
                    counts.insert(owned, 1);
                }
            }
        }
    }

    counts
}

// ── ヘルプ表示 ────────────────────────────────────────────────────────────────
fn print_usage(program: &str) {
    eprintln!("Usage: {} <corpus_file> [n] [top_k]", program);
    eprintln!();
    eprintln!("Arguments:");
    eprintln!("  corpus_file   テキストコーパスファイル（必須、UTF-8）");
    eprintln!("  n             n-gram のサイズ（省略時: 3）");
    eprintln!("  top_k         出力する上位件数（省略時: 全件）");
    eprintln!();
    eprintln!("Output:");
    eprintln!("  <corpus_dir>/<corpus_stem>_ngram<n>.csv");
    eprintln!("  <corpus_dir>/<corpus_stem>_ngram<n>_top<k>.csv  （top_k 指定時）");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = &args[0];

    // ── 引数パース ──────────────────────────────────────────────────────────
    if args.len() < 2 || args[1] == "-h" || args[1] == "--help" {
        print_usage(program);
        process::exit(if args.len() < 2 { 1 } else { 0 });
    }

    let corpus_path = Path::new(&args[1]);

    let n: usize = match args.get(2) {
        Some(s) => s.parse::<usize>().unwrap_or_else(|_| {
            eprintln!("Error: n は正の整数で指定してください: {}", s);
            process::exit(1);
        }),
        None => 3,
    };
    if n == 0 {
        eprintln!("Error: n は 1 以上にしてください");
        process::exit(1);
    }

    let top_k: Option<usize> = match args.get(3) {
        Some(s) => Some(s.parse::<usize>().unwrap_or_else(|_| {
            eprintln!("Error: top_k は正の整数で指定してください: {}", s);
            process::exit(1);
        })),
        None => None,
    };

    // ── ファイル読み込み ────────────────────────────────────────────────────
    let raw = fs::read_to_string(corpus_path).unwrap_or_else(|e| {
        eprintln!("Error: ファイルを読み込めません '{}': {}", corpus_path.display(), e);
        process::exit(1);
    });

    // ── 前処理: 改行・空白を除去して一本の文字列に ────────────────────────
    let chars: Vec<char> = raw.chars().filter(|c| !is_removable(*c)).collect();

    eprintln!("前処理後の文字数: {}", chars.len());

    if chars.len() < n {
        eprintln!("Error: コーパスの文字数 ({}) が n ({}) より少ないです", chars.len(), n);
        process::exit(1);
    }

    // ── n-gram カウント ─────────────────────────────────────────────────────
    let counts = count_ngrams(&chars, n);

    // ── ソート: 出現回数 降順、同数は辞書順 ──────────────────────────────
    let mut sorted: Vec<(String, u64)> = counts.into_iter().collect();
    sorted.sort_unstable_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

    let total_tokens: u64 = sorted.iter().map(|(_, c)| *c).sum();
    let total_types = sorted.len();

    // ── 出力スライス ─────────────────────────────────────────────────────────
    let output_slice: &[(String, u64)] = match top_k {
        Some(k) => &sorted[..k.min(sorted.len())],
        None => &sorted,
    };

    // ── 出力ファイル名生成（コーパスと同じディレクトリに出力）──────────────
    let stem = corpus_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("corpus");

    let filename = match top_k {
        Some(k) => format!("{}_ngram{}_top{}.csv", stem, n, k),
        None => format!("{}_ngram{}.csv", stem, n),
    };

    let output_path = corpus_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(&filename);

    // ── CSV 書き出し（csv クレートで RFC 4180 準拠）──────────────────────────
    let file = fs::File::create(&output_path).unwrap_or_else(|e| {
        eprintln!("Error: 出力ファイルを作成できません '{}': {}", output_path.display(), e);
        process::exit(1);
    });
    let buf_writer = BufWriter::new(file);
    let mut csv_wtr = csv::Writer::from_writer(buf_writer);

    csv_wtr.write_record(["n-gram", "出現回数", "出現頻度(%)"]).unwrap();

    for (ngram, count) in output_slice {
        let freq = if total_tokens > 0 {
            (*count as f64 / total_tokens as f64) * 100.0
        } else {
            0.0
        };
        csv_wtr.write_record([ngram.as_str(), &count.to_string(), &format!("{:.4}", freq)]).unwrap();
    }
    csv_wtr.flush().unwrap();

    // ── 実行結果サマリー ────────────────────────────────────────────────────
    let window_count = chars.len() - n + 1;
    eprintln!();
    eprintln!("=== 完了 ===");
    eprintln!("n                   : {}", n);
    eprintln!("全 n-gram 総数        : {}", window_count);
    eprintln!("漢字含む n-gram 種類数: {}", total_types);
    eprintln!("漢字含む n-gram 総数  : {}", total_tokens);
    eprintln!("出力件数            : {}", output_slice.len());
    eprintln!("出力ファイル        : {}", output_path.display());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_kanji_basic() {
        assert!(is_kanji('漢'));
        assert!(is_kanji('字'));
        assert!(is_kanji('亜')); // U+4E9C
        assert!(is_kanji('鑛')); // U+945B
    }

    #[test]
    fn test_is_kanji_non_kanji() {
        assert!(!is_kanji('a'));
        assert!(!is_kanji('あ'));
        assert!(!is_kanji('ア'));
        assert!(!is_kanji('1'));
        assert!(!is_kanji('！'));
        assert!(!is_kanji(' '));
    }

    #[test]
    fn test_is_kanji_extension_a() {
        assert!(is_kanji('\u{3400}')); // CJK 拡張A 先頭
        assert!(is_kanji('\u{4DBF}')); // CJK 拡張A 末尾
    }

    #[test]
    fn test_is_kanji_compatibility() {
        assert!(is_kanji('\u{F900}')); // CJK 互換漢字 先頭
    }

    #[test]
    fn test_is_removable() {
        assert!(is_removable(' '));
        assert!(is_removable('\t'));
        assert!(is_removable('\n'));
        assert!(is_removable('\u{3000}')); // 全角スペース
        assert!(!is_removable('漢'));
        assert!(!is_removable('a'));
    }

    #[test]
    fn test_count_bigrams() {
        let chars: Vec<char> = "漢字テスト".chars().collect();
        let counts = count_ngrams(&chars, 2);
        assert_eq!(counts.get("漢字"), Some(&1));
        assert_eq!(counts.get("字テ"), Some(&1)); // 漢字「字」を含む
        assert_eq!(counts.get("テス"), None);      // 漢字を含まない
        assert_eq!(counts.get("スト"), None);      // 漢字を含まない
    }

    #[test]
    fn test_count_trigrams() {
        let chars: Vec<char> = "私は漢字を学ぶ".chars().collect();
        let counts = count_ngrams(&chars, 3);
        assert_eq!(counts.get("私は漢"), Some(&1));
        assert_eq!(counts.get("は漢字"), Some(&1));
        assert_eq!(counts.get("漢字を"), Some(&1));
        assert_eq!(counts.get("字を学"), Some(&1)); // 字, 学 は漢字
        assert_eq!(counts.get("を学ぶ"), Some(&1)); // 学 は漢字
    }

    #[test]
    fn test_no_kanji_returns_empty() {
        let chars: Vec<char> = "あいうえお".chars().collect();
        let counts = count_ngrams(&chars, 2);
        assert!(counts.is_empty());
    }

    #[test]
    fn test_accumulate_counts() {
        let chars: Vec<char> = "漢字漢字".chars().collect();
        let counts = count_ngrams(&chars, 2);
        assert_eq!(counts.get("漢字"), Some(&2));
        assert_eq!(counts.get("字漢"), Some(&1));
    }

    #[test]
    fn test_short_input_returns_empty() {
        let chars: Vec<char> = "漢".chars().collect();
        let counts = count_ngrams(&chars, 2);
        assert!(counts.is_empty());
    }

    #[test]
    fn test_empty_input() {
        let chars: Vec<char> = Vec::new();
        let counts = count_ngrams(&chars, 2);
        assert!(counts.is_empty());
    }

    #[test]
    fn test_unigram() {
        let chars: Vec<char> = "あ漢い字う".chars().collect();
        let counts = count_ngrams(&chars, 1);
        assert_eq!(counts.len(), 2);
        assert_eq!(counts.get("漢"), Some(&1));
        assert_eq!(counts.get("字"), Some(&1));
    }

    #[test]
    fn test_whitespace_not_counted() {
        // 前処理でスペースが除去される想定だが、count_ngrams 自体は
        // 渡された文字列をそのまま処理する
        let raw = "漢 字";
        let chars: Vec<char> = raw.chars().filter(|c| !is_removable(*c)).collect();
        let counts = count_ngrams(&chars, 2);
        assert_eq!(counts.get("漢字"), Some(&1));
    }

    #[test]
    fn test_frequency_calculation() {
        // "漢字漢字" → bigram: 漢字(2), 字漢(1) → total=3
        let chars: Vec<char> = "漢字漢字".chars().collect();
        let counts = count_ngrams(&chars, 2);
        let total: u64 = counts.values().sum();
        assert_eq!(total, 3);

        let freq = *counts.get("漢字").unwrap() as f64 / total as f64 * 100.0;
        assert!((freq - 66.6667).abs() < 0.001);
    }

    #[test]
    fn test_csv_output_format() {
        use std::io::Cursor;

        let entries = vec![
            ("漢字".to_string(), 10u64),
            ("字を".to_string(), 5u64),
        ];
        let total: u64 = entries.iter().map(|(_, c)| *c).sum();

        let mut buf = Cursor::new(Vec::new());
        {
            let mut csv_wtr = csv::Writer::from_writer(&mut buf);
            csv_wtr.write_record(["n-gram", "出現回数", "出現頻度(%)"]).unwrap();
            for (ngram, count) in &entries {
                let freq = (*count as f64 / total as f64) * 100.0;
                csv_wtr.write_record([ngram.as_str(), &count.to_string(), &format!("{:.4}", freq)]).unwrap();
            }
            csv_wtr.flush().unwrap();
        }

        let output = String::from_utf8(buf.into_inner()).unwrap();
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines[0], "n-gram,出現回数,出現頻度(%)");
        assert_eq!(lines[1], "漢字,10,66.6667");
        assert_eq!(lines[2], "字を,5,33.3333");
    }

    #[test]
    fn test_end_to_end() {
        let input = "日本語の自然言語処理は面白い。日本語処理を学ぶ。";
        let chars: Vec<char> = input.chars().filter(|c| !is_removable(*c)).collect();
        let counts = count_ngrams(&chars, 2);

        // 「日本」は2回出現するはず
        assert_eq!(counts.get("日本"), Some(&2));
        // 「本語」も2回
        assert_eq!(counts.get("本語"), Some(&2));
        // 漢字を含まない bigram は含まれない
        assert!(counts.get("は面").is_none() || counts.get("は面").is_some());
        // 「は面」は漢字「面」を含むので抽出される
        assert_eq!(counts.get("は面"), Some(&1));
    }
}
