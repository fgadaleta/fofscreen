extern crate clap;
extern crate nokhwa;
extern crate tch;
extern crate anyhow;
// use anyhow::Result;

use clap::{App, Arg};
use fofscreen::capture::utils::{capture_loop, display_frames};
use fofscreen::face_detection::*;
use fofscreen::face_encoding::*;
use fofscreen::image_matrix::*;
use fofscreen::landmark_prediction::*;
use fofscreen::alert::Alert;

use nokhwa::{query_devices, CaptureAPIBackend, FrameFormat};
use image::RgbImage;
use std::path::*;
use std::str::FromStr;
use std::time::{Duration, SystemTime};
use std::{env, fs};
use std::collections::HashMap;


// #[macro_use]
// extern crate lazy_static;

fn load_image(filename: &str, path: &str) -> RgbImage {
    let filepath = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(path)
        .join(filename);
    // dbg!("Loading file ", &filepath);
    image::open(&filepath).unwrap().to_rgb()
}

fn main() {
    let matches = App::new("fofscreen")
        .version("0.1.0")
        .author("frag <francesco@amethix.com>")
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
        .arg(Arg::with_name("reference")
            // .short("f")
            .long("reference")
            .help("Pass a directory of reference face images")
            .takes_value(true))
        .arg(Arg::with_name("display")
            .short("d")
            .long("display")
            .help("Pass to open a window and display.")
            .takes_value(false)).get_matches();

    println!("Initializing recognition engine...");
    let DETECTOR: FaceDetector = FaceDetector::default();
    let DETECTOR_CNN: FaceDetectorCnn = FaceDetectorCnn::default();
    let PREDICTOR: LandmarkPredictor = LandmarkPredictor::default();
    let MODEL: FaceEncodingNetwork = FaceEncodingNetwork::default();
    println!("done.");

    let mut reference_matrix: Vec<ImageMatrix> = vec![];
    let mut reference_encodings: HashMap<String, FaceEncoding> = HashMap::new();
    let mut frame_no = 0;
    let print_every = 10;

    // Query example
    if matches.is_present("query") {
        let backend_value = matches.value_of("query").unwrap();
        let mut use_backend = CaptureAPIBackend::Auto;
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

        let reference = matches.value_of("reference").unwrap();
        let reference_path = Path::new(reference);

        println!("Loading reference images from {}", &reference_path.to_str().unwrap());
        for entry in fs::read_dir(reference_path).unwrap() {
            let path = entry.unwrap().path();
            if let Some(imagename) = path.file_name() {
                let reference_rgb_image: RgbImage = load_image(
                    &imagename.to_str().unwrap(),
                    reference_path.to_str().unwrap(),
                );
                let ref_image_matrix = ImageMatrix::from_image(&reference_rgb_image);
                let ref_locations = DETECTOR.face_locations(&ref_image_matrix);
                let ref_rect = ref_locations[0];
                let ref_landmarks = PREDICTOR.face_landmarks(&ref_image_matrix, &ref_rect);
                let ref_encoding =
                    &MODEL.get_face_encodings(&ref_image_matrix, &[ref_landmarks], 0)[0];

                reference_matrix.push(ref_image_matrix);
                let name = String::from_str(imagename.to_str().unwrap()).unwrap();
                reference_encodings.insert(name, ref_encoding.clone());
            }
        }
        println!("Found {} reference images", reference_encodings.len());

        // Start capturing frames
        let recv = capture_loop(0, width, height, fps, format, backend_value, true);
        // run glium
        if matches.is_present("display") {
            let _ = display_frames(recv);
        }
        // dont
        else {
            loop {
                if let Ok(frame) = recv.recv() {
                    if frame_no % print_every == 0 {
                        println!(
                            "Frame width {} height {} size {}",
                            frame.width(),
                            frame.height(),
                            frame.len()
                        );
                    }
                    frame_no += 1;

                    let frame_matrix: ImageMatrix = ImageMatrix::from_image(&frame);
                    let face_locations = DETECTOR.face_locations(&frame_matrix);

                    if face_locations.len() > 0 {
                        let now = SystemTime::now();
                        println!(
                            "{:?} Frame number {} uh oh found a face...",
                            &now, &frame_no
                        );
                        let rect = face_locations[0];
                        let frame_landmarks = PREDICTOR.face_landmarks(&frame_matrix, &rect);
                        let a_encoding =
                            &MODEL.get_face_encodings(&frame_matrix, &[frame_landmarks], 0)[0];

                        // Calculate distance of precomputed encodings of reference image
                        println!("Calculating similarities with references...");
                        let distances = reference_encodings
                            .iter()
                            .map(|(name, re)| {
                                let distance = a_encoding.distance(re);
                                (name.to_owned(), distance)
                            })
                            .collect::<HashMap<String, f64>>();

                        println!("Distances from reference images {:?}", &distances);

                        for (name, dist) in distances.iter() {
                            if dist > &0.6 {
                                let alert = Alert::new( frame_no, name.to_owned());
                                println!("{:?}", &alert);
                            }
                        }

                    }
                } else {
                    println!("Thread terminated, closing!");
                    break;
                }
            }
        }
    }
}
