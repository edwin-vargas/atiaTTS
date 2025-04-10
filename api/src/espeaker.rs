use espeaker;
use rodio::{OutputStream, Sink};

pub fn generate() {
    let speaker = espeaker::Speaker::new();
    let source = speaker.speak("Hello, world!");
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();
    sink.append(source);
    sink.sleep_until_end();
}
