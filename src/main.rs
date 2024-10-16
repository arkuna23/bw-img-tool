use std::fs::File;

use bw_img::{BWByteData, IterDirection, IterOutput};
use clap::Parser;
use crossterm::{style, ExecutableCommand};
use flate2::{write::GzEncoder, Compression};

#[cfg(feature = "image")]
mod img;
#[cfg(feature = "video")]
mod vid;

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
        let mut e = GzEncoder::new(&mut file, Compression::best());

        if let Err(e) = vid::transform_vid(&args.path, args.width, args.height, &mut e) {
            eprintln!("Error occurred while processing video.");
            Err(e)
        } else {
            e.finish()?;
            Ok(())
        }
    } else {
        unimplemented!()
    }
}

fn show(args: ShowArgs) -> anyhow::Result<()> {
    let mut file = std::fs::File::open(&args.path)?;
    println!("Decompressing {}...", args.path);
    let imgs = bw_img::file::zip::decompress_imgs(&mut file)?;

    fn write_pixel(stdout: &mut std::io::Stdout, is_white: bool) -> anyhow::Result<()> {
        if is_white {
            stdout.execute(style::SetBackgroundColor(style::Color::White))?;
        } else {
            stdout.execute(style::SetForegroundColor(style::Color::Black))?;
        }

        print!("  ");
        Ok(())
    }

    match imgs.get(args.index) {
        Some(img) => {
            let mut stdout = std::io::stdout();

            for ele in img.iterator(IterDirection::Vertical) {
                match ele {
                    IterOutput::Byte { byte, len } => {
                        for ele in byte.bw_byte_iter(len) {
                            write_pixel(&mut stdout, ele)?;
                        }
                    },
                    IterOutput::NewLine => println!()
                }
            }
        }
        None => {
            eprintln!("Invalid image index!");
        }
    }

    Ok(())
}
