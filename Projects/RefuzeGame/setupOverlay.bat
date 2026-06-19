@echo off
setlocal

set "TORQUE_ROOT=%~dp0..\.."
set "PROJECT_DIR=%TORQUE_ROOT%\My Projects\RefuzeGame"
set "OVERLAY=%~dp0overlay"
set "GAME_DIR=%PROJECT_DIR%\game"
set "DATA_DUMP=C:\projects\gamedev\Refuze\refuze\data_dump"

if not exist "%PROJECT_DIR%" (
  echo Project directory not found: %PROJECT_DIR%
  echo Run configure.bat first.
  exit /b 1
)

echo Copying RefuzeGame overlay into %PROJECT_DIR% ...
xcopy /E /I /Y "%OVERLAY%\game\data\RefuzeGame" "%PROJECT_DIR%\game\data\RefuzeGame\" >nul
xcopy /E /I /Y "%OVERLAY%\source" "%PROJECT_DIR%\source\" >nul
if exist "%OVERLAY%\tools" xcopy /E /I /Y "%OVERLAY%\tools" "%PROJECT_DIR%\tools\" >nul

if not exist "%DATA_DUMP%\common" (
  echo WARNING: data_dump not found at %DATA_DUMP%
  goto :junctions_done
)

call :link_dir "%GAME_DIR%\common" "%DATA_DUMP%\common"
call :link_dir "%GAME_DIR%\starter.game" "%DATA_DUMP%\starter.game"

:junctions_done
echo Overlay installed.
exit /b 0

:link_dir
set "LINK=%~1"
set "TARGET=%~2"
if exist "%LINK%" (
  echo Link exists: %LINK%
  goto :eof
)
mklink /J "%LINK%" "%TARGET%"
if errorlevel 1 (
  echo Failed to create junction %LINK% -^> %TARGET%
  exit /b 1
)
echo Junction: %LINK% -^> %TARGET%
goto :eof
