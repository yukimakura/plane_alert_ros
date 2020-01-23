extern crate portaudio;

use std::sync::{Mutex, Arc};
use rosrust;

use portaudio as pa;
use std::f64::consts::PI;

const CHANNELS: i32 = 2;
const NUM_SECONDS: i32 = 5;
const SAMPLE_RATE: f64 = 44_100.0;
const FRAMES_PER_BUFFER: u32 = 64;
const TABLE_SIZE: usize = 200;

mod msg {
    rosrust::rosmsg_include!(std_msgs/Float64);
}

fn main() {
    // Initialize node
    rosrust::init("listener");
    match run() {
        Ok(_) => {}
        e => {
            eprintln!("Example failed with the following: {:?}", e);
        }
    }
}

fn run() -> Result<(), pa::Error> {
    println!(
        "PortAudio Test: output sine wave. SR = {}, BufSize = {}",
        SAMPLE_RATE, FRAMES_PER_BUFFER
    );

    // Initialise sinusoidal wavetable.
    let mut left_phase = 0;
    let mut right_phase = 0;
    let mut hz_cnt = Arc::new(Mutex::new(0.0));

    let mut sub_hz_cnt = Arc::clone(&hz_cnt);

    let _subscriber_raii = rosrust::subscribe("/plane_probability", 100, move |v: msg::std_msgs::Float64| {

        let mut num = sub_hz_cnt.lock().unwrap();
        *num = v.data;

    }).unwrap();
    
    let mut calc_hz_cnt = Arc::clone(&hz_cnt);

    let pa = pa::PortAudio::new()?;

    let mut settings =
        pa.default_output_stream_settings(CHANNELS, SAMPLE_RATE, FRAMES_PER_BUFFER)?;
    settings.flags = pa::stream_flags::CLIP_OFF;

    let callback = move |pa::OutputStreamCallbackArgs { buffer, frames, .. }| {
        // println!("{}, raw {}",(1.0 - (&hz_cnt/100.0))*20.0,&hz_cnt);
        let mut idx = 0;
        let mut num = calc_hz_cnt.lock().unwrap();

        for _ in 0..frames {
            if *num < (100.0 - 68.3){
                buffer[idx] =  (left_phase as f64 / TABLE_SIZE as f64 * PI * (1.0 - (*num/100.0))*10.0).sin() as f32;
                buffer[idx + 1] = (right_phase as f64 / TABLE_SIZE as f64 * PI *  (1.0 - (*num/100.0))*10.0).sin() as f32;
            }else{
                buffer[idx] =  0.0;
                buffer[idx + 1] = 0.0;
            }

            left_phase += 1;
            if left_phase >= TABLE_SIZE {
                left_phase -= TABLE_SIZE;
            }
            right_phase += 1;
            if right_phase >= TABLE_SIZE {
                right_phase -= TABLE_SIZE;
            }
            idx += 2;
        }
        
        pa::Continue
    };

    let mut stream = pa.open_non_blocking_stream(settings, callback)?;

    stream.start()?;

    while rosrust::is_ok() {

        println!("{}",*hz_cnt.lock().unwrap());
        rosrust::spin();
    }

    stream.stop()?;
    stream.close()?;

    Ok(())
}
