use anyhow::Result;
use std::path::Path;

pub fn write_shape_asset(path: &Path, asset_name: &str) -> Result<()> {
    let dae_name = format!("{}.DAE", asset_name);
    let body = format!(
        r#"<ShapeAsset
    canSave="true"
    canSaveDynamicFields="true"
    AssetName="{name}"
    fileName="@assetFile={dae}" />
"#,
        name = asset_name,
        dae = dae_name
    );
    std::fs::write(path, body)?;
    Ok(())
}
