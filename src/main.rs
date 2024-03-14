mod fractal_pass;
mod test_pass;

mod context;

use image::{ImageBuffer, Rgba};
use test_pass::TestPass;

use crate::context::VulkanContext;
use crate::fractal_pass::FractalPass;

fn main() {
    // Initialize vulkan context
    let context = VulkanContext::new();
    let test_pass = FractalPass::new(&context);
    // let blur_pass = BlurPass::new(&context);

    context.render_once(|cmd_buffer| {
        test_pass.execute(cmd_buffer);
    });

    let buffer_content = test_pass.transfer_buffer.read().unwrap();
    // buffer_content.iter().map(|e| e as u8).collect();
    let aux_vec = buffer_content.iter().map(|e| *e as u8).collect::<Vec<u8>>();
    let image = ImageBuffer::<Rgba<u8>, _>::from_raw(1024, 1024, aux_vec).unwrap();

    // let image = ImageBuffer::<Rgba<u32>, _>::from_raw(1024, 1024, &buffer_content[..]).unwrap();
    image.save("result.png").unwrap();
}
