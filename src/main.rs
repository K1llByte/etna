mod blur_pass;
mod test_pass;

mod context;

use image::{ImageBuffer, Rgba};

use crate::blur_pass::BlurPass;
use crate::context::VulkanContext;

fn main() {
    // Initialize vulkan context
    let context = VulkanContext::new();
    let blur_pass = BlurPass::new(&context);

    context.render_once(|cmd_buffer| {
        blur_pass.execute(cmd_buffer);
    });

    let buffer_content = blur_pass.transfer_buffer.read().unwrap();
    let image = ImageBuffer::<Rgba<u8>, _>::from_raw(1024, 1024, &buffer_content[..]).unwrap();
    image.save("result.png").unwrap();

    println!("Ok!");
}
