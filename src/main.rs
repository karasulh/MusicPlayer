#![allow(unused_imports)]
#![allow(non_camel_case_types)]
#![allow(unused_assignments)]
#![allow(unused_variables)]
#![allow(dead_code)]

mod toolbox;
use toolbox::MusicToolBox;

mod playlist;
use playlist::Playlist;

mod mp3decoder;
mod player;

use gtk4::{prelude::*, Image, Orientation,Box, Adjustment, Scale, Application, ApplicationWindow};

use std::path::{Path, PathBuf};

use gio::File;
use gtk4::{FileChooserAction,FileChooserDialog,FileFilter,ResponseType};

use std::cell::Ref;
use std::cell::RefCell;
use std::rc::Rc;
use gtk4::glib::clone;
use gtk4::glib;

use std::sync::{Arc,Mutex};


const PLAY_MUSIC: &str = "media-playback-start";
const PAUSE_MUSIC: &str = "media-playback-pause";

struct State{
    stopped: bool,
}

//RAII: Resource Acquisition Is Initialization: Resource is allocated in cosntructor, released in descructor.

fn main(){
    
    let application = Application::builder().application_id("com.github.rust-musicplayer").build();//Application id is used to be sure the application is only run once.
    
    application.connect_activate(build_ui);//application.connect_activate(|application|{...}); //the same with the content of build_ui
    application.run();

}

fn build_ui(app: &Application){

    let window = ApplicationWindow::builder().application(app).title("My Music Player").build();
    let musictoolbox = MusicToolBox::new();
    let state: Arc<Mutex<State>> = Arc::new(Mutex::new(State{stopped:true}));
    let playlist = Rc::new(Playlist::new(state.clone()));
    let cover = Image::new();
    let adjustment = Adjustment::new(0.0,0.0,10.0,0.0,0.0,0.0);

    connect_toolbox_events(&window,&musictoolbox,&playlist,&cover,&state);

    let vert_box= Box::new(gtk4::Orientation::Vertical,5);
    vert_box.append(musictoolbox.get_tool_box());

    cover.set_from_file(Some("pictures/image.jpg"));
    cover.set_pixel_size(250);
    vert_box.append(&cover);

    let scale = Scale::new(Orientation::Horizontal,Some(&adjustment));
    scale.set_draw_value(true);
    vert_box.append(&scale);

    vert_box.append(playlist.view());

    window.set_child(Some(&vert_box));
    window.present(); //window.show(); 
}


    
fn connect_toolbox_events(window: &ApplicationWindow,musictoolbox: &MusicToolBox,playlist:&Rc<Playlist>,cover:&Image, state:&Arc<Mutex<State>>){
    //connect_clicked wants a function with static lifetime so by using 'move', we satifsy this, that is why we clone the variable.
    let window_copy = window.clone();
    musictoolbox.exit_button.connect_clicked(move|_|{window_copy.destroy();}); 

    let current_song_duration_sec= Arc::new(Mutex::new(0));
    
    let current_song_duration_sec_copy = Arc::clone(&current_song_duration_sec);
    let playlist_copy = Rc::clone(&playlist);
    let cover_copy = cover.clone();
    let state_copy = Arc::clone(&state);
    let play_button = musictoolbox.play_button.clone(); //copy of the pointer of button
    musictoolbox.play_button.connect_clicked( move|_|{
        if state_copy.lock().unwrap().stopped {
            if playlist_copy.play(){
            //if play_button.icon_name().unwrap() == GString::from(PLAY_MUSIC.to_string()){
                play_button.set_icon_name(PAUSE_MUSIC);  
                set_cover(&cover_copy, &playlist_copy);
                cover_copy.show();
                *current_song_duration_sec_copy.lock().unwrap() = playlist_copy.duration_of_song_sec().unwrap();
            } 
            println!("should be changed icon to start");
        }
        else{
            println!("should be changed icon to pause");
            playlist_copy.pause();
            play_button.set_icon_name(PLAY_MUSIC);
        }    
    });

    let cover_copy = cover.clone();
    let playlist_copy = Rc::clone(&playlist);
    let play_button = musictoolbox.play_button.clone();
    musictoolbox.stop_button.connect_clicked(move |_| {
        playlist_copy.stop();
        cover_copy.hide();
        play_button.set_icon_name(PLAY_MUSIC);
    });

    let parent = window.clone();
    let playlist_copy = Rc::clone(&playlist);
    musictoolbox.open_button.connect_clicked(move|_|{
        show_open_dialog(&parent ,&playlist_copy)
    });   

    let playlist_copy = Rc::clone(&playlist);
    musictoolbox.remove_button.connect_clicked(move|_|{
        playlist_copy.remove_selection();
    });

    let play_button_copy = musictoolbox.play_button.clone();
    let cover_copy = cover.clone();
    let playlist_copy = Rc::clone(&playlist);
    musictoolbox.next_button.connect_clicked(move|_|{
        playlist_copy.next();
        play_button_copy.set_icon_name(PAUSE_MUSIC);
        set_cover(&cover_copy, &playlist_copy);
        cover_copy.show();
    });

    let play_button_copy = musictoolbox.play_button.clone();
    let cover_copy = cover.clone();
    let playlist_copy = Rc::clone(&playlist);
    musictoolbox.prev_button.connect_clicked(move|_|{
        playlist_copy.previous();
        play_button_copy.set_icon_name(PAUSE_MUSIC);
        set_cover(&cover_copy, &playlist_copy);
        cover_copy.show();
    });

    /* //Changing Adjustment Bar
    let playlist_copy = Rc::clone(&playlist);     
    let adjustment_copy = adjustment.clone();
    let state_copy = Arc::clone(&state);
    let current_song_duration_sec_copy = Arc::clone(&current_song_duration_sec);
    // glib::source::timeout_add_seconds(3, move ||{
    //     //let sec = *current_song_duration_sec_copy.lock().unwrap() as f64;
    //     adjustment_copy.set_upper(5.0);
    //     adjustment_copy.set_value(10.0);
    //     glib::source::Continue(true)
    // });
    */

}


fn show_open_dialog(parent: &ApplicationWindow ,playlist: &Rc<Playlist>){

    let buttons = [("Cancel",ResponseType::Cancel),("Accept",ResponseType::Accept)];
    let dialog = FileChooserDialog::new(Some("Select an MP3 audio file"),
                                                    Some(parent),FileChooserAction::Open,&buttons);
    let filter = FileFilter::new();
    filter.add_mime_type("audio/mp3");
    filter.set_name(Some("MP3 audio file"));
    dialog.add_filter(&filter);
    dialog.show();
    
    //this closure wants immutable variable for file, so we use it RefCell for interior mutability property. 
    let playlist_copy = Rc::clone(&playlist);
    dialog.connect_response(move |dialog,response|{
        if response == ResponseType::Accept{       
            if let Some(path) = dialog.file().unwrap().path(){
                (*playlist_copy).add(path.as_path());
            }
        }
        dialog.destroy();
    });
}

fn set_cover(cover: &Image, playlist: &Rc<Playlist>){
    if let Some(image) = playlist.get_image(){
        cover.set_from_paintable(image.paintable().as_ref());
    }
    else{
        cover.set_from_file(Some("pictures/image.jpg"));
        cover.set_pixel_size(250);
    }
}
    
    
