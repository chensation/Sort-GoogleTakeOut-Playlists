# Sort-GoogleTakeOut-Playlists
Learning Rust. Use this to sort the google music takeout into separate playlist directories.
  
Google Music terminated at the end of 2020 with the option to download users' library using Google's Takeout function. I downloaded my library hoping to continue using the playlists I've created. However, Takeout's strange directory structure made it near impossible to work with, and I'm not going to manually sort my ~1000 songs back into playlists.

## How It Works

The basic directory layout for Takeout looks something like this: 
```
.
├── Playlists
|   ├── Playlist1
|   |   ├── Tracks
|   |   |   ├── Track1Title.csv
|   |   |   ├── Track2Title.csv
|   |   |   └── ...
|   |   | 
|   |   └── Metadata.csv
|   | 
│   ├── Playlist2
|   └── ...
|
├── Tracks
|   ├── Track1.mp3
|   ├── Track2.mp3
|   └── ...
|
└── ...
```
**Note that some of Google Music's auto generated playlists such as *Thumbs Up* may not share the same structure. Since I don't use those, they are not accounted for.**
  
We read through each track inside the **Tracks** directory, store its title from the **mp3**'s metadata, as well as its path. We then navigate into each playlist's **Tracks** directory and read the **csv** file. 

 These **csv** files are structured like so: 
|Title | Album | Artist | Duration (ms) | Rating | Play Count | Removed | Playlist Index|
|--- | --- | --- | --- | --- |---| ---| ---|
Title | Album | Artist | Duration (ms) | Rating | Play Count | Removed | Playlist Index|  

**Note that sometimes the content of the csv is encoded using html ASCCI encoding, we must unencrypt it in order to use it.**
> Example
>   
> mp3 metadata title: **Don't say "lazy"**
>  
> csv stored title: **Don\&#39;t say \&quot;lazy\&quot;**
  
We correlate each track with its corresponding playlist by comparing their title, then save the Playlist Index for each track. **(We assume that no playlists share the same track, and no tracks share the same title.)**
  
Then, a new directory is created for each of our playlists under  the output directory we pointed to. For each playlist directory, we copy and paste its tracks using the path info we've stored. The new track files are named with the format *playIndex_title.mp3*, which ensures that the tracks are ordered by their playlist order. We also replaced all **/** occurrence in the title with **_** so the program does not read it as a new directory. 
  
The remaining tracks that are not a part of any playlists are then pasted into a new directory called **Misc**.

## How To Run It
This program takes in two command line arguments. The first is the path towards the Google Takeout directory, the second is the output directory where the playlists will be saved into. The program checks if the input directory contains sub-directories **Playlists** and **Tracks**.
  
Install Rust, then 
```
$ cargo run input_dir output_dir