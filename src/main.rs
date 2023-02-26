mod toolbox;
use toolbox::MusicToolBox;

mod playlist;
use playlist::Playlist;

mod mp3;

mod player;

use gtk4::{prelude::*, Image, Orientation,Box, Adjustment, Scale, Application, ApplicationWindow};
use gtk4::glib::GString;

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

    
    //let playlist2 = Rc::clone(&playlist);
    connect_toolbox_events(&window,&musictoolbox,&playlist,&cover);

    let vert_box= Box::new(gtk4::Orientation::Vertical,5);

    vert_box.append(musictoolbox.get_tool_box());

    cover.set_from_file(Some("image.jpg"));
    cover.set_pixel_size(250);
    vert_box.append(&cover);

    let adjustment = Adjustment::new(0.0,0.0,10.0,0.0,0.0,0.0);
    let scale = Scale::new(Orientation::Horizontal,Some(&adjustment));
    scale.set_draw_value(true);
    vert_box.append(&scale);

    vert_box.append(playlist.view());

    window.set_child(Some(&vert_box));
    window.present(); //window.show(); 
}


    
fn connect_toolbox_events(window: & ApplicationWindow,musictoolbox: & MusicToolBox,playlist:&Rc<Playlist>,cover:&Image){
    //connect_clicked wants a function with static lifetime so by using 'move', we satifsy this, that is why we clone the variable.
    let window_copy = window.clone();
    musictoolbox.exit_button.connect_clicked(move|_|{window_copy.destroy();}); 

    let playlist_copy = Rc::clone(&playlist);
    let cover_copy = cover.clone();
    let play_button = musictoolbox.play_button.clone(); //copy of the pointer of button
    musictoolbox.play_button.connect_clicked( move|_|{
        if play_button.icon_name().unwrap() == GString::from(PLAY_MUSIC.to_string()){
            play_button.set_icon_name(PAUSE_MUSIC);  
            set_cover(&cover_copy, &playlist_copy);
        }    
        else{
            play_button.set_icon_name(PLAY_MUSIC);
        }    
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
        cover.set_from_file(Some("image.jpg"));
        cover.set_pixel_size(250);
    }
}
    
    
