use anyhow::anyhow;
use bw_img::{BWDataErr, BWImage, RgbData};
use ffmpeg_next::{
    codec::Context as CodecContext,
    format::{self, Pixel},
    frame::Video,
    software::scaling::{Context as ScalingContext, Flags},
};
use indicatif::{ProgressBar, ProgressStyle};

fn process_frames<T: std::io::Write>(
    decoder: &mut ffmpeg_next::decoder::Video,
    scaler: &mut ScalingContext,
    out: &mut T,
) -> anyhow::Result<(u64, u64)> {
    let mut decoded = Video::empty();
    let mut count = 0;
    let mut skipped = 0;
    while decoder.receive_frame(&mut decoded).is_ok() {
        let mut scalled = Video::empty();
        scaler.run(&decoded, &mut scalled)?;

        let (width, height) = (scaler.output().width, scaler.output().height);
        let data = scalled.data(0);

        count += 1;
        let img = match BWImage::parse(&RgbData::new(data, width, height)) {
            Ok(r) => r,
            Err(e) => {
                if let BWDataErr::WrongSize(_, _, _) = e {
                    skipped += 1;
                    continue;
                } else {
                    Err(e)?
                }
            }
        };
        img.encode_as_file(out)?;
    }

    Ok((count, skipped))
}

pub fn transform_vid<T: std::io::Write>(
    path: &str,
    width: Option<u32>,
    height: Option<u32>,
    out: &mut T,
) -> anyhow::Result<(u64, u64)> {
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

    println!("Input resulution: {}x{}", decoder.width(), decoder.height());
    println!(
        "Output resulution: {}x{}",
        scaler.output().width,
        scaler.output().height
    );
    let frames = {
        let duration = stream.duration();
        let time_base = stream.time_base();
        let frame_rate = stream.avg_frame_rate();
        if duration == 0 {
            return Err(anyhow!("No frames"));
        }
        let secs = duration as f64 / (time_base.0 * time_base.1) as f64;
        let frame_rate = (frame_rate.0 / frame_rate.1) as f64;
        println!("Duration: {:.1}s", secs);
        println!("Frame rate: {}", frame_rate);
        secs * frame_rate
    };
    let mut processed = 0;
    let mut skipped = 0;

    let progress = ProgressBar::new(frames.round() as u64);
    progress.set_style(
        ProgressStyle::with_template("{bar:40} {pos}/{len} | {elapsed}, eta: {eta}").unwrap(),
    );
    for (stream, pack) in ictx.packets() {
        if stream.index() == vid_index {
            decoder.send_packet(&pack)?;
            let (proc, skip) = process_frames(&mut decoder, &mut scaler, out)?;
            progress.inc(proc + skip);
            processed += proc;
            skipped += skip;
        }
    }
    progress.finish();

    Ok((processed, skipped))
}
