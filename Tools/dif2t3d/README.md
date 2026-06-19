# dif2t3d

Offline CLI: TGE `.dif` → Torque3D-ready **COLLADA** + **ShapeAsset** metadata.

## Build

```bash
cd Torque3D/tools/dif2t3d
cargo build --release
```

Binary: `target/release/dif2t3d.exe`

Depends on patched [`io_dif` `libdif`](../../../DIFImporter/io_dif/rust/libdif) (TGE 1.2 surface layout + large lightmaps).

## Usage

```bash
dif2t3d path/to/room.dif -o output/dir
```

Options:

- `--detail N` — pick interior LOD by `detail_level` (default: from filename suffix or highest)
- `--los` / `--no-los` — emit duplicate `LOS-10` collision mesh (default: on)
- `--blender-preview` / `--no-blender-preview` — write `*_preview.dae` with outward normals (default: on)
- `--maps-dir PATH` — embed level texture images in DAE (MissionInfo.maps)
- `--maps-dir` also writes T3D `ImageAsset` + `MaterialAsset` stubs under `materials/`

## Output

For `pantry_32.dif` → `output/pantry_32/`:

| File | Purpose |
|------|---------|
| `pantry_32.dae` | Render mesh `Detail0` + `Collision-1` + `LOS-10` |
| `pantry_32.asset.taml` | T3D `ShapeAsset` stub |
| `pantry_32.meta.json` | Bounds, materials, DIF triggers, stats |

## Data sources

`.dif` files live in `C:/projects/gamedev/Refuze/old_res_unpacked/starter.game/data/interiors/` (not in `data_dump` scripts-only tree).

## Spike (phase 1 gate)

```bash
dif2t3d "C:/projects/gamedev/Refuze/old_res_unpacked/starter.game/data/interiors/01.pantry/pantry_32.dif" \
  -o "My Projects/RefuzeGame/game/data/RefuzeGame/art/interiors/01.pantry/pantry_32"
```

Expected: thousands of render/collision triangles, 10+ materials.

Next: import DAE in T3D, place as `TSStatic`, align with `01.pantry.mis` spawn (phase 2).
