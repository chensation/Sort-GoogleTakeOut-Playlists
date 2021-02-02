use std::{env, fs, io};

struct Playlist {
    tracks: Vec<String>,
    name: String
}

//read command line arguments
fn read_config(args: &[String]) -> (&str, &str) {


    let input_dir = &args[1];
    let output_dir = &args[2];

    (input_dir, output_dir)
}

//check if input directory points to google checkout
//return true + new path if yes
fn check_input_correct(entrance : &str) -> Result<(bool, String), io::Error>{
     for entry in fs::read_dir(entrance)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() && (entry.file_name() == "Playlists") {
            
            return Ok((true, entrance.to_owned() + "/Playlists"));
        }
    
    }

    Ok((false, entrance.to_owned()))
    
}

fn main() {
    
    let args : Vec<String> = env::args().collect();

    let (input_dir, output_dir) = read_config(&args);

    let (is_playlist, playlists_path) = check_input_correct(&input_dir).expect("Unable to check input directory");

    println!("{} evaluated to {}, new path: {}",input_dir, is_playlist, playlists_path);
}
