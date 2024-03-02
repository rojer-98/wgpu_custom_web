use winit::event::WindowEvent;

pub trait Component<const N: usize, T: bytemuck::Zeroable + bytemuck::Pod> {
    fn data(&self) -> [T; N];
    fn update(&mut self, event: &WindowEvent);
}

pub trait Object {}
