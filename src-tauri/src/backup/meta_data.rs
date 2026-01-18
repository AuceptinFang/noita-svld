use std::path::{Path, PathBuf};

pub struct MeteData{
    location : (i64,i64) ,
    hp : f64,
    orbs: i8,
}

impl MeteData {
    fn get_meta_data(path : &Path){
        let player : PathBuf = path.join("/player.xml");
    }
}

