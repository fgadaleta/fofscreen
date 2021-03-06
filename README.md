# F(uck)of(f my )screen

`fofscreen` is a tool that
* captures frames from any video device (compatible with libuvc and v4l),
* recognizes faces and
* sends you alerts when intruders are sitting in front of your computer


Inspired by [a similar python library](https://github.com/ageitgey/face_recognition), face_recognition is a Rust library that binds to certain specific features of the [dlib C++ library](https://github.com/davisking/dlib).

These include:

- An FHOG-based face detector.
- A CNN-based face detector (slower, but more powerful).
- A face landmark predictor for identifying specific landmarks (eyes, nose, etc) from face rectangles.
- A face encoding neural network for generating 128 dimensional face encodings that can be compared via their euclidean distances.

## Building

`fofscreen` requires dlib to be installed.

`fofscreen` includes a `download-models` feature flag that can be used with `cargo build --features download-models`.

This will automatically download the face predictor, cnn face detector and face encoding neural network models (the fhog face detector is included in dlib and does not need to be downloaded). Alternatively, these models can be downloaded manually:

- CNN Face Detector: http://dlib.net/files/shape_predictor_68_face_landmarks.dat.bz2
- Landmark Predictor: http://dlib.net/files/mmod_human_face_detector.dat.bz2
- Face Recognition Net: http://dlib.net/files/dlib_face_recognition_resnet_model_v1.dat.bz2

if this feature flag is enabled, the matching structs will have `Default::default` implementations provided that allows you to load them without having to worry about file locations.


## Getting started

### Install dlib

Ubuntu/Debian Linux OS

`sudo apt install libdlib-dev`
#### Install from Python

`pip install dlib --verbose`


#### Install from sources

```
wget http://dlib.net/files/dlib-19.22.tar.bz2
tar xvf dlib-19.22.tar.bz2
cd dlib*
mkdir build
cd build
cmake ..
cmake --build . --config Release
sudo make install
sudo ldconfig
```

*Optional install*

`sudo apt-get install -y libuvc-dev `


### Build and Run

Build all workspaces. From root

`cargo build --features download-models`


## Where is libtorch?
`sudo apt install libtorch-dev`

`export LD_LIBRARY_PATH=/home/frag/c0ding/fofscreen/target/debug/build/torch-sys-09e5ff1706274e8f/out/libtorch/libtorch/lib`

`export LIBTORCH=/home/frag/c0ding/fofscreen/target/debug/build/torch-sys-09e5ff1706274e8f/out/libtorch/libtorch/`

Run with

`./target/debug/./fofscreen --reference images -r 15`

`./target/debug/./fofscreen -r 5 -w 640 -q V4L --reference assets`


where `images` is a directory with reference images (jpg, png)
