@echo off
setlocal
where cargo >nul 2>nul
if errorlevel 1 (
  echo Rust was not found. Install Rust from https://rustup.rs and reopen this folder.
  pause
  exit /b 1
)
cargo build --release
if errorlevel 1 pause & exit /b 1
if not exist dist mkdir dist
copy /Y target\release\deutsch-worttrainer.exe dist\DeutschWorttrainer.exe >nul
copy /Y data\vocabulary.csv dist\vocabulary.csv >nul
echo.
echo Build complete: dist\DeutschWorttrainer.exe
pause
