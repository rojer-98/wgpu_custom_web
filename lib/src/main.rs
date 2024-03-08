use pollster::block_on;

fn main() {
    block_on(async { wgpu_custom_engine::run().await });
}
