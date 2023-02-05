mod toolbox;
use toolbox::MusicToolBox;

mod playlist;
use playlist::Playlist;

use gtk4::{prelude::*, Image, Orientation,Box, Adjustment, Scale, Application, ApplicationWindow};
use gtk4::glib::GString;


const PLAY_MUSIC: &str = "media-playback-start";
const PAUSE_MUSIC: &str = "media-playback-pause";

fn main(){
    //Application id is used to be sure the application is only run once.
    //create a new application
    let application = Application::builder().application_id("com.github.rust-musicplayer").build();
    
    //application.connect_activate(|application|{...}); //the same with below with the content of build_ui
    application.connect_activate(build_ui);
    application.run();

}

fn build_ui(app: &Application){

    let window = ApplicationWindow::builder().application(app).title("My Music Player").build();
    let musictoolbox = MusicToolBox::new();

    connect_toolbox_events(&window,&musictoolbox);

    let vert_box= Box::new(gtk4::Orientation::Vertical,5);

    vert_box.append(musictoolbox.get_tool_box());

    let music_image = Image::new();
    music_image.set_from_file(Some("image.jpg"));
    music_image.set_pixel_size(250);
    vert_box.append(&music_image);

    let adjustment = Adjustment::new(0.0,0.0,10.0,0.0,0.0,0.0);
    let scale = Scale::new(Orientation::Horizontal,Some(&adjustment));
    scale.set_draw_value(true);
    vert_box.append(&scale);

    let playlist = Playlist::new();
    vert_box.append(playlist.view());

    window.set_child(Some(&vert_box));
    window.present(); //window.show(); 
}


    
fn connect_toolbox_events<'a>(window:&ApplicationWindow,musictoolbox:&'a MusicToolBox){
    //connect_clicked wants a function with static lifetime so by using 'move', we satifsy this, that is why we clone the variable.
    let window = window.clone();
    musictoolbox.exit_button.connect_clicked(move|_|{window.destroy();}); 

    let play_button = musictoolbox.play_button.clone(); //copy of the pointer of button
    musictoolbox.play_button.connect_clicked( move|_|{
        if play_button.icon_name().unwrap() == GString::from(PLAY_MUSIC.to_string()){
            play_button.set_icon_name(PAUSE_MUSIC);  
        }    
        else{
            play_button.set_icon_name(PLAY_MUSIC);
        }    
    });
    

    
}
    
    
