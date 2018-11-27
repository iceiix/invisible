
use super::glsl;
use super::shaders;
use crate::gl;
use cgmath::{Matrix4, SquareMatrix};
use collision::{Frustum};
use std::collections::HashMap;
use byteorder::{WriteBytesExt, NativeEndian};

pub struct Manager {
    collections: Vec<Collection>,

    index_buffer: gl::Buffer,
    index_type: gl::Type,
    max_index: usize,
}

pub const DEFAULT: CollectionKey = CollectionKey(0);
pub const SUN: CollectionKey = CollectionKey(1);

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct CollectionKey(usize);

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ModelKey(CollectionKey, usize);

impl Manager {
    pub fn new(greg: &glsl::Registry) -> Manager {
        let mut m = Manager {
            collections: vec![],

            index_buffer: gl::Buffer::new(),
            index_type: gl::UNSIGNED_SHORT,
            max_index: 0,
        };
        m.add_collection(
            &greg.get("model_vertex"),
            &greg.get("model_frag"),
        );
        m.add_collection(
            &greg.get("sun_vertex"),
            &greg.get("sun_frag"),
        );
        m
    }

    pub fn add_collection(&mut self, vert: &str, frag: &str) -> CollectionKey {
        let collection = Collection {
            shader: ModelShader::new_manual(vert, frag),
            models: HashMap::new(),
            next_id: 0,
        };
        self.collections.push(collection);
        CollectionKey(self.collections.len())
    }

    pub fn get_model(&mut self, key: ModelKey) -> Option<&mut Model> {
        let collection = &mut self.collections[(key.0).0];
        collection.models.get_mut(&key)
    }

    pub fn create_model(&mut self, ckey: CollectionKey, parts: Vec<Vec<Vertex>>) -> ModelKey {
        let array = gl::VertexArray::new();
        array.bind();
        self.index_buffer.bind(gl::ELEMENT_ARRAY_BUFFER);
        let buffer = gl::Buffer::new();
        buffer.bind(gl::ARRAY_BUFFER);

        let mut model = {
            let collection = &mut self.collections[ckey.0];
            collection.shader.program.use_program();
            collection.shader.position.map(|v| v.enable());
            collection.shader.texture_info.map(|v| v.enable());
            collection.shader.texture_offset.map(|v| v.enable());
            collection.shader.color.map(|v| v.enable());
            collection.shader.id.map(|v| v.enable());
            collection.shader.position.map(|v| v.vertex_pointer(3, gl::FLOAT, false, 36, 0));
            collection.shader.texture_info.map(|v| v.vertex_pointer(4, gl::UNSIGNED_SHORT, false, 36, 12));
            collection.shader.texture_offset.map(|v| v.vertex_pointer_int(3, gl::SHORT, 36, 20));
            collection.shader.color.map(|v| v.vertex_pointer(4, gl::UNSIGNED_BYTE, true, 36, 28));
            collection.shader.id.map(|v| v.vertex_pointer_int(1, gl::UNSIGNED_BYTE, 36, 32));

            let mut model = Model {
                // For culling only
                x: 0.0,
                y: 0.0,
                z: 0.0,
                radius: 0.0,
                // Per a part
                matrix: Vec::with_capacity(parts.len()),
                colors: Vec::with_capacity(parts.len()),
                block_light: 15.0,
                sky_light: 15.0,

                array,
                buffer,
                buffer_size: 0,
                count: 0,

                verts: vec![],
            };

            for (i, part) in parts.into_iter().enumerate() {
                model.matrix.push(Matrix4::identity());
                model.colors.push([1.0, 1.0, 1.0, 1.0]);
                for mut pp in part {
                    pp.id = i as u8;
                    model.verts.push(pp);
                }
            }
            model
        };

        Self::rebuild_model(&mut model);
        if self.max_index < model.count as usize {
            let (data, ty) = super::generate_element_buffer(model.count as usize);
            self.index_buffer.bind(gl::ELEMENT_ARRAY_BUFFER);
            self.index_buffer.set_data(gl::ELEMENT_ARRAY_BUFFER, &data, gl::DYNAMIC_DRAW);
            self.max_index = model.count as usize;
            self.index_type = ty;
        }

        let collection = &mut self.collections[ckey.0];
        let key = ModelKey(ckey, collection.next_id);
        collection.next_id += 1;
        collection.models.insert(key, model);

        key
    }

    pub fn remove_model(&mut self, key: ModelKey) {
        let collection = &mut self.collections[(key.0).0];
        collection.models.remove(&key);
    }

    fn rebuild_model(model: &mut Model) {
        model.array.bind();
        model.count = ((model.verts.len() / 4) * 6) as i32;

        let mut buffer = Vec::with_capacity(36 * model.verts.len());
        for vert in &model.verts {
            let _ = buffer.write_f32::<NativeEndian>(vert.x);
            let _ = buffer.write_f32::<NativeEndian>(vert.y);
            let _ = buffer.write_f32::<NativeEndian>(vert.z);
            let _ = buffer.write_u16::<NativeEndian>(0);
            let _ = buffer.write_u16::<NativeEndian>(0);
            let _ = buffer.write_u16::<NativeEndian>(0);
            let _ = buffer.write_u16::<NativeEndian>(0);
            let _ = buffer.write_i16::<NativeEndian>(((0 as f64) * 16.0 * vert.texture_x) as i16);
            let _ = buffer.write_i16::<NativeEndian>(((0 as f64) * 16.0 * vert.texture_y) as i16);
            let _ = buffer.write_i16::<NativeEndian>(0);
            let _ = buffer.write_i16::<NativeEndian>(0);
            let _ = buffer.write_u8(vert.r);
            let _ = buffer.write_u8(vert.g);
            let _ = buffer.write_u8(vert.b);
            let _ = buffer.write_u8(vert.a);
            let _ = buffer.write_u8(vert.id);
            let _ = buffer.write_u8(0);
            let _ = buffer.write_u8(0);
            let _ = buffer.write_u8(0);
        }

        model.buffer.bind(gl::ARRAY_BUFFER);
        if buffer.len() < model.buffer_size {
            model.buffer.re_set_data(gl::ARRAY_BUFFER, &buffer);
        } else {
            model.buffer.set_data(gl::ARRAY_BUFFER, &buffer, gl::DYNAMIC_DRAW);
            model.buffer_size = buffer.len();
        }
    }

    pub fn draw(&mut self, _frustum: &Frustum<f32>, perspective_matrix: &Matrix4<f32>, camera_matrix: &Matrix4<f32>) {
        for collection in &self.collections {
            collection.shader.program.use_program();
            collection.shader.perspective_matrix.map(|v| v.set_matrix4(perspective_matrix));
            collection.shader.camera_matrix.map(|v| v.set_matrix4(camera_matrix));
            collection.shader.texture.map(|v| v.set_int(0));

            for model in collection.models.values() {
                model.array.bind();
                collection.shader.lighting.map(|v| v.set_float2(model.block_light, model.sky_light));
                collection.shader.model_matrix.map(|v| v.set_matrix4_multi(&model.matrix));
                collection.shader.color_mul.map(|v| v.set_float_mutli_raw(model.colors.as_ptr() as *const _, model.colors.len()));
//println!("about to draw model {:?} {:?}", model.count, self.index_type);
                gl::draw_elements(gl::TRIANGLES, model.count, self.index_type, 0);
            }
        }
    }
}

struct Collection {
    shader: ModelShader,

    models: HashMap<ModelKey, Model>,

    next_id: usize,
}

pub struct Model {
    // For culling only
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub radius: f32,
    // Per a part
    pub matrix: Vec<Matrix4<f32>>,
    pub colors: Vec<[f32; 4]>,
    pub block_light: f32,
    pub sky_light: f32,

    array: gl::VertexArray,
    buffer: gl::Buffer,
    buffer_size: usize,
    count: i32,

    pub verts: Vec<Vertex>,
}

#[derive(Clone)]
pub struct Vertex {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub texture: super::Texture,
    pub texture_x: f64,
    pub texture_y: f64,
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
    pub id: u8,
}

init_shader! {
    Program ModelShader {
        vert = "model_vertex",
        frag = "model_frag",
        attribute = {
            optional position => "aPosition",
            optional texture_info => "aTextureInfo",
            optional texture_offset => "aTextureOffset",
            optional color => "aColor",
            optional id => "id",
        },
        uniform = {
            optional perspective_matrix => "perspectiveMatrix",
            optional camera_matrix => "cameraMatrix",
            optional model_matrix => "modelMatrix[]",
            optional texture => "textures",
            optional light_level => "lightLevel",
            optional sky_offset => "skyOffset",
            optional lighting => "lighting",
            optional color_mul => "colorMul[]",
        },
    }
}

