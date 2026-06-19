use anyhow::{Context, Result};
use clap::Parser;
use dif::dif::Dif;
use serde::Serialize;
use std::path::{Path, PathBuf};

mod dae;
mod maps;
mod mesh;
mod taml;

#[derive(Parser, Debug)]
#[command(name = "dif2t3d", about = "Convert TGE .dif interiors to Torque3D-ready assets")]
struct Args {
    /// Input .dif file
    input: PathBuf,

    /// Output directory (default: <input_stem>/ next to input)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Interior detail level to export (default: inferred from filename or highest LOD)
    #[arg(long)]
    detail: Option<u32>,

    /// Also duplicate collision geometry as LOS-10 (T3D LOS offset convention)
    #[arg(long, default_value_t = true)]
    los: bool,

    /// Also write *_preview.dae for Blender (flipped normals, render mesh only)
    #[arg(long, default_value_t = true)]
    blender_preview: bool,

    /// Level maps directory (MissionInfo.maps); embeds textures in exported DAE
    #[arg(long)]
    maps_dir: Option<PathBuf>,
}

#[derive(Serialize)]
struct RoomMeta {
    source_dif: String,
    detail_level: u32,
    bounding_box_min: [f32; 3],
    bounding_box_max: [f32; 3],
    render_triangles: usize,
    collision_triangles: usize,
    materials: Vec<String>,
    triggers: Vec<TriggerMeta>,
    sub_object_count: usize,
}

#[derive(Serialize)]
struct TriggerMeta {
    name: String,
    datablock: String,
    offset: [f32; 3],
}

fn main() -> Result<()> {
    let args = Args::parse();
    let input = args.input.canonicalize().context("input path")?;
    let stem = input
        .file_stem()
        .and_then(|s| s.to_str())
        .context("input filename")?
        .to_string();

    let output_dir = args
        .output
        .unwrap_or_else(|| input.parent().unwrap_or(Path::new(".")).join(&stem));
    std::fs::create_dir_all(&output_dir)?;

    let bytes = std::fs::read(&input).context("read dif")?;
    let (dif, _version) = Dif::from_bytes(&bytes).map_err(|e| anyhow::anyhow!("{:?}", e))?;

    let detail_hint = args.detail.or_else(|| mesh::detail_hint_from_name(&stem));
    let interior = mesh::select_interior(&dif.interiors, detail_hint);

    let detail = interior.detail_level;
    let render_name = format!("Detail{}", detail);
    let render = mesh::build_render_mesh(interior, &render_name);
    let collision = mesh::build_collision_mesh(interior, "Collision-1");
    let los_mesh = if args.los {
        Some(mesh::build_collision_mesh(interior, "LOS-10"))
    } else {
        None
    };

    let texture_paths = resolve_texture_paths(args.maps_dir.as_deref(), &render.material_slots)?;

    let dae_path = output_dir.join(format!("{}.dae", stem));
    dae::write_dae(
        &dae_path,
        &render,
        &collision,
        los_mesh.as_ref(),
        &dae_options(false, &dae_path, &texture_paths),
    )?;

    if args.blender_preview {
        let mut preview = render.clone();
        mesh::flip_for_blender_preview(&mut preview);
        let preview_path = output_dir.join(format!("{}_preview.dae", stem));
        dae::write_dae(
            &preview_path,
            &preview,
            &collision,
            None,
            &dae_options(true, &preview_path, &texture_paths),
        )?;
        println!("  {} (Blender preview)", preview_path.display());
    }

    let asset_path = output_dir.join(format!("{}.asset.taml", stem));
    taml::write_shape_asset(&asset_path, &stem)?;

    let meta = RoomMeta {
        source_dif: input.display().to_string(),
        detail_level: detail,
        bounding_box_min: [
            interior.bounding_box.min.x,
            interior.bounding_box.min.y,
            interior.bounding_box.min.z,
        ],
        bounding_box_max: [
            interior.bounding_box.max.x,
            interior.bounding_box.max.y,
            interior.bounding_box.max.z,
        ],
        render_triangles: render.triangles.len(),
        collision_triangles: collision.triangles.len(),
        materials: render.material_slots.clone(),
        triggers: dif
            .triggers
            .iter()
            .map(|t| TriggerMeta {
                name: t.name.clone(),
                datablock: t.datablock.clone(),
                offset: [t.offset.x, t.offset.y, t.offset.z],
            })
            .collect(),
        sub_object_count: dif.sub_objects.len(),
    };

    let meta_path = output_dir.join(format!("{}.meta.json", stem));
    std::fs::write(
        &meta_path,
        serde_json::to_string_pretty(&meta).context("serialize meta")?,
    )?;

    println!(
        "Converted {} -> {}",
        input.display(),
        output_dir.display()
    );
    println!(
        "  detail={} render_tris={} collision_tris={} materials={} triggers={}",
        detail,
        render.triangles.len(),
        collision.triangles.len(),
        render.material_slots.len(),
        meta.triggers.len()
    );
    println!("  {}", dae_path.display());
    println!("  {}", asset_path.display());
    println!("  {}", meta_path.display());

    Ok(())
}

fn resolve_texture_paths(
    maps_dir: Option<&Path>,
    material_slots: &[String],
) -> Result<Option<Vec<Option<PathBuf>>>> {
    let Some(maps_dir) = maps_dir else {
        return Ok(None);
    };

    let maps_dir = maps_dir.canonicalize().with_context(|| {
        format!("maps directory not found: {}", maps_dir.display())
    })?;

    let resolved = maps::resolve_material_textures(&maps_dir, material_slots);
    let mut found = 0usize;
    for (material, path) in material_slots.iter().zip(&resolved) {
        match path {
            Some(file) => {
                found += 1;
                println!("  texture {} -> {}", material, file.display());
            }
            None => eprintln!("  warning: texture not found for {}", material),
        }
    }
    println!(
        "  maps_dir={} textures {}/{}",
        maps_dir.display(),
        found,
        material_slots.len()
    );

    Ok(Some(resolved))
}

fn dae_options(
    render_only: bool,
    dae_path: &Path,
    texture_paths: &Option<Vec<Option<PathBuf>>>,
) -> dae::DaeOptions {
    let texture_hrefs = texture_paths.as_ref().map(|paths| {
        paths
            .iter()
            .map(|path| {
                path.as_ref()
                    .map(|file| maps::href_for_collada(dae_path, file))
            })
            .collect()
    });

    dae::DaeOptions {
        render_only,
        texture_hrefs,
    }
}
