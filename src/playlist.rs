use gio::ListModel;
use gio::ListStore; //new type
//use gtk4::ListStore; //old type
use gtk4::{prelude::*,TreeView, TreeViewColumn,CellRendererText, CellRendererPixbuf,
    ColumnView,ColumnViewColumn, SingleSelection,SignalListItemFactory, ListItem, Image, Label};
use gdk_pixbuf::{Pixbuf};
use gtk4::glib::types::Type;
use gtk4::glib::BoxedAnyObject;

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
}


struct Row{
    col1:Pixbuf,
    col2:String,
    col3:String,
    col4:String,
    col5:String,
    col6:String,
    col7:String,
    col8:Pixbuf, 
}


impl Playlist{
    pub fn new() -> Self{
        /*
        let list = [Pixbuf::static_type(),Type::STRING,Type::STRING,Type::STRING,Type::STRING,
        Type::STRING,Type::STRING,Type::STRING, Pixbuf::static_type()];
        let model = ListStore::new(&list);
        let treeview = TreeView::with_model(&model);
        treeview.set_hexpand(true);
        treeview.set_vexpand(true);
        Self::create_columns(&treeview);
        Playlist{model,treeview}
        */

         
        let store= ListStore::new(BoxedAnyObject::static_type());
        let sel = SingleSelection::new(Some(&store));
        let columnview = ColumnView::new(Some(&sel));
        Self::create_columns(&columnview);

        Playlist { model:store, treeview:columnview }
        
    }

    fn create_columns(columnview:&ColumnView){
        Self::add_pixbuff_column(columnview,THUMBNAIL_COLUMN as i32,Visible);
        Self::add_text_column(columnview,"Title",TITLE_COLUMN as i32);
        Self::add_text_column(columnview,"Artist",ARTIST_COLUMN as i32);
        Self::add_text_column(columnview,"Album",ALBUM_COLUMN as i32);
        Self::add_text_column(columnview,"Genre",GENRE_COLUMN as i32);
        Self::add_text_column(columnview,"Year",YEAR_COLUMN as i32);
        Self::add_text_column(columnview,"Track",TRACK_COLUMN as i32);
        Self::add_pixbuff_column(columnview,PIXBUF_COLUMN as i32,Invisible);
    }

    //CellRenderer use to indicate how the data from the model should be rendered in the view
    fn add_text_column(columnview:&ColumnView,title:&str,columnId:i32){
        let colfactory = SignalListItemFactory::new();
        let col = ColumnViewColumn::new(Some(title),Some(&colfactory));
        col.set_expand(true);

        colfactory.connect_setup(|_factory,item|{
            let item = item.downcast_ref::<ListItem>().unwrap();
            let label = Label::new(Some("Text"));
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
    /*
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