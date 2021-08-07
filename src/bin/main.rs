extern crate clap;
extern crate nokhwa;

use clap::{App, Arg};
use nokhwa::{query_devices, CaptureAPIBackend, FrameFormat};

// use fofscreen;

// use crate::capture::utils::{capture_loop, display_frames};
// use self::capture_loop;
// fofscreen::capture::utils::{capture_loop, display_frames};

// use capture::utils::{capture_loop, display_frames};
// use crate::capture_loop;
// use fofscreen::capture;
// use crate::capture;

fn main() {
    let matches = App::new("fofscreen")
        .version("0.1.0")
        .author("frag <franccesco@amethix.com>")
        .about("Fuck OfF my SCREEN")
        .arg(Arg::with_name("query")
            .short("q")
            .long("query")
            .value_name("BACKEND")
            // TODO: Update as new backends are added!
            .help("Query the system? Pass AUTO for automatic backend, UVC to query using UVC, V4L to query using Video4Linux, GST to query using Gstreamer.. Will post the list of availible devices.")
            .default_value("AUTO")
            .takes_value(true))
        .arg(Arg::with_name("capture")
            .short("c")
            .long("capture")
            .value_name("LOCATION")
            .help("Capture from device? Pass the device index or string. Defaults to 0. If the input is not a number, it will be assumed an IPCamera")
            .default_value("0")
            .takes_value(true))
        .arg(Arg::with_name("query-device")
            .short("s")
            .long("query-device")
            .help("Show device queries from `compatible_fourcc` and `compatible_list_by_resolution`. Requires -c to be passed to work.")
            .takes_value(false))
        .arg(Arg::with_name("width")
            .short("w")
            .long("width")
            .value_name("WIDTH")
            .help("Set width of capture. Does nothing if -c flag is not set.")
            .default_value("640")
            .takes_value(true))
        .arg(Arg::with_name("height")
            .short("h")
            .long("height")
            .value_name("HEIGHT")
            .help("Set height of capture. Does nothing if -c flag is not set.")
            .default_value("480")
            .takes_value(true))
        .arg(Arg::with_name("framerate")
            .short("rate")
            .long("framerate")
            .value_name("FRAMES_PER_SECOND")
            .help("Set FPS of capture. Does nothing if -c flag is not set.")
            .default_value("15")
            .takes_value(true))
        .arg(Arg::with_name("format")
            .short("4cc")
            .long("format")
            .value_name("FORMAT")
            .help("Set format of capture. Does nothing if -c flag is not set. Possible values are MJPG and YUYV. Will be ignored if not either. Ignored by GStreamer backend.")
            .default_value("MJPG")
            .takes_value(true))
        .arg(Arg::with_name("capture-backend")
            .short("b")
            .long("backend")
            .value_name("BACKEND")
            .help("Set the capture backend. Pass AUTO for automatic backend, UVC to query using UVC, V4L to query using Video4Linux, GST to query using Gstreamer, OPENCV to use OpenCV.")
            .default_value("AUTO")
            .takes_value(true))
        .arg(Arg::with_name("display")
            .short("d")
            .long("display")
            .help("Pass to open a window and display.")
            .takes_value(false)).get_matches();

    // Query example
    if matches.is_present("query") {
        let backend_value = matches.value_of("query").unwrap();
        let mut use_backend = CaptureAPIBackend::Auto;
        // AUTO
        if backend_value == "AUTO" {
            use_backend = CaptureAPIBackend::Auto;
        } else if backend_value == "UVC" {
            use_backend = CaptureAPIBackend::UniversalVideoClass;
        } else if backend_value == "GST" {
            use_backend = CaptureAPIBackend::GStreamer;
        } else if backend_value == "V4L" {
            use_backend = CaptureAPIBackend::Video4Linux;
        }

        match query_devices(use_backend) {
            Ok(devs) => {
                for (idx, camera) in devs.iter().enumerate() {
                    println!("Device at index {}: {}", idx, camera)
                }
            }
            Err(why) => {
                println!("Failed to query: {}", why.to_string())
            }
        }
    }

    if matches.is_present("capture") {
        let backend_value = {
            match matches.value_of("capture-backend").unwrap() {
                "UVC" => CaptureAPIBackend::UniversalVideoClass,
                "GST" => CaptureAPIBackend::GStreamer,
                "V4L" => CaptureAPIBackend::Video4Linux,
                "OPENCV" => CaptureAPIBackend::OpenCv,
                _ => CaptureAPIBackend::Auto,
            }
        };
        let width = matches
            .value_of("width")
            .unwrap()
            .trim()
            .parse::<u32>()
            .expect("Width must be a u32!");
        let height = matches
            .value_of("height")
            .unwrap()
            .trim()
            .parse::<u32>()
            .expect("Height must be a u32!");
        let fps = matches
            .value_of("framerate")
            .unwrap()
            .trim()
            .parse::<u32>()
            .expect("Framerate must be a u32!");
        let format = {
            match matches.value_of("format").unwrap() {
                "YUYV" => FrameFormat::YUYV,
                _ => FrameFormat::MJPEG,
            }
        };

        let recv = capture_loop(0, width, height, fps, format, backend_value, true);

        // run glium
        if matches.is_present("display") {
            let _ = display_frames(recv);
        }

        // dont
        else {
            loop {
                if let Ok(frame) = recv.recv() {
                    println!(
                        "Frame width {} height {} size {}",
                        frame.width(),
                        frame.height(),
                        frame.len()
                    );
                } else {
                    println!("Thread terminated, closing!");
                    break;
                }
            }
        }
    }
}