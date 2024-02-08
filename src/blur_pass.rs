use std::sync::Arc;

use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer},
    command_buffer::{ClearColorImageInfo, CopyBufferToImageInfo, CopyImageToBufferInfo},
    descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet},
    format::{ClearColorValue, Format},
    image::{Image, ImageCreateInfo, ImageType, ImageUsage},
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter},
    pipeline::{
        compute::ComputePipelineCreateInfo, layout::PipelineDescriptorSetLayoutCreateInfo,
        ComputePipeline, Pipeline, PipelineBindPoint, PipelineLayout,
        PipelineShaderStageCreateInfo,
    },
};

use crate::context::{CmdBufferType, VulkanContext};

// Pass for a Gaussian blur filter
pub struct BlurPass {
    pub image: Arc<Image>,
    pub transfer_buffer: Subbuffer<[u8]>,
}

impl BlurPass {
    pub fn new(context: &VulkanContext) -> Self {
        // Allocate image on the device and initialize with foo_data
        let image = Image::new(
            context.allocator.clone(),
            ImageCreateInfo {
                image_type: ImageType::Dim2d,
                format: Format::R8G8B8A8_UNORM,
                extent: [1024, 1024, 1],
                usage: ImageUsage::TRANSFER_SRC | ImageUsage::TRANSFER_DST,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
                ..Default::default()
            },
        )
        .unwrap();
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

        BlurPass {
            image,
            transfer_buffer,
        }
    }

    pub fn execute(&self, cmd_buffer: &mut CmdBufferType) {
        cmd_buffer
            .clear_color_image(ClearColorImageInfo {
                clear_value: ClearColorValue::Float([0.0, 0.5, 0.3, 1.0]),
                ..ClearColorImageInfo::image(self.image.clone())
            })
            .unwrap()
            .copy_image_to_buffer(CopyImageToBufferInfo::image_buffer(
                self.image.clone(),
                self.transfer_buffer.clone(),
            ))
            .unwrap();
    }
}
