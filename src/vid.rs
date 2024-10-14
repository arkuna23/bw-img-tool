use ffmpeg_next::{
    codec::Context as CodecContext,
    format::{self, Pixel},
    frame::Video,
    software::scaling::{Context as ScalingContext, Flags},
};

use crate::bw::BWImage;

const BW_THRESHOLD: u8 = 128;

fn process_frames(
    decoder: &mut ffmpeg_next::decoder::Video,
    scaler: &mut ScalingContext,
) -> anyhow::Result<Vec<Vec<u8>>> {
    let mut decoded = Video::empty();
    while decoder.receive_frame(&mut decoded).is_ok() {
        let mut scalled = Video::empty();
        scaler.run(&decoded, &mut scalled)?;
    }
}

fn to_bw_bytes(data: &[u8]) -> Vec<u8> {
    data.chunks(3 * 8)
        .map(|c| {
            // 8 bits per byte, one bit presents one pixel, high bit is the first pixel
            let mut bw_bit = 0u8;
            for (i, bit) in c.chunks(3).enumerate() {
                let gray_value =
                    (0.299 * bit[0] as f32 + 0.587 * bit[1] as f32 + 0.114 * bit[2] as f32) as u8;
                if gray_value > BW_THRESHOLD {
                    bw_bit |= 1 << i;
                }
            }
            bw_bit
        })
        .collect()
}

pub fn transform_vid(path: &str, width: u32, height: u32) -> anyhow::Result<Vec<BWImage>> {
    ffmpeg_next::init()?;
    let ictx = format::input(path)?;
    let stream = ictx
        .streams()
        .best(ffmpeg_next::media::Type::Video)
        .ok_or_else(|| anyhow::anyhow!("No video stream"))?;
    let vid_index = stream.index();
    let decoder = CodecContext::from_parameters(stream.parameters())?
        .decoder()
        .video()?;
    let mut scaler = ScalingContext::get(
        decoder.format(),
        decoder.width(),
        decoder.height(),
        Pixel::RGB24,
        width,
        height,
        Flags::BILINEAR,
    )?;

    let frames = vec![];
    for (stream, pack) in ictx.packets() {
        if stream.index() == vid_index {
            decoder.send_packet(packet)?;
        }
    }
}
