extern crate rand;
use rand::Rng;
use std::fs;
use std::io;

fn select_random_image(image_dir: PathBuf) -> io::Result<PathBuf>{

    
    let mut paths: Vec<PathBuf> = fs::read_dir("image_dir").?;
    let path = rand::thread_rng().choose(&paths)?;
    path




}