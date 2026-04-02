#!/usr/bin/env bash
set -euo pipefail

# ============================================================
#  設定欄 --- ここを編集してください
# ============================================================

# コーパスファイルのパス（必須）
# 例: CORPUS="/home/user/Documents/corpus.txt"
# 例: CORPUS="corpus.txt"  （このスクリプトと同じディレクトリにある場合）
CORPUS="corpus.txt"

# n-gram のサイズ（省略時は 3）
N=3

# 上位何件まで出力するか（空欄にすると全件出力）
TOP_K=""

# 出力 CSV の文字コード
#   utf-8       UTF-8（BOM なし）
#   utf-8-bom   UTF-8（BOM あり、Excel 推奨）
#   shift-jis   Shift-JIS（CP932）
ENCODING="utf-8"

# ============================================================
#  以下は変更不要です
# ============================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIN="${SCRIPT_DIR}/bin/kanji_ngram"

if [[ ! -f "${BIN}" ]]; then
    echo "[エラー] 実行ファイルが見つかりません: ${BIN}" >&2
    exit 1
fi

if [[ ! -x "${BIN}" ]]; then
    echo "[情報] 実行権限を付与します: ${BIN}"
    chmod +x "${BIN}"
fi

if [[ -z "${CORPUS}" ]]; then
    echo "[エラー] CORPUS が設定されていません。このスクリプトをテキストエディタで開いて設定してください。" >&2
    exit 1
fi

if [[ ! -f "${CORPUS}" ]]; then
    echo "[エラー] コーパスファイルが見つかりません: ${CORPUS}" >&2
    exit 1
fi

# 引数を組み立てる
ARGS=("${CORPUS}" "${N}")
if [[ -n "${TOP_K}" ]]; then
    ARGS+=("${TOP_K}")
fi
if [[ -n "${ENCODING}" ]]; then
    ARGS+=("--encoding" "${ENCODING}")
fi

echo "実行中: kanji_ngram ${ARGS[*]}"
echo ""

"${BIN}" "${ARGS[@]}"
