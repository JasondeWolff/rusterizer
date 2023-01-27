pub extern crate minifb;
pub use minifb::Key;
use std::str::FromStr;

pub struct Window {
    window: minifb::Window,
    frame_buffer: FrameBuffer
}

impl Window {
    pub fn new(name: String, width: usize, height: usize) -> Self {
        let mut options = minifb::WindowOptions::default();
        options.resize = true;

        let window = minifb::Window::new(name.as_str(), width, height, options).expect("Failed to create new window.");
        let mut window = Window {
            window: window,
            frame_buffer: FrameBuffer::new(width, height)
        };

        match minifb::Icon::from_str("assets/rust.ico") {
            Ok(icon) => {
                window.window.set_icon(icon);
            },
            Err(_) => {}
        }

        window
    }

    pub fn limit_fps(&mut self, fps_limit: Option<usize>) {
        let fps_limit = match fps_limit {
            Some(fps_limit) => {
                let fps_limit = 1.0 / (fps_limit as f32) * 1000000.0;
                Some(std::time::Duration::from_micros(fps_limit as u64))
            },
            None => None
        };

        self.window.limit_update_rate(fps_limit);
    }

    pub fn should_close(&mut self) -> bool {
        !self.window.is_open()
    }

    pub fn frame_buffer(&mut self) -> &mut FrameBuffer {
        &mut self.frame_buffer
    }
    
    pub fn display(&mut self) {
        self.window.update_with_buffer(&self.frame_buffer.data, self.frame_buffer.width(), self.frame_buffer.height()).expect("Failed to display frame buffer.");

        let (width, height) = self.window.get_size();
        if width != self.frame_buffer.width || height != self.frame_buffer.height {
            self.on_resize(width, height);
        }
    }

    fn on_resize(&mut self, width: usize, height: usize) {
        self.frame_buffer = FrameBuffer::new(width, height);
    }

    pub fn get_key(&self, key: minifb::Key) -> bool {
        self.window.is_key_pressed(key, minifb::KeyRepeat::Yes)
    }

    pub fn get_key_down(&self, key: minifb::Key) -> bool {
        self.window.is_key_down(key)
    }
}

pub struct FrameBuffer {
    data: Vec<u32>,
    width: usize,
    height: usize
}

impl FrameBuffer {
    pub fn new(width: usize, height: usize) -> Self {
        FrameBuffer {
            data: vec![0; width * height],
            width: width,
            height: height
        }
    }

    pub fn clear(&mut self, value: u32) {
        for i in &mut self.data { *i = value; }
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, value: u32) {
        self.data[y * self.width + x] = value;
    }

    pub fn get_pixel(&mut self, x: usize, y: usize) -> u32 {
        self.data[y * self.width + x]
    }

    pub fn set_pixel_f32(&mut self, x: usize, y: usize, value: f32) {
        self.data[y * self.width + x] = (value * u32::MAX as f32) as u32;
    }

    pub fn get_pixel_f32(&mut self, x: usize, y: usize) -> f32 {
        self.data[y * self.width + x] as f32 / u32::MAX as f32
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn aspect_ratio(&self) -> f32 {
        self.width as f32 / self.height as f32
    }

    pub fn iter(&self) -> core::slice::Iter<u32> {
        self.data.iter()
    }

    pub fn iter_mut(&mut self) -> core::slice::IterMut<'_, u32> {
        self.data.iter_mut()
    }
}