
use ash::vk;
use ash::prelude::*;
use ash::version::DeviceV1_0;
use super::types as dsc;

pub fn create_shader_module(
    device: &ash::Device,
    shader_module: &dsc::ShaderModule
) -> VkResult<vk::ShaderModule> {
    let create_info = vk::ShaderModuleCreateInfo::builder()
        .code(&shader_module.code);

    unsafe {
        device.create_shader_module(&*create_info, None)
    }
}

pub fn create_descriptor_set_layout(
    device: &ash::Device,
    descriptor_set_layout: &dsc::DescriptorSetLayout
) -> VkResult<vk::DescriptorSetLayout> {
    let bindings : Vec<_> = descriptor_set_layout.descriptor_set_layout_bindings.iter()
        .map(|binding| binding.as_builder().build())
        .collect();

    let create_info = vk::DescriptorSetLayoutCreateInfo::builder()
        .bindings(&bindings);

    unsafe {
        device.create_descriptor_set_layout(&*create_info, None)
    }
}

pub fn create_pipeline_layout(
    device: &ash::Device,
    pipeline_layout: &dsc::PipelineLayout,
    descriptor_set_layouts: &[vk::DescriptorSetLayout]
) -> VkResult<vk::PipelineLayout> {
    let push_constant_ranges: Vec<_> = pipeline_layout.push_constant_ranges.iter()
        .map(|push_constant_range| push_constant_range.as_builder().build())
        .collect();

    let create_info = vk::PipelineLayoutCreateInfo::builder()
        .set_layouts(descriptor_set_layouts)
        .push_constant_ranges(push_constant_ranges.as_slice());

    unsafe {
        device.create_pipeline_layout(&*create_info, None)
    }
}

pub fn create_renderpass(
    device: &ash::Device,
    renderpass: &dsc::RenderPass,
    swapchain_surface_info: &dsc::SwapchainSurfaceInfo
) -> VkResult<vk::RenderPass> {
    let attachments : Vec<_> = renderpass.attachments.iter()
        .map(|attachment| attachment.as_builder(&swapchain_surface_info).build())
        .collect();

    // One vec per subpass
    let mut color_attachments : Vec<Vec<vk::AttachmentReference>> = Vec::with_capacity(renderpass.subpasses.len());
    let mut input_attachments : Vec<Vec<vk::AttachmentReference>> = Vec::with_capacity(renderpass.subpasses.len());
    let mut resolve_attachments : Vec<Vec<vk::AttachmentReference>> = Vec::with_capacity(renderpass.subpasses.len());

    // One element per subpass that has a depth stencil attachment specified
    let mut depth_stencil_attachments : Vec<vk::AttachmentReference> = Vec::with_capacity(renderpass.subpasses.len());

    let mut subpasses : Vec<_> = Vec::with_capacity(renderpass.subpasses.len());

    for subpass in &renderpass.subpasses {
        color_attachments.push(subpass.color_attachments.iter().map(|attachment| attachment.as_builder().build()).collect());
        input_attachments.push(subpass.input_attachments.iter().map(|attachment| attachment.as_builder().build()).collect());

        // The resolve attachment array must be unused or of length == color attachments. If
        // the number of subpass resolves doesn't match the color attachments, truncate or
        // insert attachment references with AttachmentIndex::Unused
        if subpass.resolve_attachments.len() > subpass.color_attachments.len() {
            log::warn!("A renderpass definition has more resolve attachments than color attachments. The additional resolve attachments will be discarded");
        }

        let mut subpass_resolve_attachments : Vec<_> = subpass.resolve_attachments.iter().map(|attachment| attachment.as_builder().build()).collect();
        if !subpass_resolve_attachments.is_empty() {
            let unused_attachment = dsc::AttachmentReference {
                attachment: dsc::AttachmentIndex::Unused,
                layout: Default::default()
            }.as_builder().build();
            subpass_resolve_attachments.resize(color_attachments.len(), unused_attachment);
        }
        resolve_attachments.push(subpass_resolve_attachments);

        let mut subpass_description_builder = vk::SubpassDescription::builder()
            .pipeline_bind_point(subpass.pipeline_bind_point.into())
            .color_attachments(color_attachments.last().unwrap())
            .input_attachments(input_attachments.last().unwrap());

        // Only specify resolve attachments if we have more than zero of them
        {
            let subpass_resolve_attachments = resolve_attachments.last().unwrap();
            if subpass_resolve_attachments.len() > 0 {
                subpass_description_builder = subpass_description_builder.resolve_attachments(subpass_resolve_attachments);
            }
        }

        // Only specify a depth stencil attachment if we have one
        if let Some(depth_stencil_attachment) = &subpass.depth_stencil_attachment {
            depth_stencil_attachments.push(depth_stencil_attachment.as_builder().build());
            subpass_description_builder = subpass_description_builder.depth_stencil_attachment(depth_stencil_attachments.last().unwrap());
        }

        let subpass_description = subpass_description_builder.build();

        subpasses.push(subpass_description);
    }

    let dependencies : Vec<_> = renderpass.dependencies.iter()
        .map(|dependency| dependency.as_builder().build())
        .collect();

    let create_info = vk::RenderPassCreateInfo::builder()
        .attachments(&attachments)
        .subpasses(&subpasses)
        .dependencies(&dependencies);

    unsafe {
        device.create_render_pass(&*create_info, None)
    }
}

pub fn create_graphics_pipeline(
    device: &ash::Device,
    graphics_pipeline: &dsc::GraphicsPipeline,
    pipeline_layout: vk::PipelineLayout,
    renderpass: vk::RenderPass,
    shader_modules: &[vk::ShaderModule],
    swapchain_surface_info: &dsc::SwapchainSurfaceInfo,
) -> VkResult<vk::Pipeline> {
    let fixed_function_state = &graphics_pipeline.fixed_function_state;

    let input_assembly_state = fixed_function_state.input_assembly_state.as_builder().build();

    let mut vertex_input_attribute_descriptions: Vec<_> = fixed_function_state.vertex_input_state.attribute_descriptions.iter()
        .map(|attribute| attribute.as_builder(swapchain_surface_info).build())
        .collect();

    let mut vertex_input_binding_descriptions: Vec<_> = fixed_function_state.vertex_input_state.binding_descriptions.iter()
        .map(|binding| binding.as_builder().build())
        .collect();

    let vertex_input_state = vk::PipelineVertexInputStateCreateInfo::builder()
        .vertex_attribute_descriptions(vertex_input_attribute_descriptions.as_slice())
        .vertex_binding_descriptions(&vertex_input_binding_descriptions);

    let scissors: Vec<_> = fixed_function_state.viewport_state.scissors.iter()
        .map(|scissors| scissors.to_rect2d(swapchain_surface_info))
        .collect();

    let viewports: Vec<_> = fixed_function_state.viewport_state.viewports.iter()
        .map(|viewport| viewport.as_builder(swapchain_surface_info).build())
        .collect();

    let viewport_state = vk::PipelineViewportStateCreateInfo::builder()
        .scissors(&scissors)
        .viewports(&viewports);

    let rasterization_state = fixed_function_state.rasterization_state.as_builder();

    let multisample_state = fixed_function_state.multisample_state.as_builder();

    let color_blend_attachments: Vec<_> = fixed_function_state.color_blend_state.attachments.iter().map(|attachment| attachment.as_builder().build()).collect();
    let color_blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
        .logic_op(fixed_function_state.color_blend_state.logic_op.into())
        .logic_op_enable(fixed_function_state.color_blend_state.logic_op_enable)
        .blend_constants(fixed_function_state.color_blend_state.blend_constants_as_f32())
        .attachments(&color_blend_attachments);

    let dynamic_states: Vec<vk::DynamicState> = fixed_function_state.dynamic_state.dynamic_states.iter().map(|dynamic_state| dynamic_state.clone().into()).collect();
    let dynamic_state = vk::PipelineDynamicStateCreateInfo::builder()
        .dynamic_states(&dynamic_states);

    let mut stages = Vec::with_capacity(graphics_pipeline.pipeline_shader_stages.stages.len());
    for (pipeline_shader_stage, module) in graphics_pipeline.pipeline_shader_stages.stages.iter().zip(shader_modules) {
        stages.push(vk::PipelineShaderStageCreateInfo::builder()
            .stage(pipeline_shader_stage.stage.into())
            .module(*module)
            .name(&pipeline_shader_stage.entry_name)
            .build());
    }

    let pipeline_info = vk::GraphicsPipelineCreateInfo::builder()
        .input_assembly_state(&input_assembly_state)
        .vertex_input_state(&vertex_input_state)
        .viewport_state(&viewport_state)
        .rasterization_state(&rasterization_state)
        .multisample_state(&multisample_state)
        .color_blend_state(&color_blend_state)
        .dynamic_state(&dynamic_state)
        .layout(pipeline_layout)
        .render_pass(renderpass)
        .stages(&stages)
        .build();

    unsafe {
        match device.create_graphics_pipelines(
            vk::PipelineCache::null(),
            &[pipeline_info],
            None,
        ) {
            Ok(result) => Ok(result[0]),
            Err(e) => Err(e.1),
        }
    }
}
