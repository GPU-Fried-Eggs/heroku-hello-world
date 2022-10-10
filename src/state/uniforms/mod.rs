use bytemuck::{Pod, Zeroable};
use winit::dpi::PhysicalSize;
use super::time::Instant;

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, Pod, Zeroable)]
pub(super) struct SystemUniform {
    time: f32,
    _padding_01_: [u32; 3],
    resolution: [u32; 2],
    _padding_02_: [u32; 2]
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, Pod, Zeroable)]
pub(super) struct MouseUniform {
    cursor_pos: [f32; 2]
}

impl SystemUniform {
    pub(super) fn new(resolution: PhysicalSize<u32>, start_time: Instant) -> Self {
        let elapsed = start_time.elapsed();

        Self {
            time: elapsed.as_secs_f32(),
            _padding_01_: [0, 0, 0],
            resolution: [resolution.width, resolution.height],
            _padding_02_: [0, 0]
        }
    }

    pub(super) fn update(&mut self, resolution: PhysicalSize<u32>, start_time: Instant) {
        // update time to number of milliseconds since program start
        self.time = start_time.elapsed().as_secs_f32();
        self.resolution = [resolution.width, resolution.height];
    }
}

impl MouseUniform {
    pub(super) fn new() -> Self {
        Self {
            cursor_pos: [0.0, 0.0]
        }
    }

    pub(super) fn update_position(&mut self, x: f32, y: f32) {
        // update cursor position
        // y axis is reversed from GPU coords
        self.cursor_pos = [x, 1.0 - y];
    }
}

pub(super) mod bindings;
