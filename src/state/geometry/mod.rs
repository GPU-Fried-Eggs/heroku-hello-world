pub(super) mod quad;
pub(super) mod model;

use bytemuck::Pod;
use wgpu::{Buffer, BufferUsages, Device, util::{DeviceExt, BufferInitDescriptor}};

#[derive(Debug)]
pub(super) struct VertexBinding {
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    num_indices: u32,
}

impl VertexBinding {
    pub(super) fn vertex_buffer(&self) -> &Buffer {
        &self.vertex_buffer
    }

    pub(super) fn index_buffer(&self) -> &Buffer {
        &self.index_buffer
    }

    pub(super) fn num_indices(&self) -> u32 {
        self.num_indices
    }
}

pub(super) trait Vertex {
    fn get_vertices(&self) -> &'static [u8];

    fn get_indices(&self) -> &'static [u8];

    fn get_indices_number(&self) -> u32;

    fn make_binding(self, device: &Device) -> VertexBinding where Self: Sized + Pod {
        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: self.get_vertices(),
            usage: BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: self.get_indices(),
            usage: BufferUsages::INDEX,
        });
        let num_indices = self.get_indices_number();
        VertexBinding {
            vertex_buffer,
            index_buffer,
            num_indices
        }
    }
}

impl Vertex for quad::QuadVertex {
    fn get_vertices(&self) -> &'static [u8] {
        bytemuck::cast_slice(quad::VERTICES)
    }

    fn get_indices(&self) -> &'static [u8] {
        bytemuck::cast_slice(quad::INDICES)
    }

    fn get_indices_number(&self) -> u32 {
        quad::INDICES.len() as u32
    }
}

impl Vertex for model::ModelVertex {
    fn get_vertices(&self) -> &'static [u8] {
        todo!()
    }

    fn get_indices(&self) -> &'static [u8] {
        todo!()
    }

    fn get_indices_number(&self) -> u32 {
        todo!()
    }
}
