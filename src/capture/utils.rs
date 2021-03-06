use flume::Receiver;
use glium::{
    implement_vertex,
    index::{self, PrimitiveType},
    program,
    texture::RawImage2d,
    uniform, Display, IndexBuffer, Surface, Texture2d, VertexBuffer,
};
use glutin::{event_loop::EventLoop, window::WindowBuilder, ContextBuilder};
use image::{ImageBuffer, Rgb};
use nokhwa::{Camera, CaptureAPIBackend, FrameFormat};
use std::time::Instant;

#[derive(Copy, Clone)]
pub struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}

/// Loop on device and capture frames
///
pub fn capture_loop(
    index: usize,
    width: u32,
    height: u32,
    fps: u32,
    format: FrameFormat,
    backend_value: CaptureAPIBackend,
    query_device: bool,
) -> Receiver<ImageBuffer<Rgb<u8>, Vec<u8>>> {
    let mut frame_no: usize = 0;
    let print_every: usize = 100;

    // let (send, recv) = flume::unbounded();
    let (send, recv) = flume::bounded(1);

    // spawn a thread for capture
    std::thread::spawn(move || {
        {
            let mut camera =
                Camera::new_with(index, width, height, fps, format, backend_value).unwrap();

            if query_device {
                match camera.compatible_fourcc() {
                    Ok(fcc) => {
                        for ff in fcc {
                            match camera.compatible_list_by_resolution(ff) {
                                Ok(compat) => {
                                    println!("For FourCC {}", ff);
                                    for (res, fps) in compat {
                                        println!("{}x{}: {:?}", res.width(), res.height(), fps);
                                    }
                                }
                                Err(why) => {
                                    println!("Failed to get compatible resolution/FPS list for FrameFormat {}: {}", ff, why.to_string())
                                }
                            }
                        }
                    }
                    Err(why) => {
                        println!("Failed to get compatible FourCC: {}", why.to_string())
                    }
                }
            }

            // open stream
            camera.open_stream().unwrap();
            loop {
                let frame = camera.frame().unwrap();

                if frame_no % print_every == 0 {
                    println!(
                        "Captured frame {}x{} @ {}FPS size {}",
                        frame.width(),
                        frame.height(),
                        fps,
                        frame.len()
                    );
                }
                frame_no += 1;

                send.send(frame).unwrap()
            }
        }
        // IP Camera
        // else {
        // dbg!("ip camera not supported");
        // let ip_camera =
        //     NetworkCamera::new(matches_clone.value_of("capture").unwrap().to_string())
        //         .expect("Invalid IP!");
        // ip_camera.open_stream().unwrap();
        // loop {
        //     let frame = ip_camera.frame().unwrap();
        //     println!(
        //         "Captured frame {}x{} @ {}FPS size {}",
        //         frame.width(),
        //         frame.height(),
        //         fps,
        //         frame.len()
        //     );
        //     send.send(frame).unwrap();
        // }
        // }
    });
    recv
}

/// Display frame to openGL window
///
pub fn display_frames(recv: Receiver<ImageBuffer<Rgb<u8>, Vec<u8>>>) {
    let gl_event_loop = EventLoop::new();
    let window_builder = WindowBuilder::new();
    let context_builder = ContextBuilder::new().with_vsync(true);
    let gl_display = Display::new(window_builder, context_builder, &gl_event_loop).unwrap();

    implement_vertex!(Vertex, position, tex_coords);

    let vert_buffer = VertexBuffer::new(
        &gl_display,
        &[
            Vertex {
                position: [-1.0, -1.0],
                tex_coords: [0.0, 0.0],
            },
            Vertex {
                position: [-1.0, 1.0],
                tex_coords: [0.0, 1.0],
            },
            Vertex {
                position: [1.0, 1.0],
                tex_coords: [1.0, 1.0],
            },
            Vertex {
                position: [1.0, -1.0],
                tex_coords: [1.0, 0.0],
            },
        ],
    )
    .unwrap();

    let idx_buf = IndexBuffer::new(
        &gl_display,
        PrimitiveType::TriangleStrip,
        &[1 as u16, 2, 0, 3],
    )
    .unwrap();

    let program = program!(&gl_display,
        140 => {
            vertex: "
        #version 140
        uniform mat4 matrix;
        in vec2 position;
        in vec2 tex_coords;
        out vec2 v_tex_coords;
        void main() {
            gl_Position = matrix * vec4(position, 0.0, 1.0);
            v_tex_coords = tex_coords;
        }
    ",

            fragment: "
        #version 140
        uniform sampler2D tex;
        in vec2 v_tex_coords;
        out vec4 f_color;
        void main() {
            f_color = texture(tex, v_tex_coords);
        }
    "
        },
    )
    .unwrap();

    // run the event loop
    gl_event_loop.run(move |event, _window, ctrl| {
        let before_capture = Instant::now();
        let frame = recv.recv().unwrap();
        let after_capture = Instant::now();

        let width = &frame.width();
        let height = &frame.height();

        let raw_data = RawImage2d::from_raw_rgb(frame.into_raw(), (*width, *height));
        let gl_texture = Texture2d::new(&gl_display, raw_data).unwrap();

        let uniforms = uniform! {
            matrix: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0f32]
            ],
            tex: &gl_texture
        };

        let mut target = gl_display.draw();
        target.clear_color(0.0, 0.0, 0.0, 0.0);
        target
            .draw(
                &vert_buffer,
                &idx_buf,
                &program,
                &uniforms,
                &Default::default(),
            )
            .unwrap();
        target.finish().unwrap();

        match event {
            glutin::event::Event::WindowEvent { event, .. } => match event {
                glutin::event::WindowEvent::CloseRequested => {
                    *ctrl = glutin::event_loop::ControlFlow::Exit;
                }
                _ => {}
            },
            _ => {}
        }
        println!(
            "Took {}ms to capture",
            after_capture.duration_since(before_capture).as_millis()
        )
    })
}
