#![allow(unstable_name_collisions)]

use bevy::input::mouse::MouseMotion;
use bevy::input::mouse::MouseScrollUnit::{Line, Pixel};
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use bevy::render::camera::Camera;

const LINE_TO_PIXEL_RATIO: f32 = 0.1;

pub trait Clamp: Sized {
    fn clamp<L, U>(self, lower: L, upper: U) -> Self
    where
        L: Into<Option<Self>>,
        U: Into<Option<Self>>;
}

impl Clamp for f32 {
    fn clamp<L, U>(self, lower: L, upper: U) -> Self
    where
        L: Into<Option<Self>>,
        U: Into<Option<Self>>,
    {
        let below = match lower.into() {
            None => self,
            Some(lower) => self.max(lower),
        };
        match upper.into() {
            None => below,
            Some(upper) => below.min(upper),
        }
    }
}
 
#[derive(Default)]
struct State {
    motion: EventReader<MouseMotion>,
    scroll: EventReader<MouseWheel>,
}

pub struct OrbitCamera {
    pub x: f32,
    pub y: f32,
    pub distance: f32,
    pub center: Vec3,
    pub rotate_sensitivity: f32,
    pub zoom_sensitivity: f32,
    pub pan_sensitivity: f32,
    pub max_zoom_distance: f32,
    pub min_zoom_distance: f32,
    pub max_polar_angle: f32,
    pub min_polar_angle: f32,
}

impl Default for OrbitCamera {
    fn default() -> Self {
        OrbitCamera {
            x: 0.0,
            y: 0.0,
            distance: 5.0,
            center: Vec3::zero(),
            rotate_sensitivity: 1.0,
            zoom_sensitivity: 0.8,
            pan_sensitivity: 1.0,
            max_zoom_distance: -1.0,
            min_zoom_distance: -1.0,
            max_polar_angle: 3.13,
            min_polar_angle: 0.01,
        }
    }
}

impl OrbitCamera {
    pub fn new(x: f32, y: f32, dist: f32, center: Vec3) -> OrbitCamera {
        OrbitCamera {
            x: x,
            y: y,
            distance: dist,
            center: center,
            rotate_sensitivity: 1.0,
            zoom_sensitivity: 0.8,
            pan_sensitivity: 1.0,
            max_zoom_distance: 120.0,
            min_zoom_distance: 8.0,
            max_polar_angle: 3.13,
            min_polar_angle: 0.01,
        }
    }
}

pub struct OrbitCameraPlugin;
impl OrbitCameraPlugin {
    fn mouse_motion_system(
        time: Res<Time>,
        mut state: ResMut<State>,
        mouse_motion_events: Res<Events<MouseMotion>>,
        mouse_button_input: Res<Input<MouseButton>>,
        keyboard_input: Res<Input<KeyCode>>,
        mut query: Query<(&mut OrbitCamera, &mut Transform, &mut Camera)>,
    ) {
        let mut delta = Vec2::zero();
        for event in state.motion.iter(&mouse_motion_events) {
            delta += event.delta;
        }
        for (mut camera, mut transform, _) in query.iter_mut() {
            // Shift + LMB = Drag
            if keyboard_input.pressed(KeyCode::LShift) {
                if mouse_button_input.pressed(MouseButton::Left) {
                    let camera_translation = Vec3::new(
                        delta.x * camera.pan_sensitivity * time.delta_seconds(),
                        delta.y * camera.pan_sensitivity * time.delta_seconds(),
                        0.0,
                    );

                    transform.translation += camera_translation;
                    camera.center += camera_translation;
                }
            }
            else {
                // LMB = Rotate around target
                if mouse_button_input.pressed(MouseButton::Left) {
                    camera.x -= delta.x * camera.rotate_sensitivity * time.delta_seconds();
                    camera.y -= delta.y * camera.rotate_sensitivity * time.delta_seconds();
    
                    camera.y = camera.y.clamp(camera.min_polar_angle, camera.max_polar_angle);
    
                    let rot = Quat::from_axis_angle(Vec3::unit_y(), camera.x)
                        * Quat::from_axis_angle(-Vec3::unit_x(), camera.y);
                    transform.translation =
                        (rot * Vec3::new(0.0, 1.0, 0.0)) * camera.distance + camera.center;
                    transform.look_at(camera.center, Vec3::unit_y());
                }
                // RMB = Drag
                else if mouse_button_input.pressed(MouseButton::Right) {
                    let camera_translation = Vec3::new(
                        delta.x * camera.pan_sensitivity * time.delta_seconds(),
                        delta.y * camera.pan_sensitivity * time.delta_seconds(),
                        0.0,
                    );

                    transform.translation += camera_translation;
                    camera.center += camera_translation;
                }
            }
            
        }
    }

    fn mouse_zoom_system(
        mut state: ResMut<State>,
        mouse_wheel_events: Res<Events<MouseWheel>>,
        query: Query<(&mut OrbitCamera, &mut Transform, &mut Camera)>,
    ) {
        let mut total = 0.0;
        for event in state.scroll.iter(&mouse_wheel_events) {
            total += event.y
                * match event.unit {
                    Line => 1.0,
                    Pixel => LINE_TO_PIXEL_RATIO,
                };
        }
        Self::set_zoom_level(total, query);    
    }

    fn set_zoom_level(
        zoom: f32,
        mut query: Query<(&mut OrbitCamera, &mut Transform, &mut Camera)>,
    ){
        for (mut camera, mut transform, _) in query.iter_mut() {
            camera.distance *= camera.zoom_sensitivity.powf(zoom);
            camera.distance = camera.distance.clamp(
                if camera.min_zoom_distance >= 0.0 { Some(camera.min_zoom_distance) } else { None }, 
                if camera.max_zoom_distance >= 0.0 { Some(camera.max_zoom_distance) } else { None }
            );
            let translation = &mut transform.translation;
            *translation =
                (*translation - camera.center).normalize() * camera.distance + camera.center;
        }
    }

    fn keyboard_controls_system(
        keyboard_input: Res<Input<KeyCode>>,
        query: Query<(&mut OrbitCamera, &mut Transform, &mut Camera)>,
    ){
        let mut total = 0.0;
        if keyboard_input.pressed(KeyCode::Up){
            total += 0.2;
            Self::set_zoom_level(total, query);
        }
        else if keyboard_input.pressed(KeyCode::Down){
            total -= 0.2;
            Self::set_zoom_level(total, query);
        }   
    }
}
impl Plugin for OrbitCameraPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<State>()
            .add_system(Self::mouse_motion_system.system())
            .add_system(Self::mouse_zoom_system.system())
            .add_system(Self::keyboard_controls_system.system());
    }
}