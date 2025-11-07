//! Audio domain - audio file handling and playback

pub mod wav;
pub mod player;
pub mod manager;
pub mod sequence;

pub use wav::{WavFile, WavFormat, WavError};
pub use player::{AudioPlayer, AudioPlayerState, PlaybackOptions, StreamingAudioPlayer};
pub use manager::{AudioFileManager, AudioFileInfo, Language};
pub use sequence::{SequentialPlayer, SequenceBuilder};
