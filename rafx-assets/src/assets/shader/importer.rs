use crate::assets::shader::ShaderAssetData;
use distill::core::AssetUuid;
use distill::importer::{ImportOp, ImportedAsset, Importer, ImporterValue};
use rafx_api::{RafxHashedShaderPackage, RafxShaderPackage, RafxShaderPackageVulkan};
use serde::{Deserialize, Serialize};
use std::io::Read;
use type_uuid::*;

// There may be a better way to do this type coercing
// fn coerce_result_str<T>(result: Result<T, &str>) -> distill::importer::Result<T> {
//     let ok = result.map_err(|x| -> Box<dyn std::error::Error + Send> { Box::<dyn std::error::Error + Send + Sync>::from(x) })?;
//     Ok(ok)
// }

fn coerce_result_string<T>(result: Result<T, String>) -> distill::importer::Result<T> {
    let ok = result.map_err(|x| -> Box<dyn std::error::Error + Send> {
        Box::<dyn std::error::Error + Send + Sync>::from(x)
    })?;
    Ok(ok)
}

#[derive(TypeUuid, Serialize, Deserialize, Default)]
#[uuid = "867bc278-67b5-469c-aeea-1c05da722918"]
pub struct ShaderImporterSpvState(Option<AssetUuid>);

#[derive(TypeUuid)]
#[uuid = "90fdad4b-cec1-4f59-b679-97895711b6e1"]
pub struct ShaderImporterSpv;
impl Importer for ShaderImporterSpv {
    fn version_static() -> u32
    where
        Self: Sized,
    {
        5
    }

    fn version(&self) -> u32 {
        Self::version_static()
    }

    type Options = ();

    type State = ShaderImporterSpvState;

    /// Reads the given bytes and produces assets.
    #[profiling::function]
    fn import(
        &self,
        _op: &mut ImportOp,
        source: &mut dyn Read,
        _options: &Self::Options,
        state: &mut Self::State,
    ) -> distill::importer::Result<ImporterValue> {
        let asset_id = state
            .0
            .unwrap_or_else(|| AssetUuid(*uuid::Uuid::new_v4().as_bytes()));
        *state = ShaderImporterSpvState(Some(asset_id));

        // Raw compiled shader
        let mut spv_bytes = Vec::new();
        source.read_to_end(&mut spv_bytes)?;

        log::trace!(
            "Import shader asset {:?} with {} bytes of code",
            asset_id,
            spv_bytes.len()
        );

        // The hash is used in some places identify the shader
        let shader_package = RafxShaderPackage {
            dx12: None,
            metal: None,
            vk: Some(RafxShaderPackageVulkan::SpvBytes(spv_bytes)),
            gles2: None,
            gles3: None,
            vk_reflection: None,
            dx12_reflection: None,
            metal_reflection: None,
            gles2_reflection: None,
            gles3_reflection: None,
            debug_name: None,
        };

        let hashed_shader_package = RafxHashedShaderPackage::new(shader_package);

        let shader_asset = ShaderAssetData {
            shader_package: hashed_shader_package,
        };

        Ok(ImporterValue {
            assets: vec![ImportedAsset {
                id: asset_id,
                search_tags: vec![],
                build_deps: vec![],
                load_deps: vec![],
                build_pipeline: None,
                asset_data: Box::new(shader_asset),
            }],
        })
    }
}

#[derive(TypeUuid, Serialize, Deserialize, Default)]
#[uuid = "d4fb07ce-76e6-497e-ac31-bcaeb43528aa"]
pub struct ShaderImporterCookedState(Option<AssetUuid>);

#[derive(TypeUuid)]
#[uuid = "cab0cf4c-16ff-4dbd-aae7-8705246d85d6"]
pub struct ShaderImporterCooked;
impl Importer for ShaderImporterCooked {
    fn version_static() -> u32
    where
        Self: Sized,
    {
        5
    }

    fn version(&self) -> u32 {
        Self::version_static()
    }

    type Options = ();

    type State = ShaderImporterCookedState;

    /// Reads the given bytes and produces assets.
    #[profiling::function]
    fn import(
        &self,
        _op: &mut ImportOp,
        source: &mut dyn Read,
        _options: &Self::Options,
        state: &mut Self::State,
    ) -> distill::importer::Result<ImporterValue> {
        let asset_id = state
            .0
            .unwrap_or_else(|| AssetUuid(*uuid::Uuid::new_v4().as_bytes()));
        *state = ShaderImporterCookedState(Some(asset_id));

        // Raw compiled shader
        let mut bytes = Vec::new();
        source.read_to_end(&mut bytes)?;

        let hashed_shader_package: RafxHashedShaderPackage = coerce_result_string(
            bincode::deserialize::<RafxHashedShaderPackage>(&bytes)
                .map_err(|x| format!("Failed to deserialize cooked shader: {:?}", x)),
        )?;

        log::trace!(
            "Import shader asset {:?} with hash {:?}",
            asset_id,
            hashed_shader_package.shader_package_hash(),
        );

        let shader_asset = ShaderAssetData {
            shader_package: hashed_shader_package,
        };

        Ok(ImporterValue {
            assets: vec![ImportedAsset {
                id: asset_id,
                search_tags: vec![],
                build_deps: vec![],
                load_deps: vec![],
                build_pipeline: None,
                asset_data: Box::new(shader_asset),
            }],
        })
    }
}
