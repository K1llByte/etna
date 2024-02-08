use std::sync::Arc;

use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer},
    descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet},
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter},
    pipeline::{
        compute::ComputePipelineCreateInfo, layout::PipelineDescriptorSetLayoutCreateInfo,
        ComputePipeline, Pipeline, PipelineBindPoint, PipelineLayout,
        PipelineShaderStageCreateInfo,
    },
};

use crate::context::{CmdBufferType, VulkanContext};

mod cs {
    vulkano_shaders::shader! {
        ty: "compute",
        src: r"
            #version 460

            layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

            layout(set = 0, binding = 0) buffer Data {
                uint data[];
            } buf;

            void main() {
                uint idx = gl_GlobalInvocationID.x;
                buf.data[idx] *= 12;
            }
        ",
    }
}

pub struct TestPass {
    pub data_buffer: Subbuffer<[u32]>,
    pub compute_pipeline: Arc<ComputePipeline>,
    pub dset_layout_idx: u32,
    pub dset: Arc<PersistentDescriptorSet>,
}

impl TestPass {
    pub fn new(context: &VulkanContext) -> Self {
        // Allocate buffer on the device and initialize with foo_data
        let data_buffer = Buffer::from_iter(
            context.allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::STORAGE_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            0..65536u32,
        )
        .expect("failed to create buffer");

        // Try to load shader. Will fail if compute shader code has errors.
        let compute_shader =
            cs::load(context.device.clone()).expect("Failed to create shader module!");

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
        let dset_layout_idx = 0;
        let dset_layout = dset_layouts.get(dset_layout_idx).unwrap();
        let dset = PersistentDescriptorSet::new(
            &context.dset_allocator,
            dset_layout.clone(),
            [WriteDescriptorSet::buffer(0, data_buffer.clone())],
            [],
        )
        .unwrap();

        TestPass {
            data_buffer,
            compute_pipeline,
            dset_layout_idx: dset_layout_idx as u32,
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
            .dispatch([1024, 1, 1])
            .unwrap();
    }
}
