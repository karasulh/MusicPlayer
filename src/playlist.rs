use gdk_pixbuf::InterpType;
use gdk_pixbuf::PixbufLoader;

use gio::File;
use gio::ListModel;
use gtk4::ApplicationWindow;
use gtk4::TreeIter;

//use gio::ListStore; //new type
use gtk4::ListStore; //old type

use gtk4::ffi::GTK_RESPONSE_ACCEPT;
use gtk4::ffi::GTK_RESPONSE_CANCEL;
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


pub struct Playlist{
    model: ListStore,
    treeview: TreeView,
}

/*
pub struct Playlist{
    model: ListStore,
    treeview: ColumnView,
}

struct Row{
    // col1:Pixbuf,
    col2:String,
    col3:String,
    col4:String,
    col5:String,
    col6:String,
    col7:String,
    col8:String,
    // col9:Pixbuf, 
}
*/



impl Playlist{
    pub fn new() -> Self{
        
        let list = [Pixbuf::static_type(),Type::STRING,Type::STRING,Type::STRING,Type::STRING,
        Type::STRING,Type::STRING,Type::STRING, Pixbuf::static_type()];
        let model = gtk4::ListStore::new(&list);
        let treeview = TreeView::with_model(&model);
        treeview.set_hexpand(true);
        treeview.set_vexpand(true);
        Self::create_columns(&treeview);
        Playlist{model,treeview}
        

        /*
        let store= gio::ListStore::new(BoxedAnyObject::static_type());
        let sel = SingleSelection::new(Some(&store));
        let columnview = ColumnView::new(Some(&sel));
        Self::create_columns(&columnview);
        Playlist { model:store, treeview:columnview }
        */
    }
    /*
    fn create_columns(columnview:&ColumnView){
        Self::add_pixbuff_column(columnview,THUMBNAIL_COLUMN as i32,Visible);
        Self::add_text_column(columnview,"Title",TITLE_COLUMN as i32);
        Self::add_text_column(columnview,"Artist",ARTIST_COLUMN as i32);
        Self::add_text_column(columnview,"Album",ALBUM_COLUMN as i32);
        Self::add_text_column(columnview,"Genre",GENRE_COLUMN as i32);
        Self::add_text_column(columnview,"Year",YEAR_COLUMN as i32);
        Self::add_text_column(columnview,"Track",TRACK_COLUMN as i32);
        Self::add_text_column(columnview,"Path",PATH_COLUMN as i32);
        Self::add_pixbuff_column(columnview,PIXBUF_COLUMN as i32,Invisible);
    }

    //CellRenderer use to indicate how the data from the model should be rendered in the view
    fn add_text_column(columnview:&ColumnView,title:&str,columnId:i32){
        let colfactory = SignalListItemFactory::new();
        let col = ColumnViewColumn::new(Some(title),Some(&colfactory));
        col.set_expand(true);

        println!("AddTextColumn");

        colfactory.connect_setup(move |_factory,item|{
            let item = item.downcast_ref::<ListItem>().unwrap();
            let boxed = item.item().unwrap().downcast::<BoxedAnyObject>().unwrap();
            let row : Ref<Row> = boxed.borrow();
            //let label = Label::new(Some("Text"));
            let label = Label::new(Some(&row.col3));
            println!("{:?}",row.col3);
            println!("AddTextColumnColFactory");
            item.set_child(Some(&label));
        });

        columnview.append_column(&col);
    }

    fn add_pixbuff_column(columnview:&ColumnView,columnId:i32,visibility:Visibility){
        let colfactory = SignalListItemFactory::new();
        let col = ColumnViewColumn::new(Some("Pixbuff"),Some(&colfactory));
        col.set_expand(true);
        if visibility==Visible{
            col.set_visible(true);
        }
        
        colfactory.connect_setup(|_factory,item|{
            let item = item.downcast_ref::<ListItem>().unwrap();
            let image = Image::new();
            item.set_child(Some(&image));
        });

        columnview.append_column(&col);
    }
    
    
    pub fn view(&self) -> &ColumnView{
        &self.treeview
    }

    fn set_pixbuf(&self, row:&TreeIter, tag:&Tag){
        if let Some(picture) = tag.pictures().next(){
            let pixbuf_loader = PixbufLoader::new();
            pixbuf_loader.set_size(IMAGE_SIZE,IMAGE_SIZE);
            pixbuf_loader.write(&picture.data).unwrap();
            if let Some(pixbuf) = pixbuf_loader.pixbuf(){
                let thumbnail = pixbuf.scale_simple(THUMBNAIL_SIZE, THUMBNAIL_SIZE,InterpType::Nearest).unwrap();
                //self.model.set_value(row, THUMBNAIL_COLUMN, &thumbnail.to_value());
                //self.model.set_value(row, PIXBUF_COLUMN, &pixbuf.to_value());
            }
            pixbuf_loader.close().unwrap();
        }
    }

    pub fn add(&self,path:&Path){
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

            //self.set_pixbuf(&row, &tag);
            //self.model.set_value(&row,TITLE_COLUMN,&title.to_value());
            // self.model.set_value(&row,ARTIST_COLUMN,&artist.to_value());
            // self.model.set_value(&row,ALBUM_COLUMN,&album.to_value());
            // self.model.set_value(&row,GENRE_COLUMN,&genre.to_value());
            // self.model.set_value(&row,YEAR_COLUMN,&year.to_value());
            // self.model.set_value(&row,TRACK_COLUMN,&track_value.to_value());
            let itemx = Row{
                col2:title.to_string(),
                col3:artist.to_string(),
                col4:album.to_string(),
                col5:genre.to_string(),
                col6:year.to_string(),
                col7:track.to_string(),
                col8:total_tracks.to_string()};
            self.model.append(&BoxedAnyObject::new(itemx));
        }
        else{
            // self.model.set_value(&row,TITLE_COLUMN,&filename.to_value());
        }
        let path = path.to_str().unwrap_or_default();
        //self.model.set_value(&row, PATH_COLUMN, &path.to_value());

        
        //self.model.append(&BoxedAnyObject::new(Row{}));

    }
    */
    
    
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
    
   
}


       /*
        let col1factory = SignalListItemFactory::new();
        let col2factory = SignalListItemFactory::new();
        let col3factory = SignalListItemFactory::new();
        let col4factory = SignalListItemFactory::new();
        let col5factory = SignalListItemFactory::new();
        let col6factory = SignalListItemFactory::new();
        let col7factory = SignalListItemFactory::new();
        let col8factory = SignalListItemFactory::new();

        
        let col1 = ColumnViewColumn::new(Some("Thumbnail"),Some(&col1factory));
        let col2 = ColumnViewColumn::new(Some("Title"),Some(&col2factory));
        let col3 = ColumnViewColumn::new(Some("Artist"),Some(&col3factory));
        let col4 = ColumnViewColumn::new(Some("Album"),Some(&col4factory));
        let col5 = ColumnViewColumn::new(Some("Genre"),Some(&col5factory));
        let col6 = ColumnViewColumn::new(Some("Year"),Some(&col6factory));
        let col7 = ColumnViewColumn::new(Some("Track"),Some(&col7factory));
        let col8 = ColumnViewColumn::new(Some("Pixbuf"),Some(&col8factory));
        col2.set_expand(true);
        col3.set_expand(true);
        col4.set_expand(true);
        col5.set_expand(true);
        col6.set_expand(true);
        col7.set_expand(true);


        col1factory.connect_setup(|_factory,item|{
            let item = item.downcast_ref::<ListItem>().unwrap();
            let image = Image::new();
            item.set_child(Some(&image));
        });

        col2factory.connect_setup(|_factory,item|{
            let item = item.downcast_ref::<ListItem>().unwrap();
            let label = Label::new(Some("Title2"));
            item.set_child(Some(&label));
        });

        col3factory.connect_setup(|_factory,item|{
            let item = item.downcast_ref::<ListItem>().unwrap();
            let label = Label::new(Some("Title2"));
            item.set_child(Some(&label));
        });

        col4factory.connect_setup(|_factory,item|{
            let item = item.downcast_ref::<ListItem>().unwrap();
            let label = Label::new(Some("Title2"));
            item.set_child(Some(&label));
        });

        col5factory.connect_setup(|_factory,item|{
            let item = item.downcast_ref::<ListItem>().unwrap();
            let label = Label::new(Some("Title2"));
            item.set_child(Some(&label));
        });

        col6factory.connect_setup(|_factory,item|{
            let item = item.downcast_ref::<ListItem>().unwrap();
            let label = Label::new(Some("Title2"));
            item.set_child(Some(&label));
        });

        col7factory.connect_setup(|_factory,item|{
            let item = item.downcast_ref::<ListItem>().unwrap();
            let label = Label::new(Some("Title2"));
            item.set_child(Some(&label));
        });

        col8factory.connect_setup(|_factory,item|{
            let item = item.downcast_ref::<ListItem>().unwrap();
            let image = Image::new();
            item.set_child(Some(&image));
        });

        col1.set_visible(true);
        col8.set_visible(false);
        columnview.append_column(&col1);
        columnview.append_column(&col2);
        columnview.append_column(&col3);
        columnview.append_column(&col4);
        columnview.append_column(&col5);
        columnview.append_column(&col6);
        columnview.append_column(&col7);
        columnview.append_column(&col8);
        */