extern crate rspotify;
extern crate serial;
extern crate robust_arduino_serial;
 
use rspotify::client::SpotifyBuilder;
use rspotify::model::{
    Id, PlayableItem, TimeInterval
};
use rspotify::oauth2::{CredentialsBuilder, OAuthBuilder};
use rspotify::scopes;

use std::io;
use std::time::Duration;
use std::thread;
use std::convert::TryFrom;
use serial::prelude::*;
//use robust_arduino_serial::*;


pub fn create_serial_client() -> Result<serial::SystemPort, &'static str> {
    // Default settings of Arduino
    // see: https://www.arduino.cc/en/Serial/Begin
    const SETTINGS: serial::PortSettings = serial::PortSettings {
        baud_rate:    serial::Baud115200,
        char_size:    serial::Bits8,
        parity:       serial::ParityNone,
        stop_bits:    serial::Stop1,
        flow_control: serial::FlowNone,
    };

    // TODO: Make this (and others) an environment variable.
    let serial_port = "/dev/cu.usbmodem1422101";
    println!("Opening port: {:?}", serial_port);
    let mut port = serial::open(&serial_port).unwrap();
    port.configure(&SETTINGS).unwrap();
    // timeout of 30s
    port.set_timeout(Duration::from_secs(20)).unwrap();
    let i = 0; 
    //loop
    //{
    //    println!("Waiting for Arduino...");
    //    buffer = [0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x00];
    //    port.write(&buffer).unwrap();
    //    let mut read_buffer = [0u8; 1];
    //    port.read_exact(&mut read_buffer)?;
    //    if read_buffer[0] as u8 == 0xff:
    //    {
    //        break;
    //    }
    //    if i > 10 {
    //        return Err("Stopped after 10 tries")
    //    }
    //    thread::sleep(Duration::from_secs(1));
    //}

    println!("Connected to Arduino");
    Ok(port)
}

pub fn convert_f32_to_ms(time_sec: f32) -> Result<i32, i32> {
    Ok((time_sec*1000.) as i32)
}

pub fn write_beat_info<T: io::Read + io::Write>(file: &mut T, beats: Vec<TimeInterval>, progress: std::time::Duration) -> std::io::Result<usize> {
    let mut ms_into_beat = i32::try_from(progress.as_millis()).unwrap();
    let ms_from_beat:i32;
    let mut i = 0;
    loop {
        // TODO: Add error handling for when the milliseconds is longer that the total length of
        // song.
        // TODO: convert to class, keep track of progress in beats vector.
        let duration_ms = convert_f32_to_ms(beats[i].duration).unwrap();
        if ms_into_beat < duration_ms {
            ms_from_beat = duration_ms - ms_into_beat;
            break
        }
        ms_into_beat -= duration_ms;
        i += 1;
    }
    println!("Duration of beat {:?}, {:?}", i, beats[i].duration);
    println!("Milliseconds into beat {:?}, {:?}", i, ms_into_beat);
    const HIGH:i32 = 255;
    let buffer = [
        0x61,
        0x7a,
        (ms_from_beat & HIGH) as u8,
        (ms_from_beat >> 8 & HIGH) as u8,
        (ms_into_beat & HIGH) as u8,
        (ms_into_beat >> 8 & HIGH) as u8,
        0x61,
        0x7a
    ];
    println!("Buffer: {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?}", buffer[0], buffer[1], buffer[2], buffer[3], buffer[4], buffer[5], buffer[6], buffer[7]);
   
   let num_bytes = file.write(&buffer)?;
   i=0;
    loop
    {
        println!("Waiting for Arduino...");
        let mut read_buffer = [0u8; 8];
        file.read_exact(&mut read_buffer)?;
        println!("Buffer: {:?}", read_buffer);
        if read_buffer[0] as u8 > 0x00 {
            break;
        }
        if i > 10 {
            return Err(io::Error::new(io::ErrorKind::NotConnected, "Stopped after 10 tries"));
        }
        i+=1;
        thread::sleep(Duration::from_millis(10));
    }
   Ok(num_bytes)
}


#[tokio::main]
async fn main() {
    // You can use any logger for debugging.
    env_logger::init();

    // Set RSPOTIFY_CLIENT_ID, RSPOTIFY_CLIENT_SECRET and
    // RSPOTIFY_REDIRECT_URI in an .env file or export them manually:
    //
    // export RSPOTIFY_CLIENT_ID="your client_id"
    // export RSPOTIFY_CLIENT_SECRET="secret"
    //
    // These will then be read with `from_env`.
    //
    // Otherwise, set client_id and client_secret explictly:
    //
    //let creds = CredentialsBuilder::default()
    //     .id("cf52e2571f4a4133b0fb23115f627605")
    //     .secret("cca301bc68df4b808758f92f45dd27cc")
    //     .build()
    //     .unwrap();
    //let creds = CredentialsBuilder::from_env().build().unwrap();

    // The credentials must be available in the environment. Enable
    // `env-file` in order to read them from an `.env` file.
    let creds = CredentialsBuilder::from_env().build().unwrap_or_else(|_| {
        panic!(
            "No credentials configured. Make sure that either the \
            `env-file` feature is enabled, or that the required \
            environment variables are exported (`RSPOTIFY_CLIENT_ID`, \
            `RSPOTIFY_CLIENT_SECRET`)."
        )
    });

    let scope = scopes!(
        "user-library-read",
        "user-read-currently-playing",
        "user-read-playback-state"
    );
    // Using every possible scope
    let oauth = OAuthBuilder::from_env().scope(scope).build().unwrap();

    let mut spotify = SpotifyBuilder::default()
        .credentials(creds)
        .oauth(oauth)
        .build()
        .unwrap();

    let mut file = create_serial_client().unwrap();
    
    // Obtaining the access token
    spotify.prompt_for_user_token().await.unwrap();

    // Running the requests
    let played = spotify.current_playing(None, None::<&[_]>).await.unwrap().unwrap();
    
        // Debugging code.

        //let names: Vec<_> = track.artists.iter().map(|x| &x.name).collect();
        //println!("Song name: {:?}", track.name);
        //println!("Song uri: {:?}", &track.uri);
        //println!("Artist names: {:?}", names);

        //let features = spotify.track_features(Id::from_uri(&track.uri).unwrap()).await.unwrap();
        //let analysis = spotify.track_analysis(Id::from_uri(&track.uri).unwrap()).await.unwrap();
        //let mut i = 0;
        //while analysis.bars[i].start + analysis.bars[i].duration <=  played.progress.unwrap().as_secs_f32() {
        //    i += 1;
        //}
        //println!("Current duration in: {:?}", played.progress);
        //if i+1 < analysis.bars.len() {
        //    println!("Next bar: {:?}", analysis.bars[i+1].start);
        //}
        
    loop {
        let played = spotify.current_playing(None, None::<&[_]>).await.unwrap().unwrap();
        if let PlayableItem::Track(track) = played.item.unwrap() {
            let analysis = spotify.track_analysis(Id::from_uri(&track.uri).unwrap()).await.unwrap();
            write_beat_info(&mut file, analysis.beats, played.progress.unwrap()).unwrap();
        }
        thread::sleep(Duration::from_millis(1000));
    }
    //let history = spotify.current_user_recently_played(Some(10)).await;

    //println!("Response: {:?}", played);

}
