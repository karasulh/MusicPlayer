use std::fs::File;
use std::io::BufReader;
use std::path::{Path,PathBuf};
use std::sync::{Arc,Condvar,Mutex};
use std::thread;
use std::rc::Rc;
use std::cell::RefCell;

use crossbeam::queue::SegQueue;

use rodio::{OutputStream,Sink};
use crate::mp3decoder::mp3Decoder;


enum Action{
    Load(String),
    Pause,
    Stop,
}

#[derive(Clone)]
struct EventLoop {
    condition_variable: Arc<(Mutex<bool>,Condvar)>,//wake the thread which is sleeping
    queue: Arc<SegQueue<Action>>,
    playing: Arc<Mutex<bool>>,
}
//Before starting, some nodes for concurrency in Rust:
//Send and Sync traits prevent us from doing data races. A type implements the Send trait is safe to be sent to another thread.
//For example, Rc doesnot implement Send trait, so it couldnot be used with threads.
//Instead, Arc could be used with threads because it is for atomic operations.
//A  type implements the Sync trait is safe to be shared with multiple threads.
//For example, Mutex implements Sync trait, so it satisfies mutually exclusive.

//For SegQueue, we can say that it uses atomic operations and it is lock-free data structures, so we dont need to use Mutex 
//to mutate this value in mutable threads at the same time 
//but we still need to wrap this queue in an Arc to be able to share it with multiple threads. //by Rust Programming By Example(Packt Publishing,2018)
impl EventLoop{
    fn new()->Self{
        EventLoop{
            condition_variable: Arc::new((Mutex::new(false),Condvar::new())),
            queue: Arc::new(SegQueue::new()),
            playing: Arc::new(Mutex::new(false)),
        }
    }
}

pub struct Player{
    app_state: Arc<Mutex<super::State>>,
    event_loop: EventLoop,
}

impl Player{
    pub(crate) fn new(app_state: Arc<Mutex<super::State>>) -> Self{
        let event_loop = EventLoop::new();
        { //Use new scope to avoid having to rename the variables which will be sent to the thread
            //because these variables are used in the initialization of the structure at the end of constructor
            let app_state = app_state.clone();
            let event_loop = event_loop.clone();
            let condition_variable = event_loop.condition_variable.clone();

            thread::spawn(move ||{

                //block the thread when it has nothing to do
                let block = || {
                    let (ref lock,ref condition_variable) = *condition_variable;
                    let mut started = lock.lock().unwrap();
                    *started = false;
                    while !*started {//wait until started = true
                        started = condition_variable.wait(started).unwrap();
                    }
                };

                let mut current_song_path: String = String::new();
                let (_stream,stream_handle) = OutputStream::try_default().unwrap();
                let sink = Sink::try_new(&stream_handle).unwrap(); //sink represents an audio track
                let mut source;

                loop{
                    if let Some(action) = event_loop.queue.pop(){

                        match action {
                            Action::Load(path) => {
                                if sink.is_paused() && path == current_song_path {
                                    sink.play(); //resume to play the song which is paused 
                                }
                                else{
                                    current_song_path = path.clone();
                                    let file = File::open(path).expect("Failed to open path");   
                                    
                                    source = mp3Decoder::new(file).unwrap();
                                    sink.append(source);

                                    //let song_path = current_song_path.clone();
                                    //let song_duration = compute_duration(song_path).unwrap();
                                
                                    sink.play();
                                    println!("{}",sink.len());
                                }

                                //We can access the value inside mutex directly below because we access a field here in struct, Rust automatically dereference fields.
                                app_state.lock().unwrap().stopped = false; //the same with below 2 lines
                                //let mut mutex_guard = app_state.lock().unwrap(); 
                                //mutex_guard.stopped = false; //mutex guard = scoped lock => automatically unlocked when going out of scope.
                                
                                *event_loop.playing.lock().unwrap() = true;
                            }
                            Action::Pause => {
                                println!("Should be paused");
                                sink.pause();
                                app_state.lock().unwrap().stopped = true;
                                *event_loop.playing.lock().unwrap() = false;
                                
                            }
                            Action::Stop => {
                                println!("Should be stopped");
                                sink.stop();
                                sink.clear();
                                app_state.lock().unwrap().stopped = true;
                                *event_loop.playing.lock().unwrap() = false;
                                current_song_path = String::from("zzz");
                            }
                        }
                    }
                    //MutexGuard implements Deref, so we access the value by '*'.
                    else if *event_loop.playing.lock().unwrap(){ //if the song continues

                        if sink.empty(){//if the song is finished
                            println!("Song is finished");
                            app_state.lock().unwrap().stopped = true;
                            *event_loop.playing.lock().unwrap() = false;
                            block();//blocks this thread to save cpu which is worked a lot due to loop
                        }
                    }
                    else{
                        block();
                    }
                }
            });
        }
        Player { app_state, event_loop}
    }

    pub fn load(&self,path: String){
        self.event_loop.queue.push(Action::Load(path));
    }
    pub fn pause(&self){
        self.event_loop.queue.push(Action::Pause);
    }
    pub fn stop(&self){
        self.event_loop.queue.push(Action::Stop);
    }

    pub fn get_condition_var(&self) -> Arc<(Mutex<bool>, Condvar)>{
        let cond_var = self.event_loop.condition_variable.clone();
        cond_var
    }

}

pub fn compute_duration(song_path:String) -> Option<u64>{
    let mut duration_result = 0;
    let duration = mp3_duration::from_path(song_path);
    duration_result = match duration{
        Ok(duration_res) => duration_res.as_secs(),
        Err(duration_err)=> duration_err.at_duration.as_secs(),//in Err case, actually there is result.
    };
    //println!("Song duration: {}",duration_result);
    Some(duration_result)
}

