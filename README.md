# waverly
[![docs.rs](https://docs.rs/waverly/badge.svg)](https://docs.rs/waverly)

Waverly is a Rust library that allows for easy parsing and writing of WAV files.

## TODO
- [x] Parse/read and write WAV files
- [x] FORMAT chunk
- [x] DATA chunk
- [x] PEAK chunk
- [x] FACT chunk
- [x] `no_std` support
- [ ] CUE POINT chunk
- [ ] PLAYLIST chunk
- [ ] Support PEAK chunk when channels are not equal to 2
- [ ] Better support for extensible modes
- [ ] Better error messages when binary doesn't align with chunks
- [ ] Tests for additional chunks and extensible modes


## Further reading
[Multimedia Programming Interface and Data Specifications, starting on page 56](http://www-mmsp.ece.mcgill.ca/Documents/AudioFormats/WAVE/Docs/riffmci.pdf)
