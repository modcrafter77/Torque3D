@echo off
setlocal
set "TOOL=%~dp0..\..\tools\dif2t3d\target\release\dif2t3d.exe"
set "SRC=%~1"
set "OUT=%~2"
if "%SRC%"=="" (
  echo Usage: convert_dif.bat input.dif [output_dir]
  exit /b 1
)
if not exist "%TOOL%" (
  echo Building dif2t3d...
  pushd "%~dp0..\..\tools\dif2t3d"
  cargo build --release || exit /b 1
  popd
)
if "%OUT%"=="" set "OUT=%~dp1%~n1"
"%TOOL%" "%SRC%" -o "%OUT%"
exit /b %ERRORLEVEL%
