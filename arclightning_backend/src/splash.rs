use rand;
use rand::Rng;
use std::fs;
use std::io;

fn select_random_image(image_dir: PathBuf) -> Result<PathBuf, io::Error>{

    
    let  paths: Vec<PathBuf> = fs::read_dir("image_dir")?
    .flatten().map(|entry| entry.path()).collect();
    
    
    rand::thread_rng().choose(&paths).ok_or_else(|| io::Error::new
    (io::ErrorKind::Other, "choosing random file failed"))
    




}