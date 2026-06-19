@echo off
setlocal enabledelayedexpansion

set "TORQUE_ROOT=%~dp0..\.."
set "PROJECT_DIR=%TORQUE_ROOT%\My Projects\RefuzeGame"
set "BUILD_DIR=%TORQUE_ROOT%\build\RefuzeGame"
set "OVERLAY=%~dp0overlay"

echo Torque3D root: %TORQUE_ROOT%
echo Project dir:  %PROJECT_DIR%
echo Build dir:    %BUILD_DIR%

if not exist "%BUILD_DIR%" mkdir "%BUILD_DIR%"

cmake -S "%TORQUE_ROOT%" -B "%BUILD_DIR%" ^
  -G "Visual Studio 17 2022" -A x64 ^
  -DTORQUE_APP_NAME=RefuzeGame ^
  -DTORQUE_TEMPLATE=BaseGame ^
  -DTORQUE_PHYSICS_BULLET=ON ^
  -DTORQUE_NAVIGATION=ON

if errorlevel 1 (
  echo CMake configure failed.
  exit /b 1
)

call "%~dp0setupOverlay.bat"
if errorlevel 1 exit /b 1

echo.
echo Configure complete. Build with:
echo   cmake --build "%BUILD_DIR%" --config Debug --target install
echo.
echo Run from:
echo   "%PROJECT_DIR%\game\RefuzeGame.exe"
exit /b 0
