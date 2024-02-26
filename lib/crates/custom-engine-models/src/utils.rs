use std::{fs::read, path::Path};

pub fn get_image_data<P: AsRef<Path>>(file_name: P) -> Option<Vec<u8>> {
    read(file_name).ok()
}
