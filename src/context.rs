use std::sync::Arc;

use vulkano::{
    command_buffer::{
        allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder, CommandBufferUsage,
        PrimaryAutoCommandBuffer,
    },
    descriptor_set::allocator::StandardDescriptorSetAllocator,
    device::{Device, DeviceCreateInfo, Queue, QueueCreateInfo, QueueFlags},
    instance::{Instance, InstanceCreateInfo},
    memory::allocator::{FreeListAllocator, GenericMemoryAllocator, StandardMemoryAllocator},
    sync,
    sync::GpuFuture,
    VulkanLibrary,
};

pub type CmdBufferType = AutoCommandBufferBuilder<
    PrimaryAutoCommandBuffer<Arc<StandardCommandBufferAllocator>>,
    Arc<StandardCommandBufferAllocator>,
>;

pub struct VulkanContext {
    // Vulkan instance
    pub instance: Arc<Instance>,
    // GPU device
    pub device: Arc<Device>,
    // Main queue (graphics and compute)
    pub queue: Arc<Queue>,
    // Device allocator
    pub allocator: Arc<GenericMemoryAllocator<FreeListAllocator>>,
    // Descriptor set allocator
    pub dset_allocator: Arc<StandardDescriptorSetAllocator>,
    // Command buffer allocator
    pub cmd_buffer_allocator: Arc<StandardCommandBufferAllocator>,
}

impl VulkanContext {
    pub fn new() -> VulkanContext {
        let library = VulkanLibrary::new().expect("Couldn't load vulkan!");
        // Create vulkan instance
        let instance = Instance::new(library, InstanceCreateInfo::default())
            .expect("Couldn't create vulkan instance!");

        // Select physical device (GPU)
        let physical_device = instance
            .enumerate_physical_devices()
            .expect("Couldn't list devices")
            .next()
            .expect("No available devices with vulkan support!");

        // Chose a queue family with graphics capabilities
        let queue_family_index = physical_device
            .queue_family_properties()
            .iter()
            .enumerate()
            .position(|(_, queue_family_properties)| {
                queue_family_properties
                    .queue_flags
                    .contains(QueueFlags::GRAPHICS | QueueFlags::COMPUTE)
            })
            .expect("Couldn't find a suitable graphics queue family")
            as u32;

        let (device, mut queues) = Device::new(
            physical_device,
            DeviceCreateInfo {
                queue_create_infos: vec![QueueCreateInfo {
                    queue_family_index,
                    ..Default::default()
                }],
                ..Default::default()
            },
        )
        .expect("Failed to create device");
        // Get queue we requested
        let queue = queues.next().unwrap();

        // Create allocator instance
        let allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));
        // Create descriptor set allocator instance
        let dset_allocator = Arc::new(StandardDescriptorSetAllocator::new(
            device.clone(),
            Default::default(),
        ));
        // Create command buffer allocator instance
        let cmd_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(
            device.clone(),
            Default::default(),
        ));

        VulkanContext {
            instance,
            device,
            queue,
            allocator,
            dset_allocator,
            cmd_buffer_allocator,
        }
    }

    pub fn render_once<F: FnOnce(&mut CmdBufferType)>(&self, f: F) {
        // Create command buffer
        let mut command_buffer_builder = AutoCommandBufferBuilder::primary(
            &self.cmd_buffer_allocator,
            self.queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        // Create render work for submition
        f(&mut command_buffer_builder);

        // Submit work to device
        let cmd_buffer = command_buffer_builder.build().unwrap();
        let future = sync::now(self.device.clone())
            .then_execute(self.queue.clone(), cmd_buffer)
            .unwrap()
            .then_signal_fence_and_flush()
            .unwrap();
        // Wait for GPU to finish execution
        future.wait(None).unwrap();
    }
}
