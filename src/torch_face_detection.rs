use tch::{self, Tensor};
use tch::nn::ModuleT;
use tch::vision::{resnet, imagenet };
use std::path::Path;


pub fn load_load(filepath: &str) -> Tensor {
    // let image = imagenet::load_image_and_resize(filepath, 224, 224).unwrap();
    let image = imagenet::load_image(filepath).unwrap();

    dbg!(&image);
    image
}

pub fn torch_load_image(filepath: &str) -> Tensor {
    let path = Path::new(filepath);
    let rgb_image = image::open(Path::new(filepath)).unwrap().to_rgb();
    let (dim0, dim1)= &rgb_image.dimensions();

    let mut flattened: Vec<f32> = Vec::new();

    for rgb in rgb_image.pixels() {
        flattened.push(rgb[2] as f32);
        flattened.push(rgb[1] as f32);
        flattened.push(rgb[0] as f32);
    }

    let tensor_image = Tensor::of_slice(&flattened[..]);
    let tensor_image = tensor_image.reshape(&[3, *dim0 as i64, *dim1 as i64]);

    dbg!(&tensor_image);
    rgb_image.save(Path::new("output.png"));

    tensor_image
}

pub fn torch_load_model(filepath: &str) -> Box<dyn ModuleT> {
    let mut vs = tch::nn::VarStore::new(tch::Device::Cpu);

    // TODO match 18, 34

    let net: Box<dyn ModuleT> = Box::new(resnet::resnet18(&vs.root(), imagenet::CLASS_COUNT));

    let model_weights = Path::new(filepath);
    vs.load(model_weights).unwrap();
    net
}
