@echo off
chcp 65001 > nul
setlocal

rem ============================================================
rem  設定欄 --- ここを編集してください
rem ============================================================

rem コーパスファイルのパス（必須）
rem  例: set CORPUS=C:\Users\user\Documents\corpus.txt
rem  例: set CORPUS=corpus.txt  （このバッチと同じフォルダにある場合）
set CORPUS=corpus.txt

rem n-gram のサイズ（省略時は 3）
set N=3

rem 上位何件まで出力するか（空欄にすると全件出力）
set TOP_K=

rem 出力 CSV の文字コード
rem   utf-8       UTF-8（BOM なし）
rem   utf-8-bom   UTF-8（BOM あり、Excel 推奨）
rem   shift-jis   Shift-JIS（CP932）
set ENCODING=utf-8-bom

rem ============================================================
rem  以下は変更不要です
rem ============================================================

set BIN=%~dp0bin\kanji_ngram.exe

if not exist "%BIN%" (
    echo [エラー] 実行ファイルが見つかりません: %BIN%
    pause
    exit /b 1
)

if "%CORPUS%"=="" (
    echo [エラー] CORPUS が設定されていません。このファイルをテキストエディタで開いて設定してください。
    pause
    exit /b 1
)

if not exist "%CORPUS%" (
    echo [エラー] コーパスファイルが見つかりません: %CORPUS%
    pause
    exit /b 1
)

rem 引数を組み立てる
set ARGS="%CORPUS%" %N%
if not "%TOP_K%"=="" set ARGS=%ARGS% %TOP_K%
if not "%ENCODING%"=="" set ARGS=%ARGS% --encoding %ENCODING%

echo 実行中: kanji_ngram %ARGS%
echo.

"%BIN%" %ARGS%

if %errorlevel% neq 0 (
    echo.
    echo [エラー] 処理が失敗しました（終了コード: %errorlevel%）
    pause
    exit /b %errorlevel%
)

echo.
echo 処理が完了しました。
pause
