#!/usr/bin/env bash
set -euo pipefail

# ============================================================
#  クロスビルドスクリプト
#  Linux (x86_64) と Windows (x86_64) の両バイナリを bin/ に出力
# ============================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "${SCRIPT_DIR}"

BIN_DIR="${SCRIPT_DIR}/bin"
mkdir -p "${BIN_DIR}"

LINUX_TARGET="x86_64-unknown-linux-gnu"
WINDOWS_TARGET="x86_64-pc-windows-gnu"

echo "=== Linux (${LINUX_TARGET}) ==="
cargo build --release --target "${LINUX_TARGET}"
cp "target/${LINUX_TARGET}/release/kanji_ngram" "${BIN_DIR}/kanji_ngram"
chmod +x "${BIN_DIR}/kanji_ngram"
echo "  -> ${BIN_DIR}/kanji_ngram"

echo ""
echo "=== Windows (${WINDOWS_TARGET}) ==="
cargo build --release --target "${WINDOWS_TARGET}"
cp "target/${WINDOWS_TARGET}/release/kanji_ngram.exe" "${BIN_DIR}/kanji_ngram.exe"
echo "  -> ${BIN_DIR}/kanji_ngram.exe"

echo ""
echo "=== 完了 ==="
ls -lh "${BIN_DIR}"/kanji_ngram*
