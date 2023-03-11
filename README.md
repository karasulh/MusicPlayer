# MP3 Player With GTK4 Library
It is a Rust project which implements GTK4 (UI) library with the ability to play MP3 songs. It is not last version and it is open for improvement because it has missing parts which I will try to complete in my free time.

---

## The Files in Project
- `Cargo.toml` => it is the fundamental file of Rust which shows the dependencies of this project. It could be written by user.
- `Cargo.lock` => it is the fundamental file of Rust which shows the detailed dependencies and its versions of this project. It can be only read and it is auto-generated. It is generated according to last succesfull build. 
- `main.rs` => it is the main file of project as you can understand from its name. In this file, the parts of the projects like UI objects and playlist as functional part of mp3 player are initialized. Also some UI events are connected which means UI and its behaviours are defined. The contents of main file can be split to another struct object like 'App' struct in next versions. 
-`toolbox.rs` => it defines the toolbox buttons at the top of the UI. 
- `playlist.rs` => it has the functional parts of the player, it handles the UI related functions of music player like add music, play music, add list of music properties. We can say that it organizes the UI and satisfies playing ability to this project.
-`mp3decoder.rs` => for mp3 decoder, I used "symphonia" crate. It decodes the mp3, it means it takes the mp3 file and make it to ready to play by converting into understandable format for player. It converts the file into decoded buffer and divides it into packets to play.
-`player.rs` => for playback, I used "rodio" crate. It uses the above decoder and plays the streaming sound. Streaming sound, must be Source which is trait which we implements it for decoder to play by playback. We can control the songs via playback. 

---

## Notes

-Don't Forget to Install GTK4 to computer like:
`pacman -S mingw-w64-x86_64-gtk4`
Check the gtk4 official website.

1. Add a new song to list with the 'open folder' button by selecting.
2. Play the songs, pause the songs and stop the songs. To play selected song, you must stop the current one.
3. Press the next and previous buttons to skip the song and play the new song in the list.
4. Delete the songs from the list with "--" button.
5. Close the player with "x" button.

-If the song has a picture, then it will show the current song's picture on cover.
-The adjustment bar is not functional now.

-I use threads somewhere and condition variables for multithread to prevent the much usage of CPU when not playing.

-I benefited the logic when starting this mp3 player from the book of Antoni Boucher by Packt Publishing.

---

![player](https://github.com/karasulh/Music_Player_Pc_Application/blob/main/pictures/player.png)
![player2](https://github.com/karasulh/Music_Player_Pc_Application/blob/main/pictures/player2.png)
![player3](https://github.com/karasulh/Music_Player_Pc_Application/blob/main/pictures/player3.png)



