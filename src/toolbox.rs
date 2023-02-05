use gtk4::{prelude::*};
use gtk4::{Button,Box};

const PLAY_MUSIC: &str = "media-playback-start";
const STOP_MUSIC: &str = "media-playback-stop";
const DOCUMENT_OPEN: &str = "document-open";
const GO_PREVIOUS: &str = "go-previous";
const GO_NEXT: &str = "go-next";
const WINDOW_CLOSE: &str = "window-close";


pub struct MusicToolBox{
    pub open_button: Button,
    pub prev_button: Button,
    pub play_button: Button,
    pub stop_button: Button,
    pub next_button: Button,
    pub exit_button: Button,
    pub toolbox: Box,
}

impl MusicToolBox{
    pub fn new() -> Self{
        
        let toolbox= Box::new(gtk4::Orientation::Horizontal,30);
    
        let open_button = Button::from_icon_name(DOCUMENT_OPEN);
        open_button.connect_clicked(|button|{ button.set_label("Hello World");});
        toolbox.append(&open_button);

        //toolbox.append(&Separator::new(gtk4::Orientation::Vertical));
        let prev_button = Button::from_icon_name(GO_PREVIOUS);
        prev_button.connect_clicked(|button|{ button.set_label("Hello World2");});
        toolbox.append(&prev_button); 

        let play_button = Button::from_icon_name(PLAY_MUSIC);
        toolbox.append(&play_button);

        let stop_button = Button::from_icon_name(STOP_MUSIC);
        stop_button.connect_clicked(|button|{ button.set_label("Hello World4");});
        toolbox.append(&stop_button);

        let next_button = Button::from_icon_name(GO_NEXT);
        next_button.connect_clicked(|button|{ button.set_label("Hello World5");});
        toolbox.append(&next_button); 

        let exit_button = Button::from_icon_name(WINDOW_CLOSE);
        toolbox.append(&exit_button); 

        MusicToolBox { open_button, prev_button, play_button, stop_button, next_button, exit_button, toolbox}
    }

    pub fn get_tool_box(&self) -> &Box{
        &self.toolbox
    }

}