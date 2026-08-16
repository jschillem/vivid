use vivid_core::glam::{Vec2, Vec3};

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
}

impl Vertex {
    pub(crate) const LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: size_of::<Vertex>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3],
    };
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct Instance {
    pub offset: Vec3,
}

impl Instance {
    pub(crate) const LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: size_of::<Instance>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Instance,
        attributes: &wgpu::vertex_attr_array![2 => Float32x3],
    };
}

/// Per-face basis: (normal, u axis, v axis), chosen so `u cross v == normal`.
/// That relationship is what guarantees counter-clockwise winding seen from
/// OUTSIDE the cube, which is what `FrontFace::Ccw` + back-face culling wants.
/// If a face turns out invisible, swap its u and v.
const FACES: [(Vec3, Vec3, Vec3); 6] = [
    (Vec3::Z, Vec3::X, Vec3::Y),         // front  (+Z)
    (Vec3::NEG_Z, Vec3::NEG_X, Vec3::Y), // back   (-Z)
    (Vec3::X, Vec3::NEG_Z, Vec3::Y),     // right  (+X)
    (Vec3::NEG_X, Vec3::Z, Vec3::Y),     // left   (-X)
    (Vec3::Y, Vec3::X, Vec3::NEG_Z),     // top    (+Y)
    (Vec3::NEG_Y, Vec3::X, Vec3::Z),     // bottom (-Y)
];

/// Unit cube centered on the origin, 1 unit per side.
/// Each face: 4 vertices sharing one normal, 2 CCW triangles when viewed
/// from outside.
pub(crate) fn cube() -> (Vec<Vertex>, Vec<u16>) {
    let mut vertices = Vec::with_capacity(24);
    let mut indices = Vec::with_capacity(36);

    for (normal, u, v) in FACES {
        let base = vertices.len() as u16;
        let center = normal * 0.5;
        // Corners in CCW order seen from outside: (-u,-v), (+u,-v), (+u,+v), (-u,+v)
        for (su, sv) in [(-0.5, -0.5), (0.5, -0.5), (0.5, 0.5), (-0.5, 0.5)] {
            vertices.push(Vertex {
                position: center + u * su + v * sv,
                normal,
            });
        }
        indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }
    (vertices, indices)
}
