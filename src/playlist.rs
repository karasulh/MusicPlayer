use gdk_pixbuf::InterpType;
use gdk_pixbuf::PixbufLoader;

use gio::File;
use gio::ListModel;
use gtk4::ApplicationWindow;
use gtk4::BitsetIter;
use gtk4::TreeIter;

use gio::ListStore; //new type
//use gtk4::ListStore; //old type

use gtk4::ffi::GTK_RESPONSE_ACCEPT;
use gtk4::ffi::GTK_RESPONSE_CANCEL;
use gtk4::ffi::GtkBitset;
use gtk4::ffi::gtk_bitset_iter_init_at;
use gtk4::ffi::gtk_bitset_iter_init_first;
use gtk4::{prelude::*,TreeView, TreeViewColumn,CellRendererText, CellRendererPixbuf,
    ColumnView,ColumnViewColumn, SingleSelection,SignalListItemFactory, ListItem, Image, Label};
use gtk4::{FileChooserAction,FileChooserDialog,FileFilter,ResponseType};
use gdk_pixbuf::{Pixbuf};
use gtk4::glib::types::Type;
use gtk4::glib::BoxedAnyObject;
use id3::Tag;
use id3::TagLike;

use std::borrow::Borrow;
use std::ops::Deref;
use std::path::Path;
use std::path::PathBuf;

use std::cell::Ref;
use std::cell::RefCell;
use std::rc::Rc;

use std::sync::{Arc,Mutex};
use crate::player::Player;
use crate::State;

const THUMBNAIL_COLUMN: u32 = 0;
const TITLE_COLUMN: u32 = 1;
const ARTIST_COLUMN: u32 = 2;
const ALBUM_COLUMN: u32 = 3;
const GENRE_COLUMN: u32 = 4;
const YEAR_COLUMN: u32 = 5;
const TRACK_COLUMN: u32 = 6;
const PATH_COLUMN: u32 = 7;
const PIXBUF_COLUMN: u32 = 8;

const IMAGE_SIZE: i32 = 256;
const THUMBNAIL_SIZE: i32 =64;

use self::Visibility::*;
#[derive(PartialEq)]
enum Visibility{
    Invisible,
    Visible,
}

/*
pub struct Playlist{
    model: ListStore,
    treeview: TreeView,
}
*/


pub struct Playlist{
    model: ListStore,
    treeview: ColumnView,
    player:Player,
}

#[derive(Debug)]
struct Row{
    //col_thumbnail:Pixbuf,
    col_thumbnail:Image,
    col_title:String,
    col_artist:String,
    col_album:String,
    col_genre:String,
    col_year:String,
    col_track:String,
    col_path:String,
    col_pixbuf:Image, 
    //col_pixbuf:Pixbuf, 
}

//We use pub(crate) syntax to silent an error. Since State is private type and is used in public method, compiler throws an error.
//This guarantees the function is public to the other modules of the crate, but other crates cannot access it.
impl Playlist{
    pub(crate) fn new(state: Arc<Mutex<State>>) -> Self{
        /*
        let list = [Pixbuf::static_type(),Type::STRING,Type::STRING,Type::STRING,Type::STRING,
        Type::STRING,Type::STRING,Type::STRING, Pixbuf::static_type()];
        let model = gtk4::ListStore::new(&list);
        let treeview = TreeView::with_model(&model);
        treeview.set_hexpand(true);
        treeview.set_vexpand(true);
        Self::create_columns(&treeview);
        Playlist{model,treeview}
        */

        
        let store= gio::ListStore::new(BoxedAnyObject::static_type());
        let sel = SingleSelection::new(Some(&store));
        let columnview = ColumnView::new(Some(&sel));
        Self::create_columns(&columnview);
        Playlist { model:store, treeview:columnview, player:Player::new(state.clone())}
        
    }
    
    fn create_columns(columnview:&ColumnView){
        Self::add_pixbuff_column(columnview,THUMBNAIL_COLUMN as u32,Visible);
        Self::add_text_column(columnview,"Title",TITLE_COLUMN as u32);
        Self::add_text_column(columnview,"Artist",ARTIST_COLUMN as u32);
        Self::add_text_column(columnview,"Album",ALBUM_COLUMN as u32);
        Self::add_text_column(columnview,"Genre",GENRE_COLUMN as u32);
        Self::add_text_column(columnview,"Year",YEAR_COLUMN as u32);
        Self::add_text_column(columnview,"Track",TRACK_COLUMN as u32);
        Self::add_text_column(columnview,"Path",PATH_COLUMN as u32);
        Self::add_pixbuff_column(columnview,PIXBUF_COLUMN as u32,Invisible);
    }

    fn add_text_column(columnview:&ColumnView,title:&str,column_id:u32){
        let colfactory = SignalListItemFactory::new();
        let col = ColumnViewColumn::new(Some(title),Some(&colfactory));
        col.set_expand(true);

        colfactory.connect_bind(move |_factory,item|{
            let item = item.downcast_ref::<ListItem>().unwrap();
            let boxed = item.item().unwrap().downcast::<BoxedAnyObject>().unwrap();
            let row : Ref<Row> = boxed.borrow();
            let column_of_row = match column_id{
                TITLE_COLUMN => row.col_title.clone(),
                ARTIST_COLUMN => row.col_artist.clone(),
                ALBUM_COLUMN => row.col_album.clone(),
                GENRE_COLUMN=> row.col_genre.clone(),
                YEAR_COLUMN => row.col_year.clone(),
                TRACK_COLUMN => row.col_track.clone(),
                PATH_COLUMN => row.col_path.clone(),
                _ => String::from(""),
            };
            let label = Label::new(Some(&column_of_row));
            item.set_child(Some(&label));
        });

        columnview.append_column(&col);
    }

    fn add_pixbuff_column(columnview:&ColumnView,column_id:u32,visibility:Visibility){
        let colfactory = SignalListItemFactory::new();
        let col = ColumnViewColumn::new(Some("Pixbuff"),Some(&colfactory));
        col.set_expand(true);
        if visibility==Invisible {
            col.set_visible(false);
        }
        
        colfactory.connect_bind(move |_factory,item|{
            let item = item.downcast_ref::<ListItem>().unwrap();
            let boxed = item.item().unwrap().downcast::<BoxedAnyObject>().unwrap();
            let row : Ref<Row> = boxed.borrow();

            let image;
            if column_id == THUMBNAIL_COLUMN{
                image = &row.col_thumbnail;
                item.set_child(Some(image));
            }
            else if column_id == PIXBUF_COLUMN{
                image = &row.col_pixbuf;
                item.set_child(Some(image));
            }
        });

        columnview.append_column(&col);
    }
    
    
    pub fn view(&self) -> &ColumnView{
        &self.treeview
    }

    fn set_pixbuf(&self, item_row:&mut Row, tag:&Tag){
        if let Some(picture) = tag.pictures().next(){
            let pixbuf_loader = PixbufLoader::new();
            pixbuf_loader.set_size(IMAGE_SIZE,IMAGE_SIZE);
            pixbuf_loader.write(&picture.data).unwrap();
            if let Some(pixbuf) = pixbuf_loader.pixbuf(){
                let thumbnail = pixbuf.scale_simple(THUMBNAIL_SIZE, THUMBNAIL_SIZE,InterpType::Nearest).unwrap();
                item_row.col_thumbnail.set_from_pixbuf(Some(&thumbnail));
                item_row.col_pixbuf.set_from_pixbuf(Some(&pixbuf));
                //self.model.set_value(row, THUMBNAIL_COLUMN, &thumbnail.to_value());
                //self.model.set_value(row, PIXBUF_COLUMN, &pixbuf.to_value());
            }
            pixbuf_loader.close().unwrap();
        }
    }

    pub fn add(&self,path:&Path){

        let mut music_item_for_one_row = Row{
            col_thumbnail: Image::new(),
            col_title : String::from(""),
            col_artist: String::from(""),
            col_album: String::from(""),
            col_genre: String::from(""),
            col_year: String::from(""),
            col_track: String::from(""),
            col_path: String::from(""),
            col_pixbuf: Image::new(),
        };
        let filename = path.file_stem().unwrap_or_default().to_str().unwrap_or_default();

        if let Ok(tag) = Tag::read_from_path(path){
            let title = tag.title().unwrap_or(filename);
            let artist = tag.artist().unwrap_or("(no artist)");
            let album = tag.album().unwrap_or("(no artist)");
            let genre = tag.genre().unwrap_or("(no genre)");
            let year = tag.year().map(|year| year.to_string()).unwrap_or("(no year)".to_string());
            let track = tag.track().map(|track| track.to_string()).unwrap_or("??".to_string());
            let total_tracks = tag.total_tracks().map(|total_tracks|total_tracks.to_string()).unwrap_or("??".to_string());
            let track_value = format!("{} / {}",track,total_tracks);

            self.set_pixbuf(&mut music_item_for_one_row, &tag);

            music_item_for_one_row.col_title = title.to_string();
            music_item_for_one_row.col_artist = artist.to_string();
            music_item_for_one_row.col_album = album.to_string();
            music_item_for_one_row.col_genre = genre.to_string();
            music_item_for_one_row.col_year = year.to_string();
            music_item_for_one_row.col_track = track.to_string();

        }
        else{
            music_item_for_one_row.col_title = filename.to_string();
        }
        let path = path.to_str().unwrap_or_default();
        music_item_for_one_row.col_path = path.to_string();

        self.model.append(&BoxedAnyObject::new(music_item_for_one_row));
    }
    
    pub fn remove_selection(&self){
        let selection = self.treeview.model().unwrap().downcast::<SingleSelection>().unwrap();
        if let Some(sel_obj) = selection.selected_item(){
            let sel_pos = selection.selected();
            self.model.remove(sel_pos);
        }
    }

    pub fn get_image(&self) -> Option<Image>{
        let selection = self.treeview.model().unwrap().downcast::<SingleSelection>().unwrap();
        if let Some(sel_obj) = selection.selected_item(){
            let sel_pos = selection.selected();
            let boxed = self.model.item(sel_pos).unwrap().downcast::<BoxedAnyObject>().unwrap();
            let row:Ref<Row> = boxed.borrow();
            let image = row.col_pixbuf.clone();
           return Some(image)
        }
        None
    }
    
    fn selected_path(&self) -> Option<String>{
        let selection = self.treeview.model().unwrap().downcast::<SingleSelection>().unwrap();
        if let Some(sel_obj) = selection.selected_item(){
            let sel_pos = selection.selected();
            let boxed = self.model.item(sel_pos).unwrap().downcast::<BoxedAnyObject>().unwrap();
            let row:Ref<Row> = boxed.borrow();
            let path = row.col_path.clone();
           return Some(path);
        }
        None
    }

    pub fn play(&self) -> bool{
        if let Some(path) = self.selected_path(){
            self.player.load(path);
            true
        }
        else{
            false
        }
    }
    /*

    pub fn remove_selection(&self){
        let selection = self.treeview.get_selection();
        if let Some((,iter)) = selection.get_selected(){
            self.model.remove(&iter);
        }
    }

    pub fn add(&self,path:&Path){
        let filename = path.file_stem().unwrap_or_default().to_str().unwrap_or_default();
        let row= self.model.append();

        if let Ok(tag) = Tag::read_from_path(&path){
            let title = tag.title().unwrap_or(filename);
            let artist = tag.artist().unwrap_or("(no artist)");
            let album = tag.album().unwrap_or("(no artist)");
            let genre = tag.genre().unwrap_or("(no genre)");
            let year = tag.year().map(|year| year.to_string()).unwrap_or("(no year)".to_string());
            let track = tag.track().map(|track| track.to_string()).unwrap_or("??".to_string());
            let total_tracks = tag.total_tracks().map(|total_tracks|total_tracks.to_string()).unwrap_or("??".to_string());
            let track_value = format!("{} / {}",track,total_tracks);

            self.set_pixbuf(&row, &tag);
            self.model.set_value(&row,TITLE_COLUMN,&title.to_value());
            self.model.set_value(&row,ARTIST_COLUMN,&artist.to_value());
            self.model.set_value(&row,ALBUM_COLUMN,&album.to_value());
            self.model.set_value(&row,GENRE_COLUMN,&genre.to_value());
            self.model.set_value(&row,YEAR_COLUMN,&year.to_value());
            self.model.set_value(&row,TRACK_COLUMN,&track_value.to_value());
        }
        else{
            self.model.set_value(&row,TITLE_COLUMN,&filename.to_value());
        }
        let path = path.to_str().unwrap_or_default();
        self.model.set_value(&row, PATH_COLUMN, &path.to_value());
    }

    
    fn set_pixbuf(&self, row:&TreeIter, tag:&Tag){
        if let Some(picture) = tag.pictures().next(){
            let pixbuf_loader = PixbufLoader::new();
            pixbuf_loader.set_size(IMAGE_SIZE,IMAGE_SIZE);
            pixbuf_loader.write(&picture.data).unwrap();
            if let Some(pixbuf) = pixbuf_loader.pixbuf(){
                let thumbnail = pixbuf.scale_simple(THUMBNAIL_SIZE, THUMBNAIL_SIZE,InterpType::Nearest).unwrap();
                self.model.set_value(row, THUMBNAIL_COLUMN, &thumbnail.to_value());
                self.model.set_value(row, PIXBUF_COLUMN, &pixbuf.to_value());
            }
            pixbuf_loader.close().unwrap();
        }
    }
    
    fn create_columns(treeview:&TreeView){
        Self::add_pixbuff_column(treeview,THUMBNAIL_COLUMN as i32,Visible);
        Self::add_text_column(treeview,"Title",TITLE_COLUMN as i32);
        Self::add_text_column(treeview,"Artist",ARTIST_COLUMN as i32);
        Self::add_text_column(treeview,"Album",ALBUM_COLUMN as i32);
        Self::add_text_column(treeview,"Genre",GENRE_COLUMN as i32);
        Self::add_text_column(treeview,"Year",YEAR_COLUMN as i32);
        Self::add_text_column(treeview,"Track",TRACK_COLUMN as i32);
        Self::add_pixbuff_column(treeview,PIXBUF_COLUMN as i32,Invisible);
    }

    //CellRenderer use to indicate how the data from the model should be rendered in the view
    fn add_text_column(treeview:&TreeView,title:&str,column:i32){
        let view_column = TreeViewColumn::new();
        view_column.set_title(title);
        let cell = CellRendererText::new();//show text
        view_column.set_expand(true);
        view_column.pack_start(&cell, true);
        view_column.add_attribute(&cell, "text", column);//view set text attribute acc. to data comes from the column of model
        treeview.append_column(&view_column);
    }

    fn add_pixbuff_column(treeview:&TreeView,column:i32,visibility:Visibility){
        let view_column = TreeViewColumn::new();
        if visibility == Visible{
            let cell = CellRendererPixbuf::new();//show image
            view_column.pack_start(&cell,true);
            view_column.add_attribute(&cell,"pixbuf",column);
        }
        treeview.append_column(&view_column);
    }

    pub fn view(&self) -> &TreeView{
        &self.treeview
    }
    */
   
}

