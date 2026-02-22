@echo off
REM Setup MSVC environment for Rust compilation
SET "VSCMD_SKIP_SENDTELEMETRY=1"
SET "PATH=C:\Program Files\Microsoft Visual Studio\18\Insiders\VC\Tools\MSVC\14.29.30133\bin\Hostx64\x64;%PATH%"
SET "LIB=C:\Program Files\Microsoft Visual Studio\18\Insiders\VC\Tools\MSVC\14.29.30133\lib\x64;C:\Program Files (x86)\Windows Kits\10\Lib\10.0.26100.0\um\x64;C:\Program Files (x86)\Windows Kits\10\Lib\10.0.26100.0\ucrt\x64"
SET "INCLUDE=C:\Program Files\Microsoft Visual Studio\18\Insiders\VC\Tools\MSVC\14.29.30133\include;C:\Program Files (x86)\Windows Kits\10\Include\10.0.26100.0\ucrt;C:\Program Files (x86)\Windows Kits\10\Include\10.0.26100.0\um;C:\Program Files (x86)\Windows Kits\10\Include\10.0.26100.0\shared"

cd C:\Temp\md-scanner\src-tauri
cargo test windows_deployment_tests --lib
