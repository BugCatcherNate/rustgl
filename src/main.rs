#[macro_use]
extern crate glium;

#[allow(unused_imports)]
use glium::{glutin, Surface};

mod support;

fn main() {

    // building the display, ie. the main object
    let event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new();
    let cb = glutin::ContextBuilder::new().with_depth_buffer(24);
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();

    // building the vertex and index buffers
    let vertex_buffer = support::load_wavefront(&display, include_bytes!("support/cube.obj"));
    let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);

    // list of cubes with position and direction
    let n = 10;
    let mut c = 0;
    let mut cubes = (0 .. 100)
        .map(|_| {
            let mut k = (c/n*n) % n;
            let mut j = (c/n) % n; 
            let mut i = c % n;
            let pos: (f32, f32, f32) = ((i as f32), (j as f32), (k as f32));
            let dir: (f32, f32, f32) = (1.0, 1.0, 1.0);
            c += 1; 
            (pos, dir)
        })
        .collect::<Vec<_>>();

    // building the vertex buffer with the attributes per instance
    let mut per_instance = {
        #[derive(Copy, Clone)]
        struct Attr {
            world_position: (f32, f32, f32),
        }

        implement_vertex!(Attr, world_position);

        let data = cubes.iter().map(|_| {
            Attr {
                world_position: (0.0, 0.0, 0.0),
            }
        }).collect::<Vec<_>>();

        glium::vertex::VertexBuffer::dynamic(&display, &data).unwrap()
    };

    let program = glium::Program::from_source(&display,
        "
            #version 140

                uniform mat4 persp_matrix;
                uniform mat4 view_matrix;
                in vec3 position;
                in vec3 normal;
                out vec3 v_position;
                out vec3 v_normal;
                void main() {
                    v_position = position;
                    v_normal = normal;
                    gl_Position = persp_matrix * view_matrix * vec4(v_position * 0.005, 1.0);
                } 
        ",
        "
            #version 140

            in vec3 v_normal;
            in vec3 v_color;
            out vec4 f_color;

            const vec3 LIGHT = vec3(-0.2, 0.8, 0.1);

            void main() {
                float lum = max(dot(normalize(v_normal), normalize(LIGHT)), 0.0);
                vec3 color = (0.3 + 0.7 * lum) * v_color;
                f_color = vec4(color, 1.0);
            }
        ",
        None)
        .unwrap();

    let mut camera = support::camera::CameraState::new();

    // the main loop
    support::start_loop(event_loop, move |events| {
        // updating the cubes
        {
            let mut mapping = per_instance.map();
            for (src, dest) in cubes.iter_mut().zip(mapping.iter_mut()) {

                dest.world_position = src.0;
            }
        }

        // drawing a frame
        let params = glium::DrawParameters {
            depth: glium::Depth {
                test: glium::DepthTest::IfLess,
                write: true,
                .. Default::default()
            },
            .. Default::default()
        };

        let mut target = display.draw();
        target.clear_color_and_depth((0.0, 0.0, 0.0, 0.0), 1.0);
        target.draw((&vertex_buffer, per_instance.per_instance().unwrap()),
                    &indices, &program, &uniform! { matrix: camera.get_perspective() },
                    &params).unwrap();
        target.finish().unwrap();

        let mut action = support::Action::Continue;

        // polling and handling the events received by the window
        for event in events {
            match event {
                glutin::event::Event::WindowEvent { event, .. } => match event {
                    glutin::event::WindowEvent::CloseRequested => action = support::Action::Stop,
                    ev => camera.process_input(&ev),
                },
                _ => (),
            }
        };

        action
    });
}
