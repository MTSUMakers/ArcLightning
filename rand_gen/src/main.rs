extern crate rand;
use rand::Rng;
use std::fs;


fn main() {
    
    let mut count=0;
    
    let mut paths: Vec<_> = fs::read_dir("./").unwrap()
    .map(|r| r.unwrap())
    .collect();
    paths.sort_by_key(|dir| dir.path());
    
    
    for _i in paths.iter(){
        count = count+1;   
    }


    let random_number= rand::thread_rng().gen_range(0,count);
    let first = &paths[random_number];

    println!("filename: {}", first.path().display());
   
    
}
   



