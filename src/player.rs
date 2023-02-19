use std::fs::File;
use std::io::BufReader;
use std::path::{Path,PathBuf};
use std::sync::{Arc,Condvar,Mutex};
use std::thread;

use crossbeam::queue::SegQueue;
use pulse_simple::Playback;

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


