extern crate glam;
pub use glam::*;

#[path = "resources/resources.rs"] pub mod resources;
use minifb::Key;
pub use resources::*;

#[path = "graphics/graphics.rs"] pub mod graphics;
use graphics::*;

pub mod shared;
pub use shared::Shared;

mod timer;
use timer::Timer;

fn main() {
    std::env::set_var("RUST_BACKTRACE", "1");

    let mut resources = Resources::init();
    //let monkey_model = resources.get_model(String::from("assets/monkey.gltf"));
    let monkey_model = resources.get_model(String::from("assets/test_models/DamagedHelmet/glTF/DamagedHelmet.gltf"));

    let mut pipeline = Pipeline::new();
    let mut cam_position = Vec3::new(0.0, -0.03, 2.8);

    let mut delta_time;
    let mut delta_timer = Timer::new();

    let mut r = 0.0;

    let mut window = Window::new(String::from("Rusterizer"), 512, 512);
    while !window.should_close() {
        delta_time = delta_timer.elapsed() as f32;
        delta_timer.reset();
        println!("ms: {}", delta_time * 1000.0);

        let speed = 1.0;
        if window.get_key(Key::A) {
            cam_position += Vec3::X * delta_time * speed;
        }
        if window.get_key(Key::D) {
            cam_position -= Vec3::X * delta_time * speed;
        }
        if window.get_key(Key::S) {
            cam_position += Vec3::Z * delta_time * speed;
        }
        if window.get_key(Key::W) {
            cam_position -= Vec3::Z * delta_time * speed;
        }
        if window.get_key(Key::Q) {
            cam_position -= Vec3::Y * delta_time * speed;
        }
        if window.get_key(Key::E) {
            cam_position += Vec3::Y * delta_time * speed;
        }

        let mut frame_buffer = window.frame_buffer();
        frame_buffer.clear(0);
        pipeline.clear_depth();

        r += delta_time;

        pipeline.set_model_matrix(Mat4::from_axis_angle(Vec3::Y, r));
        pipeline.set_view_matrix(Mat4::from_translation(-cam_position));
        pipeline.set_proj_matrix(Mat4::perspective_rh((60.0f32).to_radians(), frame_buffer.aspect_ratio(), 0.01, 100.0));

        pipeline.draw_vertices_indexed(&mut frame_buffer, &monkey_model.as_ref().meshes[0].vertices, &monkey_model.as_ref().meshes[0].indices);

        window.display();
    }
}