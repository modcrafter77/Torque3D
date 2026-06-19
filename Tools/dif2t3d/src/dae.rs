use crate::mesh::MeshData;
use anyhow::Result;
use std::collections::BTreeMap;
use std::fmt::Write as _;

pub struct DaeOptions {
    /// Render mesh only (no collision/LOS overlap). Used for Blender preview.
    pub render_only: bool,
    /// Collada `<init_from>` per material slot (parallel to `material_slots`).
    pub texture_hrefs: Option<Vec<Option<String>>>,
}

impl Default for DaeOptions {
    fn default() -> Self {
        Self {
            render_only: false,
            texture_hrefs: None,
        }
    }
}

const COLLADA_NS: &str = "http://www.collada.org/2005/11/COLLADASchema";

pub fn write_dae(
    path: &std::path::Path,
    render: &MeshData,
    collision: &MeshData,
    los: Option<&MeshData>,
    options: &DaeOptions,
) -> Result<()> {
    let render_geom = render_geometry_id(render);
    let render_node = render_node_name(render);

    let mut xml = String::new();
    writeln!(xml, "<?xml version=\"1.0\" encoding=\"utf-8\"?>")?;
    writeln!(
        xml,
        "<COLLADA xmlns=\"{}\" version=\"1.4.1\">",
        COLLADA_NS
    )?;
    writeln!(xml, "  <asset>")?;
    writeln!(xml, "    <unit meter=\"1\" name=\"meter\"/>")?;
    writeln!(xml, "    <up_axis>Z_UP</up_axis>")?;
    writeln!(xml, "  </asset>")?;

    write_materials(&mut xml, render, options.texture_hrefs.as_ref())?;

    writeln!(xml, "  <library_geometries>")?;
    write_geometry(&mut xml, &render_geom, render, true)?;
    if !options.render_only {
        write_geometry(&mut xml, "Collision-1-mesh", collision, false)?;
        if let Some(los_mesh) = los {
            write_geometry(&mut xml, "LOS-10-mesh", los_mesh, false)?;
        }
    }
    writeln!(xml, "  </library_geometries>")?;

    writeln!(xml, "  <library_visual_scenes>")?;
    writeln!(xml, "    <visual_scene id=\"Scene\" name=\"Scene\">")?;
    write_scene_node(&mut xml, &render_node, &render_geom, render.material_slots.len())?;
    if !options.render_only {
        write_scene_node(&mut xml, "Collision-1", "Collision-1-mesh", 0)?;
        if los.is_some() {
            write_scene_node(&mut xml, "LOS-10", "LOS-10-mesh", 0)?;
        }
    }
    writeln!(xml, "    </visual_scene>")?;
    writeln!(xml, "  </library_visual_scenes>")?;
    writeln!(
        xml,
        "  <scene><instance_visual_scene url=\"{}\"/></scene>",
        url_ref("Scene")
    )?;
    writeln!(xml, "</COLLADA>")?;

    std::fs::write(path, xml)?;
    Ok(())
}

fn url_ref(id: &str) -> String {
    format!("#{}", id)
}

fn render_geometry_id(render: &MeshData) -> String {
    format!("{}-mesh", render_node_name(render))
}

fn render_node_name(render: &MeshData) -> String {
    if render.name.starts_with("Detail") {
        render.name.clone()
    } else {
        format!("Detail{}", render.name)
    }
}

fn write_materials(
    xml: &mut String,
    render: &MeshData,
    texture_hrefs: Option<&Vec<Option<String>>>,
) -> Result<()> {
    let has_textures = texture_hrefs
        .map(|hrefs| hrefs.iter().any(|href| href.is_some()))
        .unwrap_or(false);

    if has_textures {
        writeln!(xml, "  <library_images>")?;
        for (i, mat) in render.material_slots.iter().enumerate() {
            let href = texture_hrefs
                .and_then(|hrefs| hrefs.get(i))
                .and_then(|href| href.as_ref());
            if let Some(href) = href {
                let image_id = image_id(i);
                writeln!(
                    xml,
                    "    <image id=\"{id}\" name=\"{name}\"><init_from>{href}</init_from></image>",
                    id = image_id,
                    name = xml_escape(mat),
                    href = xml_escape(href)
                )?;
            }
        }
        writeln!(xml, "  </library_images>")?;
    }

    writeln!(xml, "  <library_materials>")?;
    for (i, mat) in render.material_slots.iter().enumerate() {
        let id = material_id(i);
        let _ = writeln!(
            xml,
            "    <material id=\"{id}\" name=\"{name}\"><instance_effect url=\"{fx}\"/></material>",
            id = id,
            name = xml_escape(mat),
            fx = url_ref(&format!("{id}-fx"))
        );
    }
    writeln!(xml, "  </library_materials>")?;

    writeln!(xml, "  <library_effects>")?;
    for (i, mat) in render.material_slots.iter().enumerate() {
        let id = material_id(i);
        let href = texture_hrefs
            .and_then(|hrefs| hrefs.get(i))
            .and_then(|href| href.as_ref());
        writeln!(
            xml,
            "    <effect id=\"{id}-fx\" name=\"{name}\">",
            id = id,
            name = xml_escape(mat)
        )?;
        writeln!(xml, "      <profile_COMMON>")?;
        if let Some(_) = href {
            let surface_sid = format!("{id}-surface");
            let sampler_sid = format!("{id}-sampler");
            writeln!(xml, "        <newparam sid=\"{sid}\">", sid = surface_sid)?;
            writeln!(xml, "          <surface type=\"2D\">")?;
            writeln!(
                xml,
                "            <init_from>{}</init_from>",
                image_id(i)
            )?;
            writeln!(xml, "          </surface>")?;
            writeln!(xml, "        </newparam>")?;
            writeln!(xml, "        <newparam sid=\"{sid}\">", sid = sampler_sid)?;
            writeln!(xml, "          <sampler2D>")?;
            writeln!(xml, "            <source>{surface}</source>", surface = surface_sid)?;
            writeln!(xml, "          </sampler2D>")?;
            writeln!(xml, "        </newparam>")?;
        }
        writeln!(xml, "        <technique sid=\"common\">")?;
        writeln!(xml, "          <lambert>")?;
        if href.is_some() {
            writeln!(
                xml,
                "            <diffuse><texture texture=\"{sampler}\" texcoord=\"UVMap\"/></diffuse>",
                sampler = format!("{id}-sampler")
            )?;
        } else {
            writeln!(xml, "            <diffuse><color>0.75 0.75 0.75 1</color></diffuse>")?;
        }
        writeln!(xml, "          </lambert>")?;
        writeln!(xml, "        </technique>")?;
        writeln!(xml, "      </profile_COMMON>")?;
        writeln!(xml, "    </effect>")?;
    }
    writeln!(xml, "  </library_effects>")?;
    Ok(())
}

fn write_geometry(xml: &mut String, mesh_id: &str, mesh: &MeshData, with_uv: bool) -> Result<()> {
    writeln!(
        xml,
        "    <geometry id=\"{mesh_id}\" name=\"{mesh_id}\">",
        mesh_id = mesh_id
    )?;
    writeln!(xml, "      <mesh>")?;

    let positions = flatten_positions(&mesh.vertices);
    let normals = flatten_normals(&mesh.vertices);

    write_source(xml, &format!("{mesh_id}-positions"), 3, &positions, "XYZ")?;
    write_source(xml, &format!("{mesh_id}-normals"), 3, &normals, "XYZ")?;
    if with_uv {
        let uvs = flatten_uvs(&mesh.vertices);
        write_source(xml, &format!("{mesh_id}-map0"), 2, &uvs, "ST")?;
    }

    writeln!(xml, "        <vertices id=\"{mesh_id}-vertices\">", mesh_id = mesh_id)?;
    writeln!(
        xml,
        "          <input semantic=\"POSITION\" source=\"{}\"/>",
        url_ref(&format!("{mesh_id}-positions"))
    )?;
    writeln!(xml, "        </vertices>")?;

    if with_uv && !mesh.material_slots.is_empty() {
        let groups = group_triangles_by_material(mesh);
        for (mat_slot, tris) in groups {
            write_triangle_block(xml, mesh_id, &tris, true, Some(mat_slot))?;
        }
    } else {
        write_triangle_block(xml, mesh_id, &mesh.triangles, with_uv, None)?;
    }

    writeln!(xml, "      </mesh>")?;
    writeln!(xml, "    </geometry>")?;
    Ok(())
}

fn group_triangles_by_material(mesh: &MeshData) -> BTreeMap<usize, Vec<[u32; 3]>> {
    let mut groups: BTreeMap<usize, Vec<[u32; 3]>> = BTreeMap::new();
    for (i, tri) in mesh.triangles.iter().enumerate() {
        let slot = mesh.face_material.get(i).copied().unwrap_or(0);
        groups.entry(slot).or_default().push(*tri);
    }
    groups
}

fn write_triangle_block(
    xml: &mut String,
    mesh_id: &str,
    tris: &[[u32; 3]],
    with_uv: bool,
    material_slot: Option<usize>,
) -> Result<()> {
    let indices = pack_triangle_indices(tris, with_uv);
    if let Some(slot) = material_slot {
        writeln!(
            xml,
            "        <triangles material=\"{}\" count=\"{}\">",
            material_symbol(slot),
            tris.len()
        )?;
    } else {
        writeln!(xml, "        <triangles count=\"{}\">", tris.len())?;
    }
    writeln!(
        xml,
        "          <input semantic=\"VERTEX\" offset=\"0\" source=\"{}\"/>",
        url_ref(&format!("{mesh_id}-vertices"))
    )?;
    writeln!(
        xml,
        "          <input semantic=\"NORMAL\" offset=\"1\" source=\"{}\"/>",
        url_ref(&format!("{mesh_id}-normals"))
    )?;
    if with_uv {
        writeln!(
            xml,
            "          <input semantic=\"TEXCOORD\" offset=\"2\" source=\"{}\" set=\"0\"/>",
            url_ref(&format!("{mesh_id}-map0"))
        )?;
    }
    writeln!(xml, "          <p>{}</p>", indices.join(" "))?;
    writeln!(xml, "        </triangles>")?;
    Ok(())
}

fn write_scene_node(
    xml: &mut String,
    node_name: &str,
    geom_id: &str,
    material_count: usize,
) -> Result<()> {
    writeln!(
        xml,
        "      <node id=\"{name}\" name=\"{name}\" type=\"NODE\">",
        name = node_name
    )?;
    writeln!(xml, "        <instance_geometry url=\"{}\">", url_ref(geom_id))?;
    if material_count > 0 {
        writeln!(xml, "          <bind_material>")?;
        writeln!(xml, "            <technique_common>")?;
        for i in 0..material_count {
            writeln!(
                xml,
                "              <instance_material symbol=\"{}\" target=\"{}\"/>",
                material_symbol(i),
                url_ref(&material_id(i))
            )?;
        }
        writeln!(xml, "            </technique_common>")?;
        writeln!(xml, "          </bind_material>")?;
    }
    writeln!(xml, "        </instance_geometry>")?;
    writeln!(xml, "      </node>")?;
    Ok(())
}

fn write_source(xml: &mut String, id: &str, stride: usize, data: &[f32], params: &str) -> Result<()> {
    writeln!(xml, "        <source id=\"{id}\">", id = id)?;
    writeln!(
        xml,
        "          <float_array id=\"{id}-array\" count=\"{count}\">{values}</float_array>",
        id = id,
        count = data.len(),
        values = join_floats(data)
    )?;
    writeln!(xml, "          <technique_common>")?;
    writeln!(
        xml,
        "            <accessor source=\"{src}\" count=\"{count}\" stride=\"{stride}\">",
        src = url_ref(&format!("{id}-array")),
        count = data.len() / stride.max(1),
        stride = stride
    )?;
    for param in params.chars() {
        writeln!(xml, "              <param name=\"{p}\" type=\"float\"/>", p = param)?;
    }
    writeln!(xml, "            </accessor>")?;
    writeln!(xml, "          </technique_common>")?;
    writeln!(xml, "        </source>")?;
    Ok(())
}

fn flatten_positions(vertices: &[crate::mesh::Vertex]) -> Vec<f32> {
    vertices
        .iter()
        .flat_map(|v| v.position.iter().copied())
        .collect()
}

fn flatten_normals(vertices: &[crate::mesh::Vertex]) -> Vec<f32> {
    vertices
        .iter()
        .flat_map(|v| v.normal.iter().copied())
        .collect()
}

fn flatten_uvs(vertices: &[crate::mesh::Vertex]) -> Vec<f32> {
    vertices.iter().flat_map(|v| v.uv.iter().copied()).collect()
}

fn pack_triangle_indices(tris: &[[u32; 3]], with_uv: bool) -> Vec<String> {
    let stride = if with_uv { 3 } else { 2 };
    let mut packed = Vec::new();
    for tri in tris {
        for &idx in tri {
            for _ in 0..stride {
                packed.push(idx.to_string());
            }
        }
    }
    packed
}

fn join_floats(values: &[f32]) -> String {
    values
        .iter()
        .map(|v| format!("{:.6}", v))
        .collect::<Vec<_>>()
        .join(" ")
}

fn image_id(i: usize) -> String {
    format!("img-{}", i)
}

fn material_id(i: usize) -> String {
    format!("mat-{}", i)
}

fn material_symbol(i: usize) -> String {
    format!("mat-{}", i)
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
