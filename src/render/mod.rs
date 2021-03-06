// Copyright 2016 Matthew Collins
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

pub mod glsl;
#[macro_use]
pub mod shaders;
pub mod model;

use crate::gl;
use byteorder::{WriteBytesExt, NativeEndian};
use cgmath::prelude::*;

// TEMP
const NUM_SAMPLES: i32 = 2;

pub struct Renderer {
    pub model: model::Manager,

    trans_shader: TransShader,


    perspective_matrix: cgmath::Matrix4<f32>,
    camera_matrix: cgmath::Matrix4<f32>,
    pub view_vector: cgmath::Vector3<f32>,

    trans: Option<TransInfo>,
}

impl Renderer {
    pub fn new() -> Renderer {

        let mut greg = glsl::Registry::new();
        shaders::add_shaders(&mut greg);

        // Shaders
        let trans_shader = TransShader::new(&greg);

        Renderer {
            model: model::Manager::new(&greg),
            trans_shader,
            perspective_matrix: cgmath::Matrix4::identity(),
            camera_matrix: cgmath::Matrix4::identity(),
            view_vector: cgmath::Vector3::zero(),

            trans: None,
        }
    }

    pub fn update_camera(&mut self) {
        use std::f64::consts::PI as PI64;

        let width = 854;
        let height = 480;
        gl::viewport(0, 0, width as i32, height as i32);

        self.perspective_matrix = cgmath::Matrix4::from(
            cgmath::PerspectiveFov {
                fovy: cgmath::Rad::from(cgmath::Deg(90f32)),
                aspect: (width as f32 / height as f32),
                near: 0.1f32,
                far: 500.0f32,
            }
        );

        self.init_trans(width, height);

        let yaw: f64 = -8.0;
        let pitch: f64 = 3.0;
        let x: f64 = -200.0;
        let y: f64 = 65.0;
        let z: f64 = 90.0;

        self.view_vector = cgmath::Vector3::new(
            ((yaw - PI64/2.0).cos() * -pitch.cos()) as f32,
            (-pitch.sin()) as f32,
            (-(yaw - PI64/2.0).sin() * -pitch.cos()) as f32
        );
        let camera = cgmath::Point3::new(-x as f32, -y as f32, z as f32);
        let camera_matrix = cgmath::Matrix4::look_at(
            camera,
            camera + cgmath::Point3::new(-self.view_vector.x, -self.view_vector.y, self.view_vector.z).to_vec(),
            cgmath::Vector3::new(0.0, -1.0, 0.0)
        );
        self.camera_matrix = camera_matrix * cgmath::Matrix4::from_nonuniform_scale(-1.0, 1.0, 1.0);
    }

    pub fn tick(&mut self) {
        let trans = self.trans.as_mut().unwrap();
        trans.main.bind();

        gl::clear_color(
             122.0 / 255.0,
             165.0 / 255.0,
             247.0 / 255.0,
             1.0
        );
        gl::clear(gl::ClearFlags::Color | gl::ClearFlags::Depth);

        // Model rendering
        self.model.draw(&self.perspective_matrix, &self.camera_matrix);

        trans.trans.bind();
        gl::clear_buffer(gl::COLOR, 0, &[0.0, 0.0, 0.0, 1.0]);

        gl::check_framebuffer_status();
        gl::unbind_framebuffer();
        trans.draw(&self.trans_shader);

        gl::check_gl_error();
    }

    fn init_trans(&mut self, width: u32, height: u32) {
        self.trans = Some(TransInfo::new(width, height, &self.trans_shader));
    }
}

struct TransInfo {
    main: gl::Framebuffer,
    fb_color: gl::Texture,
    _fb_depth: gl::Texture,
    trans: gl::Framebuffer,
    _depth: gl::Texture,

    array: gl::VertexArray,
    _buffer: gl::Buffer,
}

init_shader! {
    Program TransShader {
        vert = "trans_vertex",
        frag = "trans_frag",
        attribute = {
            required position => "aPosition",
        },
        uniform = {
            required color => "tcolor",
        },
    }
}

impl TransInfo {
    pub fn new(width: u32, height: u32, shader: &TransShader) -> TransInfo {
        let trans = gl::Framebuffer::new();
        trans.bind();

        let trans_depth = gl::Texture::new();
        trans_depth.bind(gl::TEXTURE_2D);
        trans_depth.image_2d_ex(gl::TEXTURE_2D, 0, width, height, gl::DEPTH_COMPONENT24, gl::DEPTH_COMPONENT, gl::UNSIGNED_BYTE, None);
        trans_depth.set_parameter(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR);
        trans_depth.set_parameter(gl::TEXTURE_2D, gl::TEXTURE_MAX_LEVEL, gl::LINEAR);
        trans.texture_2d(gl::DEPTH_ATTACHMENT, gl::TEXTURE_2D, &trans_depth, 0);

        gl::check_framebuffer_status();

        let main = gl::Framebuffer::new();
        main.bind();

        let fb_color = gl::Texture::new();
        fb_color.bind(gl::TEXTURE_2D_MULTISAMPLE);
        fb_color.image_2d_sample(gl::TEXTURE_2D_MULTISAMPLE, NUM_SAMPLES, width, height, gl::RGBA8, false);
        main.texture_2d(gl::COLOR_ATTACHMENT_0, gl::TEXTURE_2D_MULTISAMPLE, &fb_color, 0);

        let fb_depth = gl::Texture::new();
        fb_depth.bind(gl::TEXTURE_2D_MULTISAMPLE);
        fb_depth.image_2d_sample(gl::TEXTURE_2D_MULTISAMPLE, NUM_SAMPLES, width, height, gl::DEPTH_COMPONENT24, false);
        main.texture_2d(gl::DEPTH_ATTACHMENT, gl::TEXTURE_2D_MULTISAMPLE, &fb_depth, 0);
        gl::check_framebuffer_status();

        gl::unbind_framebuffer();

        shader.program.use_program();
        let array = gl::VertexArray::new();
        array.bind();
        let buffer = gl::Buffer::new();
        buffer.bind(gl::ARRAY_BUFFER);

        let mut data = vec![];
        for f in [-1.0, 1.0, 1.0, -1.0, -1.0, -1.0, 1.0, 1.0, 1.0, -1.0, -1.0, 1.0].into_iter() {
            data.write_f32::<NativeEndian>(*f).unwrap();
        }
        buffer.set_data(gl::ARRAY_BUFFER, &data, gl::STATIC_DRAW);

        shader.position.enable();
        shader.position.vertex_pointer(2, gl::FLOAT, false, 8, 0);

        TransInfo {
            main,
            fb_color,
            _fb_depth: fb_depth,
            trans,
            _depth: trans_depth,

            array,
            _buffer: buffer,
        }
    }

    fn draw(&mut self, shader: &TransShader) {
        gl::active_texture(0);
        self.fb_color.bind(gl::TEXTURE_2D_MULTISAMPLE);

        shader.program.use_program();
        shader.color.set_int(0);
        self.array.bind();
        gl::draw_arrays(gl::TRIANGLES, 0, 6);
    }
}

#[allow(unused_must_use)]
pub fn generate_element_buffer(size: usize) -> (Vec<u8>, gl::Type) {
    let mut ty = gl::UNSIGNED_SHORT;
    let mut data = if (size / 6) * 4 * 3 >= u16::max_value() as usize {
        ty = gl::UNSIGNED_INT;
        Vec::with_capacity(size * 4)
    } else {
        Vec::with_capacity(size * 2)
    };
    for i in 0..size / 6 {
        for val in &[0, 1, 2, 2, 1, 3] {
            if ty == gl::UNSIGNED_INT {
                data.write_u32::<NativeEndian>((i as u32) * 4 + val);
            } else {
                data.write_u16::<NativeEndian>((i as u16) * 4 + (*val as u16));
            }
        }
    }

    (data, ty)
}
