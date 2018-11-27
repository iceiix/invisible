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

pub mod gl;
pub mod render;
pub mod types;
pub mod sun;

use sdl2::Sdl;

pub struct Game {
    renderer: render::Renderer,
    should_close: bool,
    sdl: Sdl,
}

fn main() {
    println!("Starting steven");

    let sdl = sdl2::init().unwrap();
    let sdl_video = sdl.video().unwrap();
    let window = sdl2::video::WindowBuilder::new(&sdl_video, "Steven", 854, 480)
                            .opengl()
                            .resizable()
                            .build()
                            .expect("Could not create sdl window.");
    let gl_attr = sdl_video.gl_attr();
    gl_attr.set_depth_size(24);
    gl_attr.set_context_major_version(3);
    gl_attr.set_context_minor_version(2);
    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);

    let gl_context = window.gl_create_context().unwrap();
    window.gl_make_current(&gl_context).expect("Could not set current context.");

    gl::init(&sdl_video);

    sdl_video.gl_set_swap_interval(1);

    let renderer = render::Renderer::new();
    let mut game = Game {
        renderer,
        should_close: false,
        sdl,
    };
    game.renderer.camera.pos = cgmath::Point3::new(0.5, 13.2, 0.5);
    let mut events = game.sdl.event_pump().unwrap();
    let mut sun_model = sun::SunModel::new(&mut game.renderer);
    while !game.should_close {
        sun_model.tick(&mut game.renderer, 0.0, 0);

        game.renderer.update_camera();

        game.renderer.camera.yaw = -7.2697720829739465;
        game.renderer.camera.pitch = 2.9733976253414633;
        game.renderer.camera.pos.x = -208.76533603647485;
        game.renderer.camera.pos.y = 65.62010000000001;
        game.renderer.camera.pos.z = 90.9279311085242;
 
        game.renderer.tick();

        window.gl_swap_window();

        for event in events.poll_iter() {
            handle_window_event(&mut game, event);
        }
    }
}

fn handle_window_event( game: &mut Game,
                       event: sdl2::event::Event) {
    use sdl2::event::Event;

    match event {
        Event::Quit{..} => game.should_close = true,
        _ => (),
    }
}
