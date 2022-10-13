use bytemuck::{Pod, Zeroable};
use winit::dpi::PhysicalSize;
use crate::state::time::Instant;

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, Pod, Zeroable)]
pub(in crate::state) struct SystemUniform {
    time: f32,
    _padding_01_: [u32; 3],
    resolution: [u32; 2],
    _padding_02_: [u32; 2],
}

impl SystemUniform {
    pub(in crate::state) fn new(resolution: PhysicalSize<u32>, start_time: Instant) -> Self {
        Self {
            time: start_time.elapsed().as_secs_f32(),
            _padding_01_: [0; 3],
            resolution: [resolution.width, resolution.height],
            _padding_02_: [0; 2],
        }
    }

    pub(in crate::state) fn update_system(&mut self, resolution: PhysicalSize<u32>, start_time: Instant) {
        // update time to number of milliseconds since program start
        self.time = start_time.elapsed().as_secs_f32();
        self.resolution = [resolution.width, resolution.height];
    }
}