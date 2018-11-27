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

mod atlas;
pub mod glsl;
#[macro_use]
pub mod shaders;
pub mod model;

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use crate::gl;
use image;
use image::{GenericImageView};
use byteorder::{WriteBytesExt, NativeEndian};
use cgmath::prelude::*;
use collision;

const ATLAS_SIZE: usize = 1024;

// TEMP
const NUM_SAMPLES: i32 = 2;

pub struct Renderer {
    textures: Arc<RwLock<TextureManager>>,
    pub model: model::Manager,

    trans_shader: TransShader,


    perspective_matrix: cgmath::Matrix4<f32>,
    camera_matrix: cgmath::Matrix4<f32>,
    pub frustum: collision::Frustum<f32>,
    pub view_vector: cgmath::Vector3<f32>,

    pub frame_id: u32,

    trans: Option<TransInfo>,

    pub width: u32,
    pub height: u32,
}

impl Renderer {
    pub fn new() -> Renderer {

        let textures = TextureManager::new();
        let textures = Arc::new(RwLock::new(textures));

        let mut greg = glsl::Registry::new();
        shaders::add_shaders(&mut greg);

        // Shaders
        let trans_shader = TransShader::new(&greg);

        Renderer {
            model: model::Manager::new(&greg),
            textures,

            trans_shader,

            width: 0,
            height: 0,

            perspective_matrix: cgmath::Matrix4::identity(),
            camera_matrix: cgmath::Matrix4::identity(),
            frustum: collision::Frustum::from_matrix4(cgmath::Matrix4::identity()).unwrap(),
            view_vector: cgmath::Vector3::zero(),

            frame_id: 1,

            trans: None,
        }
    }

    pub fn update_camera(&mut self) {
        use std::f64::consts::PI as PI64;

        let width = 854;
        let height = 480;
        self.width = width;
        self.height = height;
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
        self.frustum = collision::Frustum::from_matrix4(self.perspective_matrix * self.camera_matrix).unwrap();
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
        self.model.draw(&self.frustum, &self.perspective_matrix, &self.camera_matrix);

        trans.trans.bind();
        gl::clear_buffer(gl::COLOR, 0, &[0.0, 0.0, 0.0, 1.0]);

        gl::check_framebuffer_status();
        gl::unbind_framebuffer();
        trans.draw(&self.trans_shader);

        gl::check_gl_error();

        self.frame_id = self.frame_id.wrapping_add(1);
    }

    fn init_trans(&mut self, width: u32, height: u32) {
        self.trans = None;
        self.trans = Some(TransInfo::new(width, height, &self.trans_shader));
    }

    pub fn get_textures(&self) -> Arc<RwLock<TextureManager>> {
        self.textures.clone()
    }

    pub fn get_textures_ref(&self) -> &RwLock<TextureManager> {
        &self.textures
    }

    pub fn get_texture(textures: &RwLock<TextureManager>, name: &str) -> Texture {
        return Texture {
            name: "missing".to_owned(),
            atlas: 0,
            x: 0,
            y: 0,
            width: 0,
            height: 0,
            rel_x: 0.0,
            rel_y: 0.0,
            rel_width: 1.0,
            rel_height: 1.0,
            is_rel: false,
        };

        let tex = {
            textures.read().unwrap().get_texture(name)
        };
        match tex {
            Some(val) => val,
            None => {
                let mut t = textures.write().unwrap();
                // Make sure it hasn't already been loaded since we switched
                // locks.
                if let Some(val) = t.get_texture(name) {
                    val
                } else {
                    t.load_texture(name);
                    t.get_texture(name).unwrap()
                }
            }
        }
    }
}

struct TransInfo {
    main: gl::Framebuffer,
    fb_color: gl::Texture,
    _fb_depth: gl::Texture,
    trans: gl::Framebuffer,
    accum: gl::Texture,
    revealage: gl::Texture,
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
            required accum => "taccum",
            required revealage => "trevealage",
            required color => "tcolor",
        },
    }
}

impl TransInfo {
    pub fn new(width: u32, height: u32, shader: &TransShader) -> TransInfo {
        let trans = gl::Framebuffer::new();
        trans.bind();

        let accum = gl::Texture::new();
        accum.bind(gl::TEXTURE_2D);
        accum.image_2d_ex(gl::TEXTURE_2D, 0, width, height, gl::RGBA16F, gl::RGBA, gl::FLOAT, None);
        accum.set_parameter(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR);
        accum.set_parameter(gl::TEXTURE_2D, gl::TEXTURE_MAX_LEVEL, gl::LINEAR);
        trans.texture_2d(gl::COLOR_ATTACHMENT_0, gl::TEXTURE_2D, &accum, 0);

        let revealage = gl::Texture::new();
        revealage.bind(gl::TEXTURE_2D);
        revealage.image_2d_ex(gl::TEXTURE_2D, 0, width, height, gl::R16F, gl::RED, gl::FLOAT, None);
        revealage.set_parameter(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR);
        revealage.set_parameter(gl::TEXTURE_2D, gl::TEXTURE_MAX_LEVEL, gl::LINEAR);
        trans.texture_2d(gl::COLOR_ATTACHMENT_1, gl::TEXTURE_2D, &revealage, 0);

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
            accum,
            revealage,
            _depth: trans_depth,

            array,
            _buffer: buffer,
        }
    }

    fn draw(&mut self, shader: &TransShader) {
        gl::active_texture(0);
        self.accum.bind(gl::TEXTURE_2D);
        gl::active_texture(1);
        self.revealage.bind(gl::TEXTURE_2D);
        gl::active_texture(2);
        self.fb_color.bind(gl::TEXTURE_2D_MULTISAMPLE);

        shader.program.use_program();
        shader.accum.set_int(0);
        shader.revealage.set_int(1);
        shader.color.set_int(2);
        self.array.bind();
        gl::draw_arrays(gl::TRIANGLES, 0, 6);
    }
}

pub struct TextureManager {
    textures: HashMap<String, Texture>,
    atlases: Vec<atlas::Atlas>,

    pending_uploads: Vec<(i32, atlas::Rect, Vec<u8>)>,

    dynamic_textures: HashMap<String, (Texture, image::DynamicImage)>,
    free_dynamics: Vec<Texture>,
}

impl TextureManager {
    fn new() -> TextureManager {
        let mut tm = TextureManager {
            textures: HashMap::new(),
            atlases: Vec::new(),
            pending_uploads: Vec::new(),

            dynamic_textures: HashMap::new(),
            free_dynamics: Vec::new(),
        };
        tm.add_defaults();
        tm
    }

    fn add_defaults(&mut self) {
        self.put_texture("steven",
                         "missing_texture",
                         2,
                         2,
                         vec![
            0, 0, 0, 255,
            255, 0, 255, 255,
            255, 0, 255, 255,
            0, 0, 0, 255,
        ]);
        self.put_texture("steven",
                         "solid",
                         1,
                         1,
                         vec![
            255, 255, 255, 255,
        ]);
    }

    fn get_texture(&self, name: &str) -> Option<Texture> {
        if let Some(_) = name.find(':') {
            self.textures.get(name).cloned()
        } else {
            self.textures.get(&format!("minecraft:{}", name)).cloned()
        }
    }

    fn load_texture(&mut self, name: &str) {
        let (plugin, name) = if let Some(pos) = name.find(':') {
            (&name[..pos], &name[pos + 1..])
        } else {
            ("minecraft", name)
        };
        self.insert_texture_dummy(plugin, name);
    }

    fn put_texture(&mut self,
                   plugin: &str,
                   name: &str,
                   width: u32,
                   height: u32,
                   data: Vec<u8>)
                   -> Texture {
        let (atlas, rect) = self.find_free(width as usize, height as usize);
        self.pending_uploads.push((atlas, rect, data));

        let mut full_name = String::new();
        full_name.push_str(plugin);
        full_name.push_str(":");
        full_name.push_str(name);

        let tex = Texture {
            name: full_name.clone(),
            atlas,
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: rect.height,
            rel_x: 0.0,
            rel_y: 0.0,
            rel_width: 1.0,
            rel_height: 1.0,
            is_rel: false,
        };
        self.textures.insert(full_name, tex.clone());
        tex
    }

    fn find_free(&mut self, width: usize, height: usize) -> (i32, atlas::Rect) {
        let mut index = 0;
        for atlas in &mut self.atlases {
            if let Some(rect) = atlas.add(width, height) {
                return (index, rect);
            }
            index += 1;
        }
        let mut atlas = atlas::Atlas::new(ATLAS_SIZE, ATLAS_SIZE);
        let rect = atlas.add(width, height);
        self.atlases.push(atlas);
        (index, rect.unwrap())
    }

    fn insert_texture_dummy(&mut self, plugin: &str, name: &str) -> Texture {
        let missing = self.get_texture("steven:missing_texture").unwrap();

        let mut full_name = String::new();
        full_name.push_str(plugin);
        full_name.push_str(":");
        full_name.push_str(name);

        let t = Texture {
            name: full_name.to_owned(),
            atlas: missing.atlas,
            x: missing.x,
            y: missing.y,
            width: missing.width,
            height: missing.height,
            rel_x: 0.0,
            rel_y: 0.0,
            rel_width: 1.0,
            rel_height: 1.0,
            is_rel: false,
        };
        self.textures.insert(full_name.to_owned(), t.clone());
        t
    }

    pub fn put_dynamic(&mut self, name: &str, img: image::DynamicImage) -> Texture {
        use std::mem;
        let (width, height) = img.dimensions();
        let (width, height) = (width as usize, height as usize);
        let mut rect_pos = None;
        for (i, r) in self.free_dynamics.iter().enumerate() {
            if r.width == width && r.height == height {
                rect_pos = Some(i);
                break;
            } else if r.width >= width && r.height >= height {
                rect_pos = Some(i);
            }
        }
        let data = img.to_rgba().into_vec();

        if let Some(rect_pos) = rect_pos {
            let mut tex = self.free_dynamics.remove(rect_pos);
            let rect = atlas::Rect {
                x: tex.x,
                y: tex.y,
                width,
                height,
            };
            self.pending_uploads.push((tex.atlas, rect, data));
            let mut t = tex.relative(0.0, 0.0, (width as f32) / (tex.width as f32), (height as f32) / (tex.height as f32));
            let old_name = mem::replace(&mut tex.name, format!("steven-dynamic:{}", name));
            self.dynamic_textures.insert(name.to_owned(), (tex.clone(), img));
            // We need to rename the texture itself so that get_texture calls
            // work with the new name
            let mut old = self.textures.remove(&old_name).unwrap();
            old.name = format!("steven-dynamic:{}", name);
            t.name = old.name.clone();
            self.textures.insert(format!("steven-dynamic:{}", name), old);
            t
        } else {
            let tex = self.put_texture("steven-dynamic", name, width as u32, height as u32, data);
            self.dynamic_textures.insert(name.to_owned(), (tex.clone(), img));
            tex
        }
    }

    pub fn remove_dynamic(&mut self, name: &str) {
        let desc = self.dynamic_textures.remove(name).unwrap();
        self.free_dynamics.push(desc.0);
    }
}

#[derive(Clone, Debug)]
pub struct Texture {
    pub name: String,
    pub atlas: i32,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    is_rel: bool, // Save some cycles for non-relative textures
    rel_x: f32,
    rel_y: f32,
    rel_width: f32,
    rel_height: f32,
}

impl Texture {
    pub fn get_x(&self) -> usize {
        if self.is_rel {
            self.x + ((self.width as f32) * self.rel_x) as usize
        } else {
            self.x
        }
    }

    pub fn get_y(&self) -> usize {
        if self.is_rel {
            self.y + ((self.height as f32) * self.rel_y) as usize
        } else {
            self.y
        }
    }

    pub fn get_width(&self) -> usize {
        if self.is_rel {
            ((self.width as f32) * self.rel_width) as usize
        } else {
            self.width
        }
    }

    pub fn get_height(&self) -> usize {
        if self.is_rel {
            ((self.height as f32) * self.rel_height) as usize
        } else {
            self.height
        }
    }

    pub fn relative(&self, x: f32, y: f32, width: f32, height: f32) -> Texture {
        Texture {
            name: self.name.clone(),
            x: self.x,
            y: self.y,
            atlas: self.atlas,
            width: self.width,
            height: self.height,
            is_rel: true,
            rel_x: self.rel_x + x * self.rel_width,
            rel_y: self.rel_y + y * self.rel_height,
            rel_width: width * self.rel_width,
            rel_height: height * self.rel_height,
        }
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
