use std::io::{Read,Seek,SeekFrom};
use std::time::Duration;
use simplemad; //decode the frames of an MP3 file to suitabe format to be played by the OS
//use to_millis; //reach the function in main since use statements are relative to the root of the crate

pub struct Mp3Decoder<R> where R:Read{ //Mp3Decoder<R:Read> //trait bounds
reader: simplemad::Decoder<R>,
current_frame: simplemad::Frame,
current_frame_channel: usize,
current_frame_sample_pos: usize,
current_time: u64,
}

impl<R> Mp3Decoder<R> where R:Read + Seek{
    
    pub fn new(mut data: R) -> Result<Mp3Decoder<R>,R>{
        if !is_mp3(data.by_ref()){
            return Err(data);
        }
        let mut reader = simplemad::Decoder::decode(data).unwrap();

        let current_frame = next_frame(&mut reader);
        let current_time = to_millis(current_frame.duration);

        Ok(Mp3Decoder{reader,current_frame,current_frame_channel:0,current_frame_sample_pos:0,current_time})
    }

    pub fn current_time(&self) -> u64 {
        self.current_time
    }

    pub fn samples_rate(&self) -> u32 {
        self.samples_rate()
    }

    pub fn compute_duration(mut data:R) -> Option<Duration>{
        if !is_mp3(data.by_ref()){
            return None;
        }
        //We only need frame duration, so it is faster to decode only headers
        //decoder is iterator
        let decoder = simplemad::Decoder::decode_headers(data).unwrap();
        Some(decoder.filter_map(|frame|{ //filter_map can be used instead of map().filter().map()
            match frame{
                Ok(frame) => Some(frame.duration),
                Err(_) => None,
            }
        }).sum())
    }
}

impl<R> Iterator for Mp3Decoder<R> where R:Read{
    type Item = i16;

    fn next(&mut self) -> Option<i16>{ //The only required one to be implemented
        next_sample(self)
    }
    fn size_hint(&self) -> (usize, Option<usize>) { //Optional implementation
        (self.current_frame.samples[0].len(),None)
    }
}

fn to_millis(duration: Duration) -> u64{
    duration.as_secs() * 1000 + duration.subsec_nanos() as u64 / 1_000_000
}

fn is_mp3<R>(mut data:R) -> bool where R: Read+Seek {
    let stream_pos = data.seek(SeekFrom::Current(0)).unwrap();
    let is_mp3 = simplemad::Decoder::decode(data.by_ref()).is_ok();
    data.seek(SeekFrom::Start(stream_pos)).unwrap();//return to the beginning of the file
    is_mp3
}

//Option<Result<Frame,SimplemadError>> => Option<Frame> => Frame
fn next_frame<R: Read>(decoder: &mut simplemad::Decoder<R>) -> simplemad::Frame{
    decoder.find_map(|f|f.ok())//decoder.filter_map(|f| f.ok()).next()
        .unwrap_or_else(||{
        simplemad::Frame { sample_rate: 44100, bit_rate: 0, layer: Default::default(), mode: Default::default(), 
            samples: vec![Vec::new()], duration: Duration::from_secs(0), position: Duration::from_secs(0)}
        })
}

//Do some bit shifting to get the sample and fetch the next frame
fn next_sample<R:Read>(decoder: &mut Mp3Decoder<R>) -> Option<i16>{
    if decoder.current_frame.samples[0].len() == 0 {
        return None;
    }
    //getting the sample and converting it from fixed step to i16
    let sample = decoder.current_frame.samples[decoder.current_frame_channel][decoder.current_frame_sample_pos];
    let sample = sample.to_i32() + (1 << (28-16));
    let sample = if sample >= 0x10000000 {0x10000000 -1} else if sample <= -0x10000000 {-0x10000000} else {sample};
    let sample = sample >> (28+1-16);
    let sample = sample as i16;

    decoder.current_frame_channel += 1;
    if decoder.current_frame_channel < decoder.current_frame.samples.len(){
        return Some(sample);
    }

    decoder.current_frame_channel = 0;
    decoder.current_frame_sample_pos += 1;
    if decoder.current_frame_sample_pos < decoder.current_frame.samples[0].len(){
        return Some(sample);
    }

    decoder.current_frame = next_frame(&mut decoder.reader);
    decoder.current_frame_channel = 0;
    decoder.current_frame_sample_pos = 0;
    decoder.current_time += to_millis(decoder.current_frame.duration);

    return Some(sample);
}