use bytemuck::Pod;
use wgpu::{util::DeviceExt, *};
use super::{MouseUniform, SystemUniform};

#[derive(Debug)]
pub(in crate::state) struct UniformBinding<T> {
    uniform: T,
    buffer: Buffer,
    bind_group: BindGroup
}

impl<T> UniformBinding<T> {
    pub(in crate::state) fn uniform(&self) -> &T {
        &self.uniform
    }

    pub(in crate::state) fn uniform_mut(&mut self) -> &mut T {
        &mut self.uniform
    }

    pub(in crate::state) fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    pub(in crate::state) fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }
}

pub(in crate::state) trait Uniform {
    fn get_buffer_label(&self) -> &'static str;

    fn get_bind_group_label(&self) -> &'static str;

    fn make_binding(self, device: &Device, bind_group_layout: &BindGroupLayout) -> UniformBinding<Self> where Self: Sized + Pod
    {
        let buffer = device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some(self.get_buffer_label()),
            contents: bytemuck::cast_slice(&[self]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some(self.get_bind_group_label()),
            layout: bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });
        UniformBinding::<Self> {
            uniform: self,
            buffer,
            bind_group,
        }
    }
}

impl Uniform for SystemUniform {
    fn get_buffer_label(&self) -> &'static str {
        "System Buffer"
    }

    fn get_bind_group_label(&self) -> &'static str {
        "System Bind Group"
    }
}

impl Uniform for MouseUniform {
    fn get_buffer_label(&self) -> &'static str {
        "Mouse Bind Group"
    }

    fn get_bind_group_label(&self) -> &'static str {
        "Mouse Buffer"
    }
}
