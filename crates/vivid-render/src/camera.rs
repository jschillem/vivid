use vivid_core::glam::{Mat4, Vec2, Vec3};

#[derive(Debug)]
pub struct Camera {
    pub center: Vec2,
    pub height: f32,
    buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    layout: wgpu::BindGroupLayout,
}

impl Camera {
    pub(crate) fn new(device: &wgpu::Device) -> Self {
        use wgpu::util::DeviceExt;

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("camera-uniform"),
            contents: bytemuck::cast_slice(&[CameraUniform {
                view_proj: Mat4::IDENTITY,
            }]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("camera-bind-group-layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("camera-bind-group"),
            layout: &layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });

        Self {
            center: Vec2::ZERO,
            height: 20.0,
            buffer,
            bind_group,
            layout,
        }
    }

    pub(crate) fn layout(&self) -> &wgpu::BindGroupLayout {
        &self.layout
    }

    pub(crate) fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    pub(crate) fn upload(&self, queue: &wgpu::Queue, aspect: f32) {
        let uniform = CameraUniform {
            view_proj: view_proj(self.center, self.height, aspect),
        };

        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[uniform]));
    }

    pub fn world_to_clip(&self, point: Vec2, aspect: f32) -> Vec2 {
        let clip = view_proj(self.center, self.height, aspect)
            .transform_point3(Vec3::new(point.x, point.y, 0.0));

        Vec2::new(clip.x, clip.y)
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    view_proj: Mat4,
}

fn view_proj(center: Vec2, height: f32, aspect: f32) -> Mat4 {
    let half_h = height * 0.5;
    let half_w = half_h * aspect;

    vivid_core::glam::camera::rh::proj::directx::orthographic(
        center.x - half_w,
        center.x + half_w,
        center.y - half_h,
        center.y + half_h,
        -1.0,
        1.0,
    )
}

#[cfg(test)]
mod tests {
    use approx::assert_abs_diff_eq;
    use vivid_core::glam::Vec3;

    use super::*;

    const EPS: f32 = 1e-6;

    #[test]
    fn center_maps_to_clip_origin() {
        let center = Vec2::new(37.0, -12.0);
        let m = view_proj(center, 20.0, 1.6);
        let clip = m.transform_point3(Vec3::new(center.x, center.y, 0.0));
        assert_abs_diff_eq!(clip.x, 0.0, epsilon = EPS);
        assert_abs_diff_eq!(clip.y, 0.0, epsilon = EPS);
    }

    #[test]
    fn view_corners_map_to_clip_corners() {
        let (height, aspect) = (20.0, 2.0);
        let m = view_proj(Vec2::ZERO, height, aspect);

        // half_h = 10, half_w = half_h * aspect = 20.
        // Top-right of the visible world: (half_w, half_h) -> (+1, +1).
        let corner = m.transform_point3(Vec3::new(20.0, 10.0, 0.0));
        assert_abs_diff_eq!(corner.x, 1.0, epsilon = EPS);
        assert_abs_diff_eq!(corner.y, 1.0, epsilon = EPS);
    }

    #[test]
    fn aspect_widens_view_without_distorting() {
        // The same world point sits proportionally closer to the clip origin
        // on a wider surface — more world visible, same world-space size.
        let narrow = view_proj(Vec2::ZERO, 20.0, 1.0);
        let wide = view_proj(Vec2::ZERO, 20.0, 2.0);
        let p = Vec3::new(5.0, 0.0, 0.0);

        let x_narrow = narrow.transform_point3(p).x;
        let x_wide = wide.transform_point3(p).x;
        assert_abs_diff_eq!(x_wide, x_narrow / 2.0, epsilon = EPS);
    }
}
