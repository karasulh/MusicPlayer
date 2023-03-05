use std::io::{Read,Seek,SeekFrom};
use std::time::Duration;
use gio::File;
use symphonia::core::codecs::{CODEC_TYPE_NULL, DecoderOptions, CODEC_TYPE_MP3, Decoder};
//decode the frames of an MP3 file to suitabe format to be played by the OS
use symphonia::core::io::{MediaSourceStream, MediaSourceStreamOptions}; 
use symphonia::core::probe::Hint;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::formats::{FormatOptions, Track, FormatReader};
use symphonia::core::audio::{AudioBufferRef, SampleBuffer, SignalSpec};
use rodio::Source;

use crate::mp3;
//use to_millis; //reach the function in main since use statements are relative to the root of the crate

pub struct mp3Decoder{ //Mp3Decoder<R:Read> //trait bounds
    pub samples: Vec<f32>,
    pub decoder: Box<dyn Decoder>,
    pub format_reader: Box<dyn FormatReader>,
    pub sample_buffer:SampleBuffer<f32>,
    spec:SignalSpec,
    current_packet_frame_offset:usize,
}

//Sample,frame,packet: I understand that they means "each mp3 file". CHECK IT!!!

impl mp3Decoder {

    pub fn new(mut src:std::fs::File) -> Result<mp3Decoder,String>{

        //MediaSourceStream = std::io::BufReader
        let mss = MediaSourceStream::new(Box::new(src),Default::default());
        let mut hint = Hint::new(); //hint is used to give first impression to understand type of file
        hint.with_extension("mp3");
        let meta_opts: MetadataOptions = Default::default();
        let fmt_opts: FormatOptions = Default::default();
        //probe is using to understand type of file like mp3, mp4 ...
        let probed = symphonia::default::get_probe()
                                                        .format(&hint, mss, &fmt_opts, &meta_opts)
                                                        .expect("unsupported format");
        let mut format_reader = probed.format;//format reader: it has knowledge of the file format
        //a media can contain more than one track and for each track, a decoder must be instantiated. 
        //find the first audio track with a known decodeable codec
        //let track = format_reader.tracks().iter().find(|t| t.codec_params.codec != CODEC_TYPE_NULL).expect("no supported audio tracks");
        let track = format_reader.default_track().expect("There is not any track"); //It returns the first track
        if !is_mp3(track){
            return Err(String::from("It is not mp3"));
        }
        println!("{:?}",track.codec_params);
        let dec_opts: DecoderOptions = Default::default();
        let mut decoder = symphonia::default::get_codecs()
                                                                .make(&track.codec_params,&dec_opts)
                                                                .expect("unsupported codec");
        let mut samples = Vec::new();
        
        //Decoding Loop: Get packet from media format(container),consume any new metadata, filter packet, decode packet into audio samples
        let decoded_audio_buf = loop{
            let curr_packet_frame = format_reader.next_packet().unwrap();
            while !format_reader.metadata().is_latest(){
                format_reader.metadata().pop();
            }
            match decoder.decode(&curr_packet_frame){
                Ok(decoded_audio_buf) => {
                    /*
                    match decoded_audio_buf{
                        AudioBufferRef::F32(buf) => {
                            let planes = buf.planes(); //audio plane = channel
                            for  plane in planes.planes(){
                                for &sample in plane.iter(){
                                    samples.push(sample);
                                } 
                            }
                            break decoded_audio_buf;
                        }
                        _ => {}
                    }
                    */
                    break decoded_audio_buf;
                }
                Err(err) => {
                    println!("Error in decoding mp3: {:?}",err);
                    return Err(err.to_string());
                }
            }
        };
        let spec = decoded_audio_buf.spec().to_owned();
        let sample_buffer = get_buffer(decoded_audio_buf).unwrap();

        Ok(mp3Decoder{samples,decoder,format_reader,sample_buffer,spec,current_packet_frame_offset:0})
    }


}

fn is_mp3 (track:&Track) -> bool {
    let is_mp3 = track.codec_params.codec == CODEC_TYPE_MP3;
    is_mp3
}

fn get_buffer(decoded_audio_buf: AudioBufferRef)->Option<SampleBuffer<f32>>{
    
    let mut sample_count = 0;
    let mut sample_buf = None;

    //Do things below, if we need to be accessed in an interleaved order or converted into another sample format or byte buffer is required
    if sample_buf.is_none(){//create a sample buffer matching decoded audio buffer format.
        let spec = *decoded_audio_buf.spec();
        let duration = decoded_audio_buf.capacity() as u64;
        sample_buf = Some(SampleBuffer::<f32>::new(duration,spec));
        let mut interleaved_buf = sample_buf.unwrap();
        interleaved_buf.copy_interleaved_ref(decoded_audio_buf);//Copy decoded audio buffer into a sample buffer in interleaved order with f32 format
        sample_count += interleaved_buf.samples().len();
        return Some(interleaved_buf);
    }    
    return None;
}

//Source and Iterator must be implemented for "decoder" because "playback" needs these traits.
impl Source for mp3Decoder{
    fn current_frame_len(&self) -> Option<usize> {
        Some(self.sample_buffer.samples().len())
    }
    fn channels(&self) -> u16 {
        self.spec.channels.count() as u16
    }
    fn sample_rate(&self) -> u32 {
        self.spec.rate
    }
    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

impl Iterator for mp3Decoder{
    type Item = f32;

    fn next(&mut self) -> Option<f32>{
        if self.current_packet_frame_offset == self.sample_buffer.len(){
            let mut decode_errors: usize = 0;
            //Decoding Loop: Get packet from media format(container),consume any new metadata, filter packet, decode packet into audio samples
            let decoded_audio_buf = loop{
                let curr_packet_frame = self.format_reader.next_packet().unwrap();
                while !self.format_reader.metadata().is_latest(){//consume the metadatas
                    self.format_reader.metadata().pop(); 
                }
                match self.decoder.decode(&curr_packet_frame){
                    Ok(decoded_audio_buf) => {
                        /*
                        match decoded_audio_buf{
                            AudioBufferRef::F32(buf) => {
                                let planes = buf.planes(); //audio plane = channel
                                for  plane in planes.planes(){
                                    for &sample in plane.iter(){
                                        self.samples.push(sample);
                                    } 
                                }
                                break decoded_audio_buf;
                            }
                            _ => {}
                        }
                        */
                        break decoded_audio_buf;
                    }
                    Err(err) => {
                        println!("Error in decoding mp3: {:?}",err);
                        return None;
                    }
                }
            };
            self.sample_buffer = get_buffer(decoded_audio_buf).unwrap();   
            self.current_packet_frame_offset = 0;
        }
        let sample = self.sample_buffer.samples()[self.current_packet_frame_offset];
        self.current_packet_frame_offset += 1;
        Some(sample)
    }
}

