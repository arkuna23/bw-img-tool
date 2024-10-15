use anyhow::anyhow;
use bw_img::{BWImage, RgbImage};
use ffmpeg_next::{
    codec::Context as CodecContext,
    format::{self, Pixel},
    frame::Video,
    software::scaling::{Context as ScalingContext, Flags},
};

const BW_THRESHOLD: u8 = 128;

fn process_frames(
    decoder: &mut ffmpeg_next::decoder::Video,
    scaler: &mut ScalingContext,
    frames: &mut Vec<BWImage>,
) -> anyhow::Result<()> {
    let mut decoded = Video::empty();
    while decoder.receive_frame(&mut decoded).is_ok() {
        let mut scalled = Video::empty();
        scaler.run(&decoded, &mut scalled)?;

        let img = BWImage::parse(RgbImage::new(
            decoded.data(0),
            scaler.output().width,
            scaler.output().height,
        ))
        .map_err(|e| anyhow!("{}", e.to_string()))?;
        frames.push(img);
    }

    Ok(())
}

pub fn transform_vid(path: &str, width: Option<u32>, height: Option<u32>) -> anyhow::Result<Vec<BWImage>> {
    ffmpeg_next::init()?;
    let mut ictx = format::input(path)?;
    let stream = ictx
        .streams()
        .best(ffmpeg_next::media::Type::Video)
        .ok_or_else(|| anyhow::anyhow!("No video stream"))?;
    let vid_index = stream.index();
    let mut decoder = CodecContext::from_parameters(stream.parameters())?
        .decoder()
        .video()?;
    let mut scaler = ScalingContext::get(
        decoder.format(),
        decoder.width(),
        decoder.height(),
        Pixel::RGB24,
        width.unwrap_or_else(|| decoder.width()),
        height.unwrap_or_else(|| decoder.height()),
        Flags::BILINEAR,
    )?;

    let mut frames = vec![];
    for (stream, pack) in ictx.packets() {
        if stream.index() == vid_index {
            decoder.send_packet(&pack)?;
            process_frames(&mut decoder, &mut scaler, &mut frames)?;
        }
    }

    Ok(frames)
}
