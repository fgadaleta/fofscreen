#[warn(dead_code)]

use std::time::SystemTime;


#[derive(Debug)]
enum AlertType {
    Stdout,
    Email,
    Sound
}


#[derive(Debug)]
pub struct Alert {
    timestamp: SystemTime,
    frameno: usize,
    reference_name: String,
    atype: AlertType
}

impl Alert {
    pub fn new(frameno: usize, reference_name: String) -> Self {
        Alert {
            timestamp: SystemTime::now(),
            frameno,
            reference_name,
            atype: AlertType::Stdout
        }
    }

}


