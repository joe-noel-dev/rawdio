mod audio_buffer;
mod borrowed_audio_buffer;
mod buffer_pool;
mod mutable_borrowed_audio_buffer;
mod owned_audio_buffer;
mod sample_location;

pub use audio_buffer::AudioBuffer;
pub type BorrowedAudioBuffer<'a> = borrowed_audio_buffer::BorrowedAudioBuffer<'a>;
pub type MutableBorrowedAudioBuffer<'a> =
    mutable_borrowed_audio_buffer::MutableBorrowedAudioBuffer<'a>;
pub type OwnedAudioBuffer = owned_audio_buffer::OwnedAudioBuffer;
pub type SampleLocation = sample_location::SampleLocation;
pub type BufferPool = buffer_pool::BufferPool;
