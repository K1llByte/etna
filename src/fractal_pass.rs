use std::sync::Arc;

use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer},
    command_buffer::{ClearColorImageInfo, CopyBufferToImageInfo, CopyImageToBufferInfo},
    descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet},
    format::{ClearColorValue, Format},
    image::{view::ImageView, Image, ImageCreateInfo, ImageType, ImageUsage},
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter},
    pipeline::{
        compute::ComputePipelineCreateInfo, layout::PipelineDescriptorSetLayoutCreateInfo,
        ComputePipeline, Pipeline, PipelineBindPoint, PipelineLayout,
        PipelineShaderStageCreateInfo,
    },
};

use crate::context::{CmdBufferType, VulkanContext};

mod fractal_cs {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "shaders/fractal.comp",
    }
}

// Pass for a generating mandelbrot fractal image
pub struct FractalPass {
    // Resurces
    pub image: Arc<Image>,
    pub image_view: Arc<ImageView>,
    pub transfer_buffer: Subbuffer<[u8]>,
    // Pipeline
    pub compute_pipeline: Arc<ComputePipeline>,
    pub dset_layout_idx: u32,
    pub dset: Arc<PersistentDescriptorSet>,
}

impl FractalPass {
    pub fn new(context: &VulkanContext) -> Self {
        // Initialize device target image
        // Allocate image on the device and initialize with foo_data
        let image = Image::new(
            context.allocator.clone(),
            ImageCreateInfo {
                image_type: ImageType::Dim2d,
                format: Format::R8G8B8A8_UNORM,
                extent: [1024, 1024, 1],
                usage: ImageUsage::TRANSFER_SRC | ImageUsage::STORAGE,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
                ..Default::default()
            },
        )
        .unwrap();
        // Initialize transfer output buffer
        let image_view = ImageView::new_default(image.clone()).unwrap();
        let image_size = 1024 * 1024 * 4;
        let transfer_buffer = Buffer::from_iter(
            context.allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::TRANSFER_DST,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_HOST
                    | MemoryTypeFilter::HOST_RANDOM_ACCESS,
                ..Default::default()
            },
            (0..image_size).map(|_| 0u8),
        )
        .unwrap();

        // Create mandlebrot fractal compute pipeline
        let compute_shader =
            fractal_cs::load(context.device.clone()).expect("Failed to create shader module!");
        // Create compute pipeline.
        let entry_point = compute_shader.entry_point("main").unwrap();
        let stage = PipelineShaderStageCreateInfo::new(entry_point);
        let layout = PipelineLayout::new(
            context.device.clone(),
            PipelineDescriptorSetLayoutCreateInfo::from_stages([&stage])
                .into_pipeline_layout_create_info(context.device.clone())
                .unwrap(),
        )
        .unwrap();
        let compute_pipeline = ComputePipeline::new(
            context.device.clone(),
            None,
            ComputePipelineCreateInfo::stage_layout(stage, layout),
        )
        .expect("failed to create compute pipeline");

        // Create descriptor set for compute shader
        let pipeline_layout = compute_pipeline.layout();
        let dset_layouts = pipeline_layout.set_layouts();
        // Get the layout of the only descriptor sets
        let dset_layout_idx: u32 = 0;
        let dset_layout = dset_layouts.get(dset_layout_idx as usize).unwrap();
        let dset = PersistentDescriptorSet::new(
            &context.dset_allocator,
            dset_layout.clone(),
            [WriteDescriptorSet::image_view(0, image_view.clone())],
            [],
        )
        .unwrap();

        FractalPass {
            image,
            image_view,
            transfer_buffer,
            compute_pipeline,
            dset_layout_idx,
            dset,
        }
    }

    pub fn execute(&self, cmd_buffer: &mut CmdBufferType) {
        cmd_buffer
            .bind_pipeline_compute(self.compute_pipeline.clone())
            .unwrap()
            .bind_descriptor_sets(
                PipelineBindPoint::Compute,
                self.compute_pipeline.layout().clone(),
                self.dset_layout_idx,
                self.dset.clone(),
            )
            .unwrap()
            .dispatch([128, 128, 1])
            .unwrap()
            .copy_image_to_buffer(CopyImageToBufferInfo::image_buffer(
                self.image.clone(),
                self.transfer_buffer.clone(),
            ))
            .unwrap();
    }
}
