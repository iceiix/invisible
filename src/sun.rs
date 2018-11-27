use crate::render;
use crate::render::model;
use cgmath::{Vector3, Matrix4, Decomposed, Rotation3, Rad, Quaternion};

pub struct SunModel {
    moon: model::ModelKey,
}

impl SunModel {

    pub fn new(renderer: &mut render::Renderer) -> SunModel {
        SunModel {
            moon: SunModel::generate_moon(renderer),
        }
    }

    pub fn tick(&mut self, renderer: &mut render::Renderer) {
        let moon = renderer.model.get_model(self.moon).unwrap();
        moon.matrix[0] = Matrix4::from(Decomposed {
            scale: 1.0,
            rot: Quaternion::from_angle_z(Rad((0) as f32)),
            disp: Vector3::new(-300.5, -13.2, 0.5),
        });
    }

    pub fn generate_moon(renderer: &mut render::Renderer) -> model::ModelKey {
        renderer.model.create_model(
            model::SUN,
            vec![vec![
                model::Vertex{x: 0.0, y: -50.0, z: -50.0},
                model::Vertex{x: 0.0, y: 50.0, z: -50.0},
                model::Vertex{x: 0.0, y: -50.0, z: 50.0},
                model::Vertex{x: 0.0, y: 50.0, z: 50.0},
            ]]
        )
    }
}
