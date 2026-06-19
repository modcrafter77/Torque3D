# RefuzeGame on Torque3D 4.0.3

Phase 0 bootstrap for the Home Wars / Refuze port.

## Configure (Win64)

From a **x64 Native Tools Command Prompt for VS 2022** (or after `vcvars64.bat`):

```bat
Projects\RefuzeGame\configure.bat
cmake --build build\RefuzeGame --config Debug --target install
```

CMake flags used:

- `TORQUE_APP_NAME=RefuzeGame`
- `TORQUE_TEMPLATE=BaseGame`
- `TORQUE_PHYSICS_BULLET=ON`
- `TORQUE_NAVIGATION=ON`
- Win64 / C++17 (from root `CMakeLists.txt`)

## Layout

| Path | Purpose |
|------|---------|
| `My Projects/RefuzeGame/game/` | Runtime game root (`RefuzeGame.exe`) |
| `My Projects/RefuzeGame/game/data/RefuzeGame/` | RefuzeGame module |
| `My Projects/RefuzeGame/source/refuze/` | Future C++ game code |
| `Projects/RefuzeGame/overlay/` | Source-of-truth copied by `setupOverlay.bat` |
| `game/common`, `game/starter.game` | Junctions to `refuze/data_dump` |

## Legacy scripts

Engine patch: `isScriptFile()` accepts `.cs` in addition to `.tscript`.
Legacy data is exposed via directory junctions: `game/common/`, `game/starter.game/` -> `refuze/data_dump`.

## Smoke test

Run `RefuzeGame.exe` from `My Projects/RefuzeGame/game/`.
BaseGame main menu should appear; TorqueScript console (`~`) should work.
Bootstrap mission: `data/RefuzeGame/levels/empty.mis`.
