use std::{thread, time};

use glium::*;//{
//     DisplayBuild,
//     Surface,
//     VertexBuffer,
//     Program,
//     DrawParameters,
//     Blend,
// };
// use glium::index::{
    // NoIndices,
    // PrimitiveType
// };
use glium::index::*;
use glium::glutin::*;
// use glium::glutin::{
    // WindowBuilder,
    // Event,
// };

// use crate::file_loader;
// use config::Config;
use crate::audio::{
    MultiBuffer,
};

#[derive(Copy, Clone)]
pub struct Scalar {
    pub v: f32
}
implement_vertex!(Scalar, v);


#[derive(Debug)]
pub struct Uniforms {
    pub decay: f32,
    pub thickness: f32,
    pub min_thickness: f32,
    pub thinning: f32,
    pub base_hue: f32,
    pub desaturation : f32,
    pub colorize: bool,
}
/*
[default uniforms]
*/
impl Default for Uniforms {
    fn default() -> Uniforms {
        Uniforms {
            decay           : 0.1,
            thickness       : 10.0,
            min_thickness   : 1.5,
            thinning        : 0.05,
            base_hue        : 0.0,
            desaturation    : 0.1,
            colorize        : true
        }
    }
}

// #[derive(Copy, Clone)]
// pub struct Vec2 {
    // pub vec: [f32; 2],
// }
// implement_vertex!(Vec2, vec);
// 
// #[derive(Copy, Clone)]
// pub struct Vec4 {
    // pub vec: [f32; 4],
// }
// implement_vertex!(Vec4, vec);
use crate::audio::{Vec2, Vec4};
use std::io::prelude::*;
use std::fs::File;

pub fn load_from_file(filename: &str) -> String {
    let mut file = File::open(filename).unwrap_or_else(|e| panic!("couldn't open file {}: {}", filename, e));
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap_or_else(|e| panic!("couldn't read file {}: {}", filename, e));
    contents
}


// pub fn display(config: &Config, buffers: MultiBuffer) {
pub fn display(buffers: MultiBuffer) {
    let mut events_loop = glium::glutin::EventsLoop::new();
    let wb = glutin::WindowBuilder::new();
    let cb = glutin::ContextBuilder::new().with_vsync(true);
    let display = glium::Display::new(wb, cb, &events_loop).unwrap();
        // .with_multisampling(4) // THIS IS LAGGY!
        // .with_vsync(true)
        // .build_glium().unwrap();

    let n = 259;//config.audio.buffer_size + 3;
    let mut ys_data: Vec<_> = (0..n).map(|_| Vec4 { vec: [0.0, 0.0, 0.0, 0.0] }).collect();
    let ys = VertexBuffer::dynamic(&display, &ys_data).unwrap();
    let indices = NoIndices(PrimitiveType::LineStripAdjacency);
    let wave_program = Program::from_source(
        &display,
        &load_from_file("src/glsl/line.vert"),
        &load_from_file("src/glsl/line.frag"),
        Some(&load_from_file("src/glsl/line.geom"))
    ).unwrap();

    let clear_rect = [[-1.0, -1.0], [-1.0, 1.0], [1.0, -1.0], [1.0, 1.0]].into_iter()
        .map(|&v| Vec2 { vec: v })
        .collect::<Vec<_>>();
    let clear_rect_verts = VertexBuffer::new(&display, &clear_rect).unwrap();
    let clear_rect_indices = NoIndices(PrimitiveType::TriangleStrip);
    let clear_program = Program::from_source(
        &display,
        &load_from_file("src/glsl/clear.vert"),
        &load_from_file("src/glsl/clear.frag"),
        None
    ).unwrap();

    let params = DrawParameters {
        blend: Blend::alpha_blending(),
        .. Default::default()
    };

    let Uniforms {
        decay,
        thickness,
        min_thickness,
        thinning,
        base_hue,
        colorize,
        desaturation,
    } = Uniforms::default();

    let mut index = 0;
    let mut render_loop = || {
        // for ev in display.poll_events() {
            events_loop.poll_events(|ev| {
            match ev {
                // glutin::Event::Closed => return Action::Stop,
                _ => {}
            }
            });

        let mut target = display.draw();
        while { !buffers[index].lock().unwrap().rendered } {
            {
                let mut buffer = buffers[index].lock().unwrap();
                let analytic_slice = buffer.analytic.as_slice();
                ys_data.copy_from_slice(analytic_slice);
                buffer.rendered = true;
            };
            ys.write(&ys_data);
            index = (index + 1) % buffers.len();

            let window = display.gl_window();
            // let window = display.get_window().unwrap();
            let (width, height) = window.get_inner_size_points().unwrap();

            let uniforms = uniform! {
                // n: n,
                decay: decay,
                window: [width as f32, height as f32],
                thickness: thickness,
                min_thickness: min_thickness,
                thinning: thinning,
                base_hue: base_hue,
                colorize: colorize,
                desaturation: desaturation,
            };
            target.draw(&clear_rect_verts, &clear_rect_indices, &clear_program, &uniforms, &params).unwrap();
            target.draw(&ys, &indices, &wave_program, &uniforms, &params).unwrap();
        }

        target.finish().unwrap();

        Action::Continue
    };
    // match config.max_fps {
    let max_fps = 30;
    limit_fps(max_fps, render_loop);

    // match max_fps {
        // Some(fps) => limit_fps(fps, render_loop),
        // None => loop {
            // match render_loop() {
                // Action::Stop => return,
                // _ => {}
            // }
        // },
    // }
}

enum Action {
    Continue,
    Stop
}

fn limit_fps<F>(fps: u32, mut render_loop: F) where F: FnMut() -> Action {
    let duration = time::Duration::new(0, 1_000_000_000 / fps);
    loop {
        let now = time::Instant::now();

        match render_loop() {
            Action::Stop => return,
            _ => {}
        }

        let dt = now.elapsed();
        if dt < duration {
            thread::sleep(duration - dt);
        }
    }
}
