use tch;
use tch::nn::ModuleT;
use tch::vision::{resnet, imagenet};
use std::path::Path;


pub fn torch_load_model(filepath: &str) -> Box<dyn ModuleT> {
    let mut vs = tch::nn::VarStore::new(tch::Device::Cpu);
    let net: Box<dyn ModuleT> = Box::new(resnet::resnet34(&vs.root(), imagenet::CLASS_COUNT));

    let model_weights = Path::new(filepath);
    vs.load(model_weights).unwrap();
    net
}
