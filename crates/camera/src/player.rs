
use cgmath::{Matrix4, Rad, Vector3};

/// PlayerCamera struct
/// Camera object that responds to user input - namely forward, backwards, left and right. Uses
/// a momentum mechanic such that it accelerates to a maximum speed over time and also decelerates
/// over time. The momentum mechanic applies to both linear and angular velocities.
pub struct PlayerCamera {
    speed: f32,
    angular_speed: f32,
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
            speed: 0.0,
            angular_speed: 0.0,
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

    /// Move the camera as per the up/down/left/right inputs in the supplied controller
    pub fn update(&mut self, time_step_millis: u64, dx: f32, dy: f32) {

        let time_step_secs: f32 = 0.001 * time_step_millis as f32;

        // Update angular speed
        self.angular_speed = {
            let deadzone: f32 = 0.01;
            let max_speed: f32 = 3.0;
            let accel: f32 = 4.0;
            let decel: f32 = 10.0;

            if self.angular_speed == 0.0 {
                let unclamped_speed = self.angular_speed - accel * time_step_secs * dx;
                unclamped_speed.min(max_speed).max(-max_speed)
            } else if self.angular_speed > 0.0 {
                if dx > -deadzone {
                    (self.angular_speed - decel * time_step_secs).max(0.0)
                } else {
                    let unclamped_speed = self.angular_speed - accel * time_step_secs * dx;
                    unclamped_speed.min(max_speed).max(-max_speed)
                }
            } else if dx < deadzone {
                (self.angular_speed + decel * time_step_secs).min(0.0)
            } else {
                let unclamped_speed = self.angular_speed - accel * time_step_secs * dx;
                unclamped_speed.min(max_speed).max(-max_speed)
            }
        };

        self.rotation += self.angular_speed * time_step_secs;
        if self.rotation > 2.0 * std::f32::consts::PI {
            self.rotation -= 2.0 * std::f32::consts::PI;
        }
        if self.rotation < -2.0 * std::f32::consts::PI {
            self.rotation += 2.0 * std::f32::consts::PI;
        }

        // Update linear speed
        self.speed = {
            let deadzone: f32 = 0.01;
            let max_speed: f32 = 8.0;
            let max_reverse_speed = -3.0;
            let accel: f32 = 9.0;
            let decel: f32 = 25.0;

            if self.speed == 0.0 {
                let unclamped_speed = self.speed + accel * time_step_secs * dy;
                unclamped_speed.min(max_speed).max(max_reverse_speed)
            } else if self.speed > 0.0 {
                if dy < deadzone {
                    (self.speed - decel * time_step_secs).max(0.0)
                } else {
                    let unclamped_speed = self.speed + accel * time_step_secs * dy;
                    unclamped_speed.min(max_speed).max(max_reverse_speed)
                }
            } else if dy > -deadzone {
                (self.speed + decel * time_step_secs).min(0.0)
            } else {
                let unclamped_speed = self.speed + accel * time_step_secs * dy;
                unclamped_speed.min(max_speed).max(max_reverse_speed)
            }
        };

        self.position_x -= self.speed * time_step_secs * self.rotation.sin();
        self.position_z += self.speed * time_step_secs * self.rotation.cos();
    }
}
