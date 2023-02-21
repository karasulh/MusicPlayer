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
//Before starting, some nodes for concurrency in Rust:
//Send and Sync traits prevent us from doing data races. A type implements the Send trait is safe to be sent to another thread.
//For example, Rc doesnot implement Send trait, so it couldnot be used with threads.
//Instead, Arc could be used with threads because it is for atomic operations.
//A  type implements the Sync trait is safe to be shared with multiple threads.
//For example, Mutex implements Sync trait, so it satisfies mutually exclusive.

//For SegQueue, we can say that it uses atomic operations and it is lock-free data structures, so we dont need to use Mutex 
//to mutate this value in mutable threads at the same time 
//but we still need to wrap this queue in an Arc to be able to share it with multiple threads. //by Rust Programming By Example(Packt Publishing,2018)
