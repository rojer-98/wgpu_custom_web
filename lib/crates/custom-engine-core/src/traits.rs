use std::{ops::Sub, path::PathBuf, time::Duration};

use crate::{errors::CoreError, worker::Worker};

use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{
        AxisId, DeviceId, ElementState, Ime, InnerSizeWriter, KeyEvent, Modifiers, MouseButton,
        MouseScrollDelta, Touch, TouchPhase, WindowEvent,
    },
    window::Theme,
};

pub trait VertexLayout
where
    Self: bytemuck::Zeroable + bytemuck::Pod,
{
    const ATTRIBUTES: &'static [wgpu::VertexAttribute];

    fn desc() -> wgpu::VertexBufferLayout<'static>;
    fn to_bytes(&self) -> Vec<u8> {
        bytemuck::cast_vec(vec![*self])
    }
}

pub trait Builder<'a> {
    type Final;

    fn new(device: &'a wgpu::Device) -> Self
    where
        Self: Sized;
    fn new_indexed(device: &'a wgpu::Device, id: usize) -> Self
    where
        Self: Sized;

    fn build(self) -> Result<Self::Final, CoreError>
    where
        Self: Sized;
}

pub trait RenderWorker {
    fn new() -> Self
    where
        Self: Sized;
    fn init(&mut self, _: &mut Worker<'_>) -> Result<(), CoreError>;

    fn render(&mut self, _: &mut Worker<'_>) -> Result<(), CoreError> {
        Ok(())
    }
    fn update(
        &mut self,
        _: &mut Worker<'_>,
        _: &WindowEvent,
        _: Duration,
    ) -> Result<(), CoreError> {
        Ok(())
    }
    fn resize(&mut self, _: &mut Worker<'_>) -> Result<(), CoreError> {
        Ok(())
    }
}

pub trait OnEvent {
    fn on_event(&self);
}

pub trait EventHandler<R: RenderWorker>: Default {
    fn on_resize<S: Sub>(
        &mut self,
        _: &mut R,
        _: &mut Worker,
        _: PhysicalSize<S>,
    ) -> Result<(), CoreError> {
        Ok(())
    }
    fn on_focused(&mut self, _: &mut R, _: &mut Worker, _: bool) -> Result<(), CoreError> {
        Ok(())
    }
    fn on_scale_factor_changed(
        &mut self,
        _: &mut R,
        _: &mut Worker,
        _: f64,
        _: InnerSizeWriter,
    ) -> Result<(), CoreError> {
        Ok(())
    }
    fn on_theme(&mut self, _: &mut R, _: &mut Worker, _: Theme) -> Result<(), CoreError> {
        Ok(())
    }
    fn on_occluded(&mut self, _: &mut R, _: &mut Worker, _: bool) -> Result<(), CoreError> {
        Ok(())
    }
    fn on_modifiers_changed(
        &mut self,
        _: &mut R,
        _: &mut Worker,
        _: Modifiers,
    ) -> Result<(), CoreError> {
        Ok(())
    }
    fn on_mouse_wheel(
        &mut self,
        _: &mut R,
        _: &mut Worker,
        _: DeviceId,
        _: MouseScrollDelta,
        _: TouchPhase,
    ) -> Result<(), CoreError> {
        Ok(())
    }
    fn on_keyboard_input(
        &mut self,
        _: &mut R,
        _: &mut Worker,
        _: DeviceId,
        _: KeyEvent,
        _: bool,
    ) -> Result<(), CoreError> {
        Ok(())
    }
    fn on_mouse_input(
        &mut self,
        _: &mut R,
        _: &mut Worker,
        _: DeviceId,
        _: ElementState,
        _: MouseButton,
    ) -> Result<(), CoreError> {
        Ok(())
    }
    fn on_cursor_left(&mut self, _: &mut R, _: &mut Worker, _: DeviceId) -> Result<(), CoreError> {
        Ok(())
    }
    fn on_cursor_moved<S: Sub>(
        &mut self,
        _: &mut R,
        _: &mut Worker,
        _: DeviceId,
        _: PhysicalPosition<S>,
    ) -> Result<(), CoreError> {
        Ok(())
    }
    fn on_ime(&mut self, _: &mut R, _: &mut Worker, _: Ime) -> Result<(), CoreError> {
        Ok(())
    }
    fn on_pinch_gesture(
        &mut self,
        _: &mut R,
        _: &mut Worker,
        _: DeviceId,
        _: f64,
        _: TouchPhase,
    ) -> Result<(), CoreError> {
        Ok(())
    }
    fn on_rotation_gesture<S: Sub>(
        &mut self,
        _: &mut R,
        _: &mut Worker,
        _: DeviceId,
        _: S,
        _: TouchPhase,
    ) -> Result<(), CoreError> {
        Ok(())
    }
    fn on_pan_gesture<S: Sub>(
        &mut self,
        _: &mut R,
        _: &mut Worker,
        _: DeviceId,
        _: PhysicalPosition<S>,
        _: TouchPhase,
    ) -> Result<(), CoreError> {
        Ok(())
    }
    fn on_double_tap_gesture(
        &mut self,
        _: &mut R,
        _: &mut Worker,
        _: DeviceId,
    ) -> Result<(), CoreError> {
        Ok(())
    }
    fn on_touchpad_pressure<S: Sub>(
        &mut self,
        _: &mut R,
        _: &mut Worker,
        _: DeviceId,
        _: S,
        _: i64,
    ) -> Result<(), CoreError> {
        Ok(())
    }
    fn on_hovered_file_cancelled(&mut self, _: &mut R, _: &mut Worker) -> Result<(), CoreError> {
        Ok(())
    }
    fn on_cursor_entered(
        &mut self,
        _: &mut R,
        _: &mut Worker,
        _: DeviceId,
    ) -> Result<(), CoreError> {
        Ok(())
    }
    fn on_axis_motion(
        &mut self,
        _: &mut R,
        _: &mut Worker,
        _: DeviceId,
        _: AxisId,
        _: f64,
    ) -> Result<(), CoreError> {
        Ok(())
    }
    fn on_dropped_file(&mut self, _: &mut R, _: &mut Worker, _: PathBuf) -> Result<(), CoreError> {
        Ok(())
    }
    fn on_hovered_file(&mut self, _: &mut R, _: &mut Worker, _: PathBuf) -> Result<(), CoreError> {
        Ok(())
    }
    fn on_destroyed(&mut self, _: &mut R, _: &mut Worker) -> Result<(), CoreError> {
        Ok(())
    }
    fn on_touch(&mut self, _: &mut R, _: &mut Worker, _: Touch) -> Result<(), CoreError> {
        Ok(())
    }
    fn on_moved<S: Sub>(
        &mut self,
        _: &mut R,
        _: &mut Worker,
        _: PhysicalPosition<S>,
    ) -> Result<(), CoreError> {
        Ok(())
    }
}

/*
trait Block {
    fn wait(self) -> <Self as futures::Future>::Output
        where Self: Sized, Self: futures::Future
    {
        futures::executor::block_on(self)
    }
}

impl<F,T> Block for F
    where F: futures::Future<Output = T>
{}

*/
