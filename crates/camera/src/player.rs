
use cgmath::{Matrix4, Rad, Vector3};

/// PlayerCamera struct
/// Camera object that sits in a requested position and never moves
pub struct PlayerCamera {
    rotation: f32,
    position_x: f32,
    position_y: f32,
    position_z: f32,
    perspective_projection: Matrix4<f32>
}

impl PlayerCamera {

    /// Constant near and far plane distances used for the perspective projection
    const NEAR_PLANE: f32 = 1.0;
    const FAR_PLANE: f32 = 100.0;

    /// Creates a new camera with zero speed and oriented at the supplied angle
    pub fn new(x: f32, y: f32, z: f32, angle_rad: f32) -> PlayerCamera {
        let aspect_ratio = 1.0;
        PlayerCamera {
            rotation: angle_rad,
            position_x: x,
            position_y: y,
            position_z: z,
            perspective_projection: Self::make_vulkan_perspective_matrix(
                aspect_ratio,
                Self::NEAR_PLANE,
                Self::FAR_PLANE)
        }
    }

    /// Creates a projection matrix suitable for Vulkan. Note that OpenGL, DirectX, etc may need
    /// alternate implementations due to differing up/down coordinates or clip volumes.
    fn make_vulkan_perspective_matrix(
        aspect_ratio: f32,
        near_plane: f32,
        far_plane: f32
    ) -> Matrix4<f32> {
        let half_width = aspect_ratio;
        let half_height = 1.0;
        Matrix4::<f32>::new(
            near_plane / half_width, 0.0, 0.0, 0.0,
            0.0, near_plane / half_height, 0.0, 0.0,
            0.0, 0.0, far_plane / (far_plane - near_plane), 1.0,
            0.0, 0.0, (-far_plane * near_plane) / (far_plane - near_plane), 0.0
        )
    }

    /// Get the view matrix, based on the camera's position and orientation
    pub fn get_view_matrix(&self) -> Matrix4<f32> {
        let rotation = Matrix4::from_angle_y(Rad(self.rotation));
        let translation = Matrix4::<f32>::from_translation(
            Vector3::<f32> { x: -self.position_x, y: -self.position_y, z: -self.position_z }
        );
        rotation * translation
    }

    /// Get the stored perspective projection matrix
    pub fn get_projection_matrix(&self) -> Matrix4<f32> {
        self.perspective_projection
    }
}
