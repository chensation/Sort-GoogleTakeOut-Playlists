use std::{env, fs, io, process, path, collections::HashMap, error::Error};
use id3::Tag;
use htmlescape;

const TITLE : usize = 0; //song title is the first column in the csv
const PLAYORDER : usize = 7; //playorder is the 8th column in the csv

struct Playlist {
    tracks: HashMap<usize, Track>, //i32 for the pos of track in the playlist 
    name: String
}

#[derive(Debug)]
struct Track {

    name: String,
    path: path::PathBuf
}

//read command line arguments
fn read_config(args: &[String]) -> Result<(&str, &str), &'static str> {

    if args.len() < 3 {
        return Err("not enough arguments");
    }

    let input_dir = &args[1];
    let output_dir = &args[2];

    Ok((input_dir, output_dir))
}

//check if input directory points to google checkout
//return true + new path if yes
fn check_input_correct(entrance : &str) -> Result<(bool, path::PathBuf, path::PathBuf), Box<dyn Error>>{

    let (mut is_playlist,mut is_tracks) = (false,  false);
    let (mut playlist_path, mut tracks_path) = (path::PathBuf::new(), path::PathBuf::new());

     for entry in fs::read_dir(entrance)? { //for dir inside the input dir
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() { //if is a dir
            match entry.file_name().to_str().unwrap(){ //match it to playlists or tracks
                
                "Playlists" => {
                    is_playlist = true;
                    playlist_path = path;
                },
                "Tracks" => {
                    is_tracks = true;
                    tracks_path = path;
                },
                _ => {}

            }
        }
    
    }

    Ok((is_playlist && is_tracks, playlist_path, tracks_path))
   
}

//given the path to the Tracks directory, store and return a vector with the tracks
fn store_tracks(tracks_path: &path::PathBuf) -> Result <Vec<Track>, Box<dyn Error>>{

    let mut tracks : Vec<Track> = Vec::new();

    for track in fs::read_dir(tracks_path)? { //for tracks inside the tracks dir
        let track = track?;

        let tag = Tag::read_from_path(track.path()) //use id3 to read the metadata of mp3
                                                .expect(&format!("Unable to read the tag for {}", track.file_name().to_str()
                                                        .expect("store_track() read an empty file name"))); //???
        
        tracks.push(Track{ //add it to tracks vector
            name: tag.title().expect("store_track: song does not contain title").to_owned(),
            path: track.path(),
        });
    
    }

    Ok(tracks)
}

//for a passed in playlist dir, read the csv files, grab the corresponding stored tracks, paste it into a playlist
fn store_playlist(playlist_path: &mut path::PathBuf, all_tracks: &mut Vec<Track>) -> Result <Playlist, Box<dyn Error>>{
    
    let mut playlist = Playlist{
        tracks: HashMap::new(),
        name: String::new()
    };

    playlist.name = playlist_path.file_name().expect(&format!("store_playlist: path {:?} doesn't exist.", playlist_path)) //don't like it
                                        .to_str().unwrap().to_owned();
    
    playlist_path.push("Tracks"); //change dir to the tracks inside the playlist dir
                                // since takeout include a metadata csv file for each playlist
    for track in fs::read_dir(playlist_path)? { //for each track csv files
        
        let track = track?;
        let mut track_reader = csv::Reader::from_path(track.path())?; //use csv reader 

        let track_info =  track_reader.records().next() //only one record for every file, this is the metadata for the track
                                        .expect(&format!("stored_playlist: found empty csv at: {:?}", track.path())).unwrap(); 
        
        let track_title = htmlescape::decode_html(&track_info[TITLE]).unwrap(); //google_takeout had the genius idea of using html ascii encoding in the csv

        let stored_track_index = all_tracks.iter().position(|x| x.name == track_title); //find the index for this track in our all_tracks vector 
        
        match stored_track_index { //move track to playlist if found, else err and skip
            Some(index) => {
                playlist.tracks.insert(track_info[PLAYORDER].parse().unwrap(), all_tracks.remove(index)); //use remove to move our track from all_tracks into the playlist
             } 
            None => {
                eprintln!("stored_playlist: There are no stored tracks corresponding to :\n {:?}", track_info);
                //process::exit(1);
            }
        }

        

    }


    println!("The playlist {} contains:", playlist.name);
    //println!("{:?}", playlist.tracks);
    println!("Total count: {}", playlist.tracks.len());

    Ok(playlist)
}

fn create_playlist(playlist_path: &path::PathBuf, playlist: &Playlist) -> Result<bool, io::Error>{

    match fs::create_dir(playlist_path) { //attempt to create the playlist directory

        Err(e) if e.kind() == io::ErrorKind::AlreadyExists => { //ignore if directory already exists
            println!("{} already exists, copying into it", playlist_path.display());
        }

        Err(e) => return Err(e),

        _=>()
    }

    for i in 0..playlist.tracks.len() { //loop through tracks by playorder

        let track = &playlist.tracks[&i];
        let track_filename = format!("{}_{}.mp3", i, track.name.as_str().replace("/", "_")); //create track file name as playorder_title.mp3

        let track_path = playlist_path.join(track_filename);

        if !track_path.exists() {
            fs::copy(&track.path, track_path)?; //copy from the source path into the new path
        }
        
    }
    Ok(true)
}
fn main() {
    
    let args : Vec<String> = env::args().collect(); //get command line arguments

    let (input_dir, output_dir) = read_config(&args).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {}", err);
        process::exit(1);
    });

    println!("Checking input directory...");

    let (is_takeout, playlists_path, tracks_path) = check_input_correct(&input_dir).unwrap_or_else(|err| {
        eprintln!("Unable to check input directory: {}", err); //check and return paths
        process::exit(1);
    });


    if is_takeout { //if valid path

        println!("Valid directory! Storing tracks...");

        // store all tracks
        let mut all_tracks = store_tracks(&tracks_path).expect("can't store tracks");

        //println!("Here be tracks:\n {:?}", tracks);

        println!("Track successfully stored, sorting playlists...");
        let mut all_playlists : Vec<Playlist> = Vec::new();
        
        //look through every playlist and store their info
        for playlist in fs::read_dir(playlists_path).unwrap(){
            let playlist = playlist.unwrap();

            all_playlists.push(store_playlist(&mut playlist.path(), &mut all_tracks).unwrap_or_else(|err| {
                eprintln!("Unable to sort playlist {:?}: {}", playlist, err);
                process::exit(1);
            }));
        }

        println!("playlists successfully sorted!");

        //loop through playlists and create them in output dir
        for playlist in all_playlists {

            let playlist_path = path::PathBuf::from(output_dir).join(&playlist.name);

            println!("Copying into {}", playlist_path.display());
            
            match create_playlist(&playlist_path, &playlist) {
                Err(err) => {
                    eprintln!("Unable to copy to playlist {}: {}", playlist_path.display(), err);
                }
                _ => ()
            }
        }

        //create a misc playlist if there are still tracks remaining
        if all_tracks.len() > 0 {
            println!("There are still tracks unassociated with a playlist left, placing them into a Misc playlist.");

            let mut misc_playlist = Playlist{
                tracks: HashMap::new(),
                name: "Misc".to_owned()
            };

            let mut index: usize = 0;
            while all_tracks.len() > 0 {
                misc_playlist.tracks.insert(index, all_tracks.pop().unwrap());
                index += 1;

            }

            let misc_path = path::PathBuf::from(output_dir).join(&misc_playlist.name);

            match create_playlist(&misc_path, &misc_playlist) {
                Err(err) => {
                    eprintln!("Unable to copy to playlist {}: {}", misc_path.display(), err);
                }
                _ => ()
            }
        }

        println!("Playlists created! Go check it out!");

    }

    else{
        eprintln!("{} is not a valid path to Google Takeout!",input_dir);
        process::exit(1);
    }
    
}
