use std::fs::File;

use bw_img::{iter_direction::Horizontal, BWByteData, IterOutput};
use bw_img::file::video;
use clap::Parser;
use flate2::{write::ZlibEncoder, Compression};
use indicatif::{ProgressBar, ProgressStyle};

#[derive(clap::Parser)]
struct Args {
    #[command(subcommand)]
    commands: Commands,
}

#[derive(clap::Args)]
#[group(id = "input-type", required = true, multiple = false)]
struct ConvertType {
    /// read as video file
    #[cfg(feature = "video")]
    #[arg(short, long)]
    video: bool,
    /// read as image file
    #[cfg(feature = "image")]
    #[arg(short, long)]
    image: bool,
}

#[derive(clap::Args)]
struct ConvertArgs {
    #[clap(flatten)]
    typ: ConvertType,
    #[arg(required = true, index = 1)]
    path: String,
    /// Output file
    #[arg(short, long)]
    output: Option<String>,
    /// Output height
    #[arg(long)]
    height: Option<u32>,
    /// Output width
    #[arg(long)]
    width: Option<u32>,
}

#[derive(clap::Args)]
struct ShowArgs {
    /// zipped bw imgs file path
    #[arg(required = true)]
    path: String,
    /// Index of the image to show
    #[arg(short, long, default_value_t = 0)]
    index: usize,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// Convert video or image to zipped bw imgs
    Convert(ConvertArgs),
    /// Show bw image from zipped bw imgs
    Show(ShowArgs),
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    match args.commands {
        Commands::Convert(args) => convert(args),
        Commands::Show(args) => show(args),
    }
}

fn convert(args: ConvertArgs) -> anyhow::Result<()> {
    #[cfg(feature = "video")]
    if args.typ.video {
        let mut file = File::create(args.output.unwrap_or("output.imgs".into()))?;
        let mut encoder = ZlibEncoder::new(&mut file, Compression::best());
        println!("Processing video...");

        let vid_iter = video::convert_video(&args.path, args.width, args.height)?;
        println!("Total frames: {}", vid_iter.frame_count);
        println!(
            "Input resolution: {}x{}",
            vid_iter.input_size.0, vid_iter.input_size.1
        );
        println!(
            "Output resolution: {}x{}",
            vid_iter.output_size.0, vid_iter.output_size.1
        );
        println!("Video duration: {}s", vid_iter.duration);
        println!("Frame rate: {}", vid_iter.frame_rate);
        println!("Converting...");

        let progress = ProgressBar::new(vid_iter.frame_count);
        progress.set_style(
            ProgressStyle::with_template("{bar:40} {pos}/{len} | {elapsed}, eta: {eta}").unwrap(),
        );
        let mut skipped = 0;
        for result in vid_iter {
            let (f, s) = result?;
            progress.inc(f.len() as u64);
            skipped += s;

            for ele in f {
                ele.encode_as_file(&mut encoder)?;
            }
        }
        encoder.finish()?;
        progress.finish();
        println!("Finished with {} frames skipped.", skipped);

        Ok(())
    } else {
        unimplemented!()
    }
}

fn show(args: ShowArgs) -> anyhow::Result<()> {
    let mut file = std::fs::File::open(&args.path)?;
    println!("Decompressing {}...", args.path);

    fn write_pixel(is_white: bool) {
        if is_white {
            print!("██");
        } else {
            print!("  ");
        }
    }

    let mut imgs = bw_img::file::compress::decompress_imgs(&mut file);
    match imgs.nth(args.index) {
        Some(img) => {
            for ele in img?.iterator(Horizontal) {
                match ele {
                    IterOutput::Byte { byte, len } => {
                        for ele in byte.bw_byte_iter(len) {
                            write_pixel(ele);
                        }
                    }
                    IterOutput::NewLine => println!(),
                }
            }
        }
        None => {
            eprintln!("Invalid image index!");
        }
    }

    Ok(())
}
