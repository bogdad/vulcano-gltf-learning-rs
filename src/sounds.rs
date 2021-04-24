use rodio::{Decoder, OutputStream, OutputStreamHandle, source::Source};

use std::fs::File;
use std::io::BufReader;

pub struct Sounds {
  stream: OutputStream,
  stream_handle: OutputStreamHandle,
  sources: Vec<Decoder<BufReader<File>>>,
}

impl Sounds {
  pub fn new() -> Self {
    let mut sources = vec![];

    // Get a output stream handle to the default physical sound device
    let (stream, stream_handle) = OutputStream::try_default().unwrap();
    // Load a sound from a file, using a path relative to Cargo.toml
    let file = BufReader::new(File::open("assets/sounds/huggy13ear__wind-2.wav").unwrap());
    // Decode that sound file into a source
    let source = Decoder::new(file).unwrap();
    sources.push(source);

    Sounds {
      stream,
      stream_handle,
      sources,
    }
  }

  pub fn play(&mut self) {
    while !self.sources.is_empty() {
      let source = self.sources.remove(0);
      self.stream_handle.play_raw(source.convert_samples());
    }
  }
}
