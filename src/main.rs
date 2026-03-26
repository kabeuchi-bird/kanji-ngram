use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::process;

// ── 漢字判定 ────────────────────────────────────────────────────────────────
// CJK 統合漢字・拡張 A–G・互換漢字・互換漢字補助をすべてカバー
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
    eprintln!("  <corpus_stem>_ngram<n>.csv");
    eprintln!("  <corpus_stem>_ngram<n>_top<k>.csv  （top_k 指定時）");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = &args[0];

    // ── 引数パース ──────────────────────────────────────────────────────────
    if args.len() < 2 || args[1] == "-h" || args[1] == "--help" {
        print_usage(program);
        process::exit(if args.len() < 2 { 1 } else { 0 });
    }

    let corpus_path = &args[1];

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
        eprintln!("Error: ファイルを読み込めません '{}': {}", corpus_path, e);
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
    // 漢字を含むものだけ HashMap に蓄積する
    let window_count = chars.len() - n + 1;
    let total_all_tokens = window_count as u64; // 全 n-gram 数（分母）
    let mut counts: HashMap<String, u64> = HashMap::with_capacity(window_count / 4);

    // key バッファを外に出してアロケーション回数を削減:
    //   - 新規キー → バッファを HashMap に move してそのまま所有させる
    //   - 既存キー → バッファを使い回す（clone 不要）
    let mut key_buf = String::with_capacity(n * 4); // UTF-8 最大 4 byte/char

    for i in 0..window_count {
        let window = &chars[i..i + n];
        if window.iter().any(|&c| is_kanji(c)) {
            // .copied() で &char → char に変換（&char は FromIterator<String> 対象外）
            key_buf.clear();
            key_buf.extend(window.iter().copied());

            // 既存キーならバッファを再利用、新規キーならバッファを move
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

    // ── ソート: 出現回数 降順、同数は辞書順 ──────────────────────────────
    let mut sorted: Vec<(String, u64)> = counts.into_iter().collect();
    sorted.sort_unstable_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

    let total_tokens: u64 = sorted.iter().map(|(_, c)| c).sum();
    let total_types = sorted.len();

    // ── 出力スライス ─────────────────────────────────────────────────────────
    let output_slice: &[(String, u64)] = match top_k {
        Some(k) => &sorted[..k.min(sorted.len())],
        None => &sorted,
    };

    // ── 出力ファイル名生成 ──────────────────────────────────────────────────
    let stem = Path::new(corpus_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("corpus");

    let output_filename = match top_k {
        Some(k) => format!("{}_ngram{}_top{}.csv", stem, n, k),
        None => format!("{}_ngram{}.csv", stem, n),
    };

    // ── CSV 書き出し ─────────────────────────────────────────────────────────
    let file = fs::File::create(&output_filename).unwrap_or_else(|e| {
        eprintln!("Error: 出力ファイルを作成できません '{}': {}", output_filename, e);
        process::exit(1);
    });
    let mut writer = BufWriter::new(file);

    writeln!(writer, "n-gram,出現回数,出現頻度(%)").unwrap();

    for (ngram, count) in output_slice {
        let freq = (*count as f64 / total_all_tokens as f64) * 100.0;
        // RFC 4180: フィールド内の " は "" にエスケープしてからクォートで囲む
        let escaped = ngram.replace('"', "\"\"");
        writeln!(writer, "\"{}\",{},{:.4}", escaped, count, freq).unwrap();
    }

    // ── 実行結果サマリー ────────────────────────────────────────────────────
    eprintln!();
    eprintln!("=== 完了 ===");
    eprintln!("n                   : {}", n);
    eprintln!("全 n-gram 総数        : {}", total_all_tokens);
    eprintln!("漢字含む n-gram 種類数: {}", total_types);
    eprintln!("漢字含む n-gram 総数  : {}", total_tokens);
    eprintln!("出力件数            : {}", output_slice.len());
    eprintln!("出力ファイル        : {}", output_filename);
}
