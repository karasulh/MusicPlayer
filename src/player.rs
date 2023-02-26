use std::fs::File;
use std::io::BufReader;
use std::path::{Path,PathBuf};
use std::sync::{Arc,Condvar,Mutex};
use std::thread;

use crossbeam::queue::SegQueue;

use crate::mp3::Mp3Decoder;
//use pulse_simple::Playback;

//use mp3::Mp3Decoder;
//use self::Action::*;


const BUFFER_SIZE:usize = 1000; //# of samples we'll decode and play to avoid usage of all CPU
const DEFAULT_RATE:u32 = 44100;

enum Action{
    Load(PathBuf),
    Stop,
}

#[derive(Clone)]
struct EventLoop {
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
            thread::spawn(move ||{
                let mut buffer = [[0;2]; BUFFER_SIZE];//contains samples to be played
                let mut playback = Playback::new("MP3","MP3 Playback",None,DEFAULT_RATE); //object that allow us to play music on hardware
                let mut source = None;
                loop{
                    if let Some(action) = event_loop.queue.pop(){
                        match action {
                            Action::Load(path) => {
                                let file = File::open(path).unwrap();
                                source = Some(Mp3Decoder::new(BufReader::new(file)).unwrap());
                                let rate = source.as_ref().map(|source| 
                                                                    source.samples_rate()).unwrap_or(DEFAULT_RATE);
                                playback = Playback::new("MP3","MP3 Playback",None,rate); //Acc. to sample rate of the song, create a new Playback
                                
                                //We can access the value inside mutex directly below because we access a field here in struct, Rust automatically dereference fields.
                                app_state.lock().unwrap().stopped = false; //the same with below 2 lines
                                //let mut mutex_guard = app_state.lock().unwrap(); 
                                //mutex_guard.stopped = false; //mutex guard = scoped lock => automatically unlocked when going out of scope.
                            }
                            Action::Stop => {}
                        }
                    }
                    //MutexGuard implements Deref, so we access the value by '*'.
                    else if *event_loop.playing.lock().unwrap(){
                        let mut written = false; //show it can play a sample
                        if let Some(ref mut source) = source{
                            let size = iter_to_buffer(source, &mut buffer); //take the value from the decoder and write them to buffer
                            if size > 0 {
                                playback.write(&buffer[..size]); //play the sounds on our sound card.
                                written = true;
                            }
                        }
                        if !written { //shows the end of the song
                            app_state.lock().unwrap().stopped = true;
                            *event_loop.playing.lock().unwrap() = false;
                            source = None;
                        }
                    }

                }
            });
        }
        Player { app_state, event_loop}
    }
}

fn iter_to_buffer<I:Iterator<Item = i16>>(iter: &mut I, buffer: &mut [[i16;2]; BUFFER_SIZE]) -> usize{
    let mut iter = iter.take(BUFFER_SIZE);
    let mut index = 0;
    while let Some(sample1) = iter.next(){
        if let Some(sample2) = iter.next(){
            buffer[index][0] = sample1;
            buffer[index][1] = sample2;
        }
        index += 1;
    }
    index
}