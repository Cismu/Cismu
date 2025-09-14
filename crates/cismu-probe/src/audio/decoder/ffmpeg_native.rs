use ff::codec::threading::{Config as ThreadConfig, Type as ThreadingType};
use ff::media;
use ff::util::channel_layout::ChannelLayout;
use ff::util::error::{EINVAL, Error as FfmpegError};
use ff::util::format::sample::{Sample, Type as SampleType};
use ffmpeg_next as ff;
use std::{path::Path, sync::mpsc, thread};

use crate::audio::{AudioDecoder, PcmStream, StreamInfo};
use crate::error::Error;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FFmpegNativeError {
    #[error("unsupported operation: {0}")]
    Unsupported(&'static str),
    #[error("ffmpeg init failed")]
    FfmpegInit,
    #[error("ffmpeg: failed to open input")]
    OpenInput,
    #[error("no audio stream found")]
    NoAudioStream,
    #[error("ffmpeg: failed to create codec context")]
    CodecContext,
    #[error("ffmpeg: no audio decoder available")]
    NoAudioDecoder,
    #[error("ffmpeg: failed to create resampling context (swr)")]
    SwrContext,
    #[error("ffmpeg: error while sending packet to the decoder")]
    SendPacket,
    #[error("ffmpeg: error while receiving frame from the decoder")]
    ReceiveFrame,
    #[error("ffmpeg: resample (swr.run) failed")]
    ResampleRun,
    #[error("decoder worker channel closed")]
    ChannelClosed,
}

#[cfg(feature = "ffmpeg")]
pub struct FFmpegNativeDecoder;

#[cfg(feature = "ffmpeg")]
impl FFmpegNativeDecoder {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(feature = "ffmpeg")]
impl AudioDecoder for FFmpegNativeDecoder {
    fn open(&self, path: &Path) -> Result<Box<dyn PcmStream + Send>, Error> {
        FFmpegPcmStream::open_native(path).map(|s| Box::new(s) as Box<dyn PcmStream + Send>)
    }
}

#[cfg(feature = "ffmpeg")]
pub struct FFmpegPcmStream {
    rx: mpsc::Receiver<Vec<f32>>,
    info: StreamInfo,
    eof: bool,
}

#[cfg(feature = "ffmpeg")]
impl FFmpegPcmStream {
    pub fn open_native(path: &Path) -> Result<Self, Error> {
        ff::init().map_err(|_| FFmpegNativeError::FfmpegInit)?;
        let mut ictx = ff::format::input(path).map_err(|_| FFmpegNativeError::OpenInput)?;

        let input = ictx
            .streams()
            .best(media::Type::Audio)
            .ok_or(FFmpegNativeError::NoAudioStream)?;

        let mut ctx = ff::codec::context::Context::from_parameters(input.parameters())
            .map_err(|_| FFmpegNativeError::CodecContext)?;
        ctx.set_threading(ThreadConfig {
            kind: ThreadingType::Frame,
            count: 0,
        });

        let mut dec = ctx.decoder().audio().map_err(|_| FFmpegNativeError::NoAudioDecoder)?;

        let in_ch = dec.channels() as u16;
        let in_rate = dec.rate();
        let in_layout = if dec.channel_layout().is_empty() {
            ChannelLayout::default(in_ch.into())
        } else {
            dec.channel_layout()
        };
        dec.set_channel_layout(in_layout);

        let in_sample_fmt = dec.format();
        let out_sample_fmt = Sample::F32(SampleType::Packed);

        let mut swr = ff::software::resampling::context::Context::get(
            in_sample_fmt,
            in_layout,
            in_rate,
            out_sample_fmt,
            in_layout,
            in_rate,
        )
        .map_err(|_| FFmpegNativeError::SwrContext)?;

        let (tx, rx) = mpsc::channel::<Vec<f32>>();
        let stream_index = input.index();

        thread::spawn(move || {
            for (s, packet) in ictx.packets() {
                if s.index() != stream_index {
                    continue;
                }

                if let Err(e) = dec.send_packet(&packet) {
                    match e {
                        FfmpegError::Other { errno: EINVAL } => {
                            let _ = tx.send(Vec::new());
                            return;
                        }
                        FfmpegError::Eof => break,
                        _ => continue,
                    }
                }
                loop {
                    let mut decoded = ff::frame::Audio::empty();
                    match dec.receive_frame(&mut decoded) {
                        Ok(_) => {
                            let mut out = ff::frame::Audio::empty();
                            if swr.run(&decoded, &mut out).is_err() {
                                continue;
                            }
                            let samples = out.samples();
                            if samples == 0 {
                                continue;
                            }
                            let needed = ff::util::format::sample::Buffer::size(out_sample_fmt, in_ch, samples, false);
                            let mut chunk = Vec::<f32>::with_capacity(needed / 4);
                            let bytes = &out.data(0)[..needed];
                            for b in bytes.chunks_exact(4) {
                                let mut a = [0u8; 4];
                                a.copy_from_slice(b);
                                chunk.push(f32::from_le_bytes(a));
                            }
                            if tx.send(chunk).is_err() {
                                return;
                            }
                        }
                        Err(_) => break,
                    }
                }
            }

            let _ = dec.send_packet(&ff::codec::packet::Packet::empty());
            loop {
                let mut decoded = ff::frame::Audio::empty();
                match dec.receive_frame(&mut decoded) {
                    Ok(_) => {
                        let mut out = ff::frame::Audio::empty();
                        if swr.run(&decoded, &mut out).is_ok() && out.samples() > 0 {
                            let needed =
                                ff::util::format::sample::Buffer::size(out_sample_fmt, in_ch, out.samples(), false);
                            let mut chunk = Vec::<f32>::with_capacity(needed / 4);
                            let bytes = &out.data(0)[..needed];
                            for b in bytes.chunks_exact(4) {
                                let mut a = [0u8; 4];
                                a.copy_from_slice(b);
                                chunk.push(f32::from_le_bytes(a));
                            }
                            if tx.send(chunk).is_err() {
                                return;
                            }
                        }
                    }
                    Err(_) => break,
                }
            }
            let _ = tx.send(Vec::new());
        });

        Ok(Self {
            rx,
            info: StreamInfo {
                sample_rate: in_rate,
                channels: in_ch,
            },
            eof: false,
        })
    }
}

#[cfg(feature = "ffmpeg")]
impl PcmStream for FFmpegPcmStream {
    fn next_chunk(&mut self) -> Result<Option<Vec<f32>>, Error> {
        if self.eof {
            return Ok(None);
        }
        match self.rx.recv() {
            Ok(chunk) if chunk.is_empty() => {
                self.eof = true;
                Ok(None)
            }
            Ok(chunk) => Ok(Some(chunk)),
            Err(_) => {
                self.eof = true;
                Err(FFmpegNativeError::ChannelClosed.into())
            } // ← se convierte a Error
        }
    }

    fn format(&self) -> Option<StreamInfo> {
        Some(self.info)
    }
}
