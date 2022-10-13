use std::ops::Range;
use bytemuck::{Pod, Zeroable};
use wgpu::{
    BufferAddress, IndexFormat, RenderPass, vertex_attr_array,
    VertexAttribute, VertexBufferLayout, VertexStepMode
};
use super::VertexBinding;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub(in crate::state) struct QuadVertex {
    pub position:       [f32; 3],
    pub uv:             [f32; 2],
}

impl QuadVertex {
    const ATTRIBS: [VertexAttribute; 2] = vertex_attr_array![0 => Float32x3, 1 => Float32x2];

    pub(in crate::state) fn new() -> Self {
        Self {
            position:   [0.0; 3],
            uv:         [0.0; 2],
        }
    }

    pub(in crate::state) fn desc<'pipeline>() -> VertexBufferLayout<'pipeline> {
        use std::mem;
        VertexBufferLayout {
            array_stride: mem::size_of::<QuadVertex>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

// square, a quad's corners
pub(in crate::state) const VERTICES: &[QuadVertex] = &[
    QuadVertex { position: [-1.0, -1.0, 0.0], uv: [0.0, 0.0] }, // Top left
    QuadVertex { position: [ 1.0, -1.0, 0.0], uv: [1.0, 0.0] }, // Top right
    QuadVertex { position: [ 1.0,  1.0, 0.0], uv: [1.0, 1.0] }, // Bottom left
    QuadVertex { position: [-1.0,  1.0, 0.0], uv: [0.0, 1.0] }, // Bottom right
];

// a simple quad shape
pub(in crate::state) const INDICES: &[u16] = &[2, 3, 0, 1, 2, 0];

pub(in crate::state) trait DrawQuad<'a> {
    fn draw_mesh(&mut self, vertex_binding: &'a VertexBinding);

    fn draw_mesh_instanced(&mut self, instances: Range<u32>, vertex_binding: &'a VertexBinding);
}

impl<'a, 'b> DrawQuad<'b> for RenderPass<'a> where 'b: 'a {
    fn draw_mesh(&mut self, vertex_binding: &'b VertexBinding) {
        self.draw_mesh_instanced(0..1, vertex_binding);
    }

    fn draw_mesh_instanced(&mut self, instances: Range<u32>, vertex_binding: &'b VertexBinding) {
        self.set_vertex_buffer(0, vertex_binding.vertex_buffer().slice(..));
        self.set_index_buffer(vertex_binding.index_buffer().slice(..), IndexFormat::Uint16);
        self.draw_indexed(0..vertex_binding.num_indices(), 0, instances);
    }
}