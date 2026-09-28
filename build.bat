@echo off
rem Z-Accounts one-shot build script
setlocal
cd /d "%~dp0"

where node >nul 2>nul || (echo [!] Node.js is required & exit /b 1)
where cargo >nul 2>nul || set "PATH=%USERPROFILE%\.cargo\bin;%PATH%"
where cargo >nul 2>nul || (echo [!] Rust (rustup, stable-msvc) is required & exit /b 1)

echo [1/2] Building frontend (Vite)...
call npm install --no-audit --no-fund || exit /b 1
call npm run build || exit /b 1

echo [2/2] Building backend (cargo release)...
pushd src-tauri
cargo build --release || exit /b 1
popd

echo.
echo Done: src-tauri\target\release\Z-Accounts.exe
endlocal
