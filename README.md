# waverly
Waverly is a Rust library that allows for easy parsing and writing of WAV files with the primary
goal of providing access to all metadata within a WAV file, not just the format and data chunks.
It's secondary goal is to support `no_std`. If you only care about the data chunk already 
formatted as samples, there are plenty of [good alternatives](https://crates.io/search?q=wav).

```rust
use std::fs::File;
use std::io::Cursor;
use waverly::Wave;

fn main() -> Result<(), waverly::WaverlyError> {
    let file = File::open("./meta/16bit-2ch-float-peak.wav")?;
    let wave: Wave = Wave::from_reader(file)?;

    let mut virt_file = Cursor::new(Vec::new());
    wave.write(&mut virt_file)?;
    Ok(())
}
```


## TODO
- [x] Parse/read and write WAV files
- [x] FORMAT chunk
- [x] DATA chunk
- [x] PEAK chunk
- [x] FACT chunk
- [x] `no_std` support
- [ ] Single pass generation of samples in any bit depth
- [ ] Most metadata in WAV can be generated without user input, do so where possible on write.
- [ ] Feature to skip or target chunks
- [ ] CUE POINT chunk
- [ ] PLAYLIST chunk
- [ ] Support PEAK chunk when channels are not equal to 2
- [ ] Better support for extensible modes
- [ ] Better error messages when binary doesn't align with chunks
- [ ] ATests for additional chunks, extensible modes, `no_std`


## Further reading
[Multimedia Programming Interface and Data Specifications, starting on page 56](http://www-mmsp.ece.mcgill.ca/Documents/AudioFormats/WAVE/Docs/riffmci.pdf)
