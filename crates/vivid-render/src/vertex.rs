use vivid_core::glam::Vec2;

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct Vertex {
    pub position: Vec2,
}

impl Vertex {
    pub(crate) const LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: size_of::<Vertex>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &wgpu::vertex_attr_array![0 => Float32x2],
    };
}

pub(crate) const QUAD: &[Vertex] = &[
    Vertex {
        position: Vec2::new(-0.5, -0.5),
    },
    Vertex {
        position: Vec2::new(0.5, -0.5),
    },
    Vertex {
        position: Vec2::new(0.5, 0.5),
    },
    Vertex {
        position: Vec2::new(-0.5, 0.5),
    },
];

pub(crate) const QUAD_INDICES: &[u16] = &[0, 1, 2, 0, 2, 3];

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct Instance {
    pub offset: Vec2,
}

impl Instance {
    pub(crate) const LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: size_of::<Instance>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Instance,
        attributes: &wgpu::vertex_attr_array![1 => Float32x2],
    };
}
