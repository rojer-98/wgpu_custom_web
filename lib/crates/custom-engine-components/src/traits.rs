use winit::event::WindowEvent;

pub trait Component<T: bytemuck::Zeroable + bytemuck::Pod> {
    fn data(&self) -> T;
    fn update(&mut self, event: &WindowEvent);
}

pub trait Object {}
