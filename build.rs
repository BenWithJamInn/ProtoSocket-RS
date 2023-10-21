extern crate prost_build;

use std::path::{Path, PathBuf};

fn main() ->std::io::Result<()> {
    let paths = get_all_protos(Path::new("protocol").to_path_buf());
    for x in paths.iter() {
        println!("{:?}", x);
    }
    prost_build::compile_protos(paths.as_ref(),
                                &["protocol"]).unwrap();
    Ok(())
}

fn get_all_protos(path: PathBuf) -> Vec<PathBuf> {
    let paths = std::fs::read_dir(path).unwrap();
    let mut proto_paths: Vec<PathBuf> = Vec::new();
    for x in paths {
        let path = x.unwrap();
        let meta = path.metadata().unwrap();
        if meta.is_dir() {
            let protos_in_dir = get_all_protos(path.path());
            proto_paths.extend(protos_in_dir.into_iter());
        } else if meta.is_file() {
            if path.file_name().to_str().unwrap().ends_with(".proto") {
                proto_paths.push(path.path());
            }
        }
    }
    proto_paths
}
