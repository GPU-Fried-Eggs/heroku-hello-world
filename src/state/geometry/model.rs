use std::ops::Range;
use bytemuck::{Pod, Zeroable};
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindingResource, BufferAddress,
    Device, RenderPass, VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode
};
use crate::state::texture::Texture;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub(in crate::state) struct ModelVertex {
    pub position:       [f32; 3],
    pub uv:             [f32; 2],
    pub normal:         [f32; 3],
    pub tangent:        [f32; 3],
    pub bitangent:      [f32; 3],
}

impl ModelVertex {
    pub(in crate::state) fn new() -> Self {
        Self {
            position:   [0.0; 3],
            uv:         [0.0; 2],
            normal:     [0.0; 3],
            tangent:    [0.0; 3],
            bitangent:  [0.0; 3],
        }
    }

    pub(in crate::state) fn desc<'a>() -> VertexBufferLayout<'a> {
        use std::mem;
        VertexBufferLayout {
            array_stride: mem::size_of::<ModelVertex>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &[
                VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: VertexFormat::Float32x3,
                },
                VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as BufferAddress,
                    shader_location: 1,
                    format: VertexFormat::Float32x2,
                },
                VertexAttribute {
                    offset: mem::size_of::<[f32; 5]>() as BufferAddress,
                    shader_location: 2,
                    format: VertexFormat::Float32x3,
                },
                // Tangent and bitangent
                VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as BufferAddress,
                    shader_location: 3,
                    format: VertexFormat::Float32x3,
                },
                VertexAttribute {
                    offset: mem::size_of::<[f32; 11]>() as BufferAddress,
                    shader_location: 4,
                    format: VertexFormat::Float32x3,
                },
            ],
        }
    }
}

pub(in crate::state) struct Material {
    pub(in crate::state) name: String,
    pub(in crate::state) diffuse_texture: Texture,
    pub(in crate::state) normal_texture: Texture,
    pub(in crate::state) bind_group: BindGroup,
}

impl Material {
    pub(in crate::state) fn new(device: &Device, name: &str, diffuse_texture: Texture, normal_texture: Texture, layout: &wgpu::BindGroupLayout) -> Self {
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&diffuse_texture.view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&diffuse_texture.sampler),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::TextureView(&normal_texture.view),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: BindingResource::Sampler(&normal_texture.sampler),
                },
            ],
            label: Some(name),
        });

        Self {
            name: String::from(name),
            diffuse_texture,
            normal_texture,
            bind_group,
        }
    }
}

pub(in crate::state) struct Mesh {
    pub(in crate::state) name: String,
    pub(in crate::state) vertex_buffer: wgpu::Buffer,
    pub(in crate::state) index_buffer: wgpu::Buffer,
    pub(in crate::state) num_elements: u32,
    pub(in crate::state) material: usize,
}

pub(in crate::state) struct Model {
    pub(in crate::state) meshes: Vec<Mesh>,
    pub(in crate::state) materials: Vec<Material>,
}

pub(in crate::state) trait DrawModel<'a> {
    fn draw_mesh(&mut self, mesh: &'a Mesh, material: &'a Material, camera_bind_group: &'a BindGroup, light_bind_group: &'a BindGroup);

    fn draw_mesh_instanced(&mut self, mesh: &'a Mesh, material: &'a Material, instances: Range<u32>, camera_bind_group: &'a BindGroup, light_bind_group: &'a BindGroup);

    fn draw_model(&mut self, model: &'a Model, camera_bind_group: &'a BindGroup, light_bind_group: &'a BindGroup);

    fn draw_model_instanced(&mut self, model: &'a Model, instances: Range<u32>, camera_bind_group: &'a BindGroup, light_bind_group: &'a BindGroup);

    fn draw_model_instanced_with_material(&mut self, model: &'a Model, material: &'a Material, instances: Range<u32>, camera_bind_group: &'a BindGroup, light_bind_group: &'a BindGroup);
}

impl<'a, 'b> DrawModel<'b> for RenderPass<'a> where 'b: 'a {
    fn draw_mesh(&mut self, mesh: &'b Mesh, material: &'b Material, camera_bind_group: &'b BindGroup, light_bind_group: &'b BindGroup) {
        self.draw_mesh_instanced(mesh, material, 0..1, camera_bind_group, light_bind_group);
    }

    fn draw_mesh_instanced(&mut self, mesh: &'b Mesh, material: &'b Material, instances: Range<u32>, camera_bind_group: &'b BindGroup, light_bind_group: &'b BindGroup) {
        self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        self.set_bind_group(0, &material.bind_group, &[]);
        self.set_bind_group(1, camera_bind_group, &[]);
        self.set_bind_group(2, light_bind_group, &[]);
        self.draw_indexed(0..mesh.num_elements, 0, instances);
    }

    fn draw_model(&mut self, model: &'b Model, camera_bind_group: &'b BindGroup, light_bind_group: &'b BindGroup) {
        self.draw_model_instanced(model, 0..1, camera_bind_group, light_bind_group);
    }

    fn draw_model_instanced(&mut self, model: &'b Model, instances: Range<u32>, camera_bind_group: &'b BindGroup, light_bind_group: &'b BindGroup) {
        for mesh in &model.meshes {
            let material = &model.materials[mesh.material];
            self.draw_mesh_instanced(mesh, material, instances.clone(), camera_bind_group, light_bind_group);
        }
    }

    fn draw_model_instanced_with_material(&mut self, model: &'b Model, material: &'b Material, instances: Range<u32>, camera_bind_group: &'b BindGroup, light_bind_group: &'b BindGroup) {
        for mesh in &model.meshes {
            self.draw_mesh_instanced(mesh, material, instances.clone(), camera_bind_group, light_bind_group);
        }
    }
}