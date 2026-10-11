@echo off
setlocal

echo ======================================================================
echo   Building Yarac UI
echo ======================================================================

set "PROJECT_ROOT=%~dp0"
cd /d "%PROJECT_ROOT%"

set "MODE=release"
set "BUILD_FRONTEND=0"

:parse_args
if "%~1"=="" goto after_args
if /i "%~1"=="--debug" (
    set "MODE=debug"
    shift
    goto parse_args
)
if /i "%~1"=="--release" (
    set "MODE=release"
    shift
    goto parse_args
)
if /i "%~1"=="--with-frontend" (
    set "BUILD_FRONTEND=1"
    shift
    goto parse_args
)
if /i "%~1"=="--help" goto show_help
if /i "%~1"=="-h" goto show_help

echo [WARNING] Unknown argument: %~1
shift
goto parse_args

:after_args

if "%BUILD_FRONTEND%"=="1" (
    echo.
    echo [*] Building web frontend assets...
    if not exist "frontend\node_modules" (
        echo [*] Installing frontend dependencies...
        pushd frontend
        call npm install
        if errorlevel 1 (
            popd
            echo [ERROR] npm install failed.
            exit /b 1
        )
        popd
    )

    pushd frontend
    call npm run build
    if errorlevel 1 (
        popd
        echo [ERROR] Frontend build failed.
        exit /b 1
    )
    popd
    echo [OK] Frontend build succeeded.
)

echo.
if "%MODE%"=="release" (
    echo [*] Compiling yarac-ui in release mode...
    cargo build --release -p yarac-ui
) else (
    echo [*] Compiling yarac-ui in debug mode...
    cargo build -p yarac-ui
)

if errorlevel 1 (
    echo.
    echo [ERROR] Cargo build for yarac-ui failed.
    exit /b %errorlevel%
)

echo.
echo ======================================================================
if "%MODE%"=="release" (
    echo [SUCCESS] yarac-ui built successfully: target\release\yarac-ui.exe
) else (
    echo [SUCCESS] yarac-ui built successfully: target\debug\yarac-ui.exe
)
echo ======================================================================

exit /b 0

:show_help
echo Usage: build.bat [options]
echo.
echo Options:
echo   --release          Build in release mode (default)
echo   --debug            Build in debug mode
echo   --with-frontend    Also compile frontend assets with npm run build
echo   --help, -h         Display this help message
exit /b 0
