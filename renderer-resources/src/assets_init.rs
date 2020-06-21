use legion::prelude::*;
use renderer_shell_vulkan::VkDeviceContext;
use crate::resource_managers::ResourceManager;
use renderer_assets::asset_resource::AssetResource;
use renderer_assets::assets::shader::ShaderAsset;
use renderer_assets::assets::pipeline::{PipelineAsset, MaterialAsset, MaterialInstanceAsset, RenderpassAsset};
use renderer_assets::assets::image::ImageAsset;
use renderer_assets::assets::buffer::BufferAsset;
use renderer_assets::assets::gltf::{MeshAsset, GltfMaterialAsset};
use ash::prelude::VkResult;

pub fn init_renderer_assets(
    resources: &mut Resources,
) {
    //
    // Create the resource manager
    //
    let device_context = resources.get_mut::<VkDeviceContext>().unwrap().clone();
    let mut resource_manager = ResourceManager::new(&device_context);
    resources.insert(resource_manager);

    //
    // Connect the asset system with the resource manager
    //
    let mut asset_resource_fetch = resources.get_mut::<AssetResource>().unwrap();
    let asset_resource = &mut *asset_resource_fetch;

    let mut resource_manager_fetch = resources.get_mut::<ResourceManager>().unwrap();
    let resource_manager = &mut *resource_manager_fetch;

    asset_resource.add_storage_with_load_handler::<ShaderAsset, _>(Box::new(
        resource_manager.create_shader_load_handler(),
    ));
    asset_resource.add_storage_with_load_handler::<PipelineAsset, _>(Box::new(
        resource_manager.create_pipeline_load_handler(),
    ));
    asset_resource.add_storage_with_load_handler::<RenderpassAsset, _>(Box::new(
        resource_manager.create_renderpass_load_handler(),
    ));
    asset_resource.add_storage_with_load_handler::<MaterialAsset, _>(Box::new(
        resource_manager.create_material_load_handler(),
    ));
    asset_resource.add_storage_with_load_handler::<MaterialInstanceAsset, _>(Box::new(
        resource_manager.create_material_instance_load_handler(),
    ));
    asset_resource.add_storage_with_load_handler::<ImageAsset, _>(Box::new(
        resource_manager.create_image_load_handler(),
    ));
    asset_resource.add_storage_with_load_handler::<BufferAsset, _>(Box::new(
        resource_manager.create_buffer_load_handler(),
    ));
    asset_resource.add_storage_with_load_handler::<MeshAsset, _>(Box::new(
        resource_manager.create_mesh_load_handler(),
    ));

    asset_resource.add_storage::<GltfMaterialAsset>();
}

pub fn update_renderer_assets(
    resources: &Resources
) -> VkResult<()> {
    resources.get_mut::<ResourceManager>().unwrap().update_resources()
}

pub fn destroy_renderer_assets(
    resources: &mut Resources,
) {
    resources.remove::<ResourceManager>();
}


