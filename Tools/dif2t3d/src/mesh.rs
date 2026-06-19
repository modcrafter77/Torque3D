use dif::interior::{Interior, Surface, SurfaceFlags, TexGenEq};
use dif::types::{PlaneF, Point3F};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
}

#[derive(Clone, Debug)]
pub struct MeshData {
    pub name: String,
    pub vertices: Vec<Vertex>,
    pub triangles: Vec<[u32; 3]>,
    pub material_slots: Vec<String>,
    pub face_material: Vec<usize>,
}

pub fn select_interior<'a>(interiors: &'a [Interior], hint: Option<u32>) -> &'a Interior {
    if interiors.is_empty() {
        panic!("DIF has no interiors");
    }
    if let Some(level) = hint {
        if let Some(found) = interiors.iter().find(|i| i.detail_level == level) {
            return found;
        }
    }
    interiors
        .iter()
        .max_by_key(|i| i.detail_level)
        .unwrap()
}

pub fn build_render_mesh(interior: &Interior, mesh_name: &str) -> MeshData {
    let mut mesh = MeshData {
        name: mesh_name.to_string(),
        vertices: Vec::new(),
        triangles: Vec::new(),
        material_slots: Vec::new(),
        face_material: Vec::new(),
    };

    let mut material_map: HashMap<u16, usize> = HashMap::new();

    for surface in &interior.surfaces {
        if surface.surface_flags.contains(SurfaceFlags::DETAIL) {
            continue;
        }
        emit_surface(interior, surface, &mut mesh, &mut material_map, false);
    }

    mesh
}

pub fn build_collision_mesh(interior: &Interior, mesh_name: &str) -> MeshData {
    let mut mesh = MeshData {
        name: mesh_name.to_string(),
        vertices: Vec::new(),
        triangles: Vec::new(),
        material_slots: vec!["ColMesh".to_string()],
        face_material: Vec::new(),
    };

    let mut material_map: HashMap<u16, usize> = HashMap::new();
    material_map.insert(0, 0);

    for hull in &interior.convex_hulls {
        for j in 0..hull.surface_count {
            let idx = *hull.surface_start.inner() + j as u32;
            let surface_index = &interior.hull_surface_indices[idx as usize];
            match surface_index {
                dif::interior::PossiblyNullSurfaceIndex::Null(null_idx) => {
                    let null_surface = &interior.null_surfaces[*null_idx.inner() as usize];
                    emit_null_surface(interior, null_surface, &mut mesh);
                }
                dif::interior::PossiblyNullSurfaceIndex::NonNull(surf_idx) => {
                    let surface = &interior.surfaces[*surf_idx.inner() as usize];
                    emit_surface(interior, surface, &mut mesh, &mut material_map, true);
                }
            }
        }
    }

    if mesh.triangles.is_empty() {
        let mut fallback_map: HashMap<u16, usize> = HashMap::new();
        for surface in &interior.surfaces {
            if surface.surface_flags.contains(SurfaceFlags::DETAIL) {
                continue;
            }
            emit_surface(interior, surface, &mut mesh, &mut fallback_map, true);
        }
    }

    mesh
}

fn emit_null_surface(interior: &Interior, surface: &dif::interior::NullSurface, mesh: &mut MeshData) {
    let start = *surface.winding_start.inner() as usize;
    let count = surface.winding_count as usize;
    let mut indices: Vec<u32> = interior.indices[start..start + count]
        .iter()
        .map(|i| *i.inner())
        .collect();
    indices = fix_indices(&indices);
    indices.reverse();
    push_fan(mesh, &interior.points, &indices, [0.0, 0.0, 1.0], 0);
}

fn emit_surface(
    interior: &Interior,
    surface: &Surface,
    mesh: &mut MeshData,
    material_map: &mut HashMap<u16, usize>,
    collision: bool,
) {
    let start = *surface.winding_start.inner() as usize;
    let count = surface.winding_count as usize;
    if count < 3 {
        return;
    }

    let mut indices: Vec<u32> = interior.indices[start..start + count]
        .iter()
        .map(|i| *i.inner())
        .collect();
    indices = fix_indices(&indices);
    indices.reverse();

    let plane_idx = *surface.plane_index.inner() as usize;
    let mut normal = interior.normals[*interior.planes[plane_idx].normal_index.inner() as usize];
    if surface.plane_flipped {
        normal = -normal;
    }
    let normal = [normal.x, normal.y, normal.z];

    let tex_gen = &interior.tex_gen_eqs[*surface.tex_gen_index.inner() as usize];
    let mat_slot = if collision {
        0
    } else {
        let tex_idx = *surface.texture_index.inner();
        *material_map.entry(tex_idx).or_insert_with(|| {
            let name = interior
                .material_names
                .get(tex_idx as usize)
                .cloned()
                .unwrap_or_else(|| format!("mat_{}", tex_idx));
            mesh.material_slots.push(name);
            mesh.material_slots.len() - 1
        })
    };

    push_fan_with_uv(mesh, &interior.points, &indices, normal, tex_gen, mat_slot);
}

fn push_fan(
    mesh: &mut MeshData,
    points: &[Point3F],
    indices: &[u32],
    normal: [f32; 3],
    mat_slot: usize,
) {
    if indices.len() < 3 {
        return;
    }
    let base = mesh.vertices.len() as u32;
    for &idx in indices {
        let p = points[idx as usize];
        mesh.vertices.push(Vertex {
            position: [p.x, p.y, p.z],
            normal,
            uv: [0.0, 0.0],
        });
    }
    for i in 1..indices.len() - 1 {
        mesh.triangles.push([base, base + i as u32, base + i as u32 + 1]);
        mesh.face_material.push(mat_slot);
    }
}

fn push_fan_with_uv(
    mesh: &mut MeshData,
    points: &[Point3F],
    indices: &[u32],
    normal: [f32; 3],
    tex_gen: &TexGenEq,
    mat_slot: usize,
) {
    if indices.len() < 3 {
        return;
    }
    let base = mesh.vertices.len() as u32;
    for &idx in indices {
        let p = points[idx as usize];
        let u = plane_eval(p, &tex_gen.plane_x);
        let v = -plane_eval(p, &tex_gen.plane_y);
        mesh.vertices.push(Vertex {
            position: [p.x, p.y, p.z],
            normal,
            uv: [u, v],
        });
    }
    for i in 1..indices.len() - 1 {
        mesh.triangles.push([base, base + i as u32, base + i as u32 + 1]);
        mesh.face_material.push(mat_slot);
    }
}

fn plane_eval(pt: Point3F, plane: &PlaneF) -> f32 {
    pt.x * plane.normal.x + pt.y * plane.normal.y + pt.z * plane.normal.z + plane.distance
}

fn fix_indices(indices: &[u32]) -> Vec<u32> {
    let mut out = vec![0; indices.len()];
    for i in 0..indices.len() {
        if i >= 2 {
            if i % 2 == 0 {
                out[indices.len() - 1 - (i - 2) / 2] = indices[i];
            } else {
                out[(i + 1) / 2] = indices[i];
            }
        } else {
            out[i] = indices[i];
        }
    }
    out
}

/// Flip normals/winding so the room is visible from outside in Blender (preview only).
pub fn flip_for_blender_preview(mesh: &mut MeshData) {
    for v in &mut mesh.vertices {
        v.normal[0] = -v.normal[0];
        v.normal[1] = -v.normal[1];
        v.normal[2] = -v.normal[2];
    }
    for tri in &mut mesh.triangles {
        tri.swap(1, 2);
    }
}

pub fn detail_hint_from_name(path: &str) -> Option<u32> {
    let stem = std::path::Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())?;
    let mut digits = String::new();
    for ch in stem.chars().rev() {
        if ch.is_ascii_digit() {
            digits.insert(0, ch);
        } else if !digits.is_empty() {
            break;
        }
    }
    if digits.is_empty() {
        None
    } else {
        digits.parse().ok()
    }
}
