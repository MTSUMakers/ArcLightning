pub extern crate volume;
use volume::Volume;
use std::io;
pub fn set_volume(volume: f32) -> Result<(), io::Error> {
    
	
	let mut x  = Volume::new();
	x.set(volume);
	
	Ok(())
	
		
	
}