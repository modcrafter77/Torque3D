use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

const MODULE: &str = "RefuzeGame";

pub struct MaterialExport {
    pub slot_name: String,
    pub map_to: String,
    pub asset_name: String,
    pub image_asset_name: String,
    pub image_file: String,
}

/// Copy textures and write ImageAsset + MaterialAsset TAML files for T3D.
pub fn write_material_assets(
    output_dir: &Path,
    material_slots: &[String],
    texture_paths: &[Option<PathBuf>],
) -> Result<Vec<MaterialExport>> {
    let materials_dir = output_dir.join("materials");
    fs::create_dir_all(&materials_dir)?;

    let mut exports = Vec::new();

    for (material, texture_path) in material_slots.iter().zip(texture_paths) {
        let Some(texture_path) = texture_path else {
            continue;
        };

        let map_to = material_map_to_name(material);
        let asset_name = material_asset_name(&map_to);
        let image_asset_name = format!("{asset_name}_ALBEDO");
        let image_filename = image_filename_for_texture(&map_to, texture_path);

        let dest_image = materials_dir.join(&image_filename);
        if !dest_image.exists() {
            fs::copy(texture_path, &dest_image).with_context(|| {
                format!(
                    "copy texture {} -> {}",
                    texture_path.display(),
                    dest_image.display()
                )
            })?;
        }

        write_image_asset(&materials_dir, &image_asset_name, &image_filename)?;
        write_material_asset(
            &materials_dir,
            &asset_name,
            &map_to,
            &image_asset_name,
        )?;

        exports.push(MaterialExport {
            slot_name: material.clone(),
            map_to,
            asset_name,
            image_asset_name,
            image_file: image_filename,
        });
    }

    Ok(exports)
}

fn material_map_to_name(material: &str) -> String {
    material
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(material)
        .to_string()
}

fn material_asset_name(map_to: &str) -> String {
    map_to
        .chars()
        .map(|ch| if ch == '.' { '_' } else { ch })
        .collect()
}

fn image_filename_for_texture(map_to: &str, texture_path: &Path) -> String {
    let ext = texture_path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("jpg");
    format!("{map_to}.{ext}")
}

fn write_image_asset(dir: &Path, asset_name: &str, image_filename: &str) -> Result<()> {
    let path = dir.join(format!("{asset_name}.asset.taml"));
    let body = format!(
        r#"<ImageAsset
    AssetName="{asset_name}"
    imageFile="@assetFile={image_filename}" />
"#
    );
    fs::write(path, body)?;
    Ok(())
}

fn write_material_asset(
    dir: &Path,
    asset_name: &str,
    map_to: &str,
    image_asset_name: &str,
) -> Result<()> {
    let path = dir.join(format!("{asset_name}.asset.taml"));
    let image_ref = format!("{MODULE}:{image_asset_name}");
    let body = format!(
        r#"<MaterialAsset
    AssetName="{asset_name}"
    materialDefinitionName="{map_to}"
    imageMap0="@asset={image_ref}"
    originalFilePath="{map_to}">
    <Material
        Name="{map_to}"
        mapTo="{map_to}"
        originalAssetName="{asset_name}">
        <Material.Stages>
            <Stages_beginarray
                DiffuseMapAsset="{image_ref}"/>
        </Material.Stages>
    </Material>
</MaterialAsset>
"#
    );
    fs::write(path, body)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_to_strips_directory() {
        assert_eq!(
            material_map_to_name("ddd/concrete.hi.base"),
            "concrete.hi.base"
        );
    }

    #[test]
    fn asset_name_replaces_dots() {
        assert_eq!(
            material_asset_name("concrete.hi.base"),
            "concrete_hi_base"
        );
    }
}
