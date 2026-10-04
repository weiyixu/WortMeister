@echo off
REM build_and_serve.bat - build the WASM and serve the PWA locally for testing.
REM Run this from anywhere; it uses paths relative to this script.
REM Requires: Rust (rustup), wasm-pack, Python 3.

setlocal
set HERE=%~dp0

echo === Building WASM package ===
wasm-pack build "%HERE%wasm" --target web --out-dir "%HERE%public\pkg" --release
if errorlevel 1 (
  echo WASM build failed.
  exit /b 1
)

echo === Copying vocabulary.csv ===
copy /Y "%HERE%..\data\vocabulary.csv" "%HERE%public\vocabulary.csv" >nul

echo === Serving at http://localhost:8777  (Ctrl+C to stop) ===
cd /d "%HERE%public"
python -m http.server 8777

endlocal
