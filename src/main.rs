use anyhow::{bail, Context, Result};
use clap::Parser;
use std::path::PathBuf;
use std::process::Command;

//comp to  target size

#[derive(Parser, Debug)]
#[command(name = "vcompress", version, about)]
struct Args {
    /// input
    input: PathBuf,

    /// output
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// targ. size
    #[arg(short, long, default_value_t = 9.0)]
    target_mb: f64,

    /// audio bitrate
    #[arg(short = 'a', long, default_value_t = 128)]
    audio_kbps: u32,
}


fn get_duration_secs(input: &PathBuf) -> Result<f64> {
    let output = Command::new("ffprobe")
        .args(["-v", "error", "-show_entries", "format=duration",
               "-of", "default=noprint_wrappers=1:nokey=1"])
        .arg(input)
        .output()
        .context("failed to run ffprobe. is it installed??")?;

    String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse::<f64>()
        .context("could not parse video duration")
}




fn main() -> Result<()> {
    let args = Args::parse();
    let duration = get_duration_secs(&args.input)?;
    println!("duration: {:.2}s", duration);

    let target_bits = args.target_mb * 8.0 * 1024.0 * 1024.0 * 0.98; // saftey margin (doubt itll be
                                                                     // needed, nts to remove later)
    let audio_bits = args.audio_kbps as f64 * 1000.0 * duration;
    let video_kbps = ((target_bits - audio_bits) / duration / 1000.0).max(100.0);
    println!("Target video bitrate: {video_kbps:.0}kbps");

    let output_path = args.output.clone().unwrap_or_else(|| {
        let stem = args.input.file_stem().unwrap().to_string_lossy();
        PathBuf::from(format!("{stem}-compressed.mp4"))
    });

    let status = Command::new("ffmpeg") // if ffmpeg does it, might as well use it.. despite how it
                                        // makes this entire project just a wrapper for it
        .args(["-y", "-i"]).arg(&args.input)
        .args(["-c:v", "libx264", "-b:v", &format!("{video_kbps}k"),
               "-pass", "1", "-an", "-f", "mp4"])
        .arg("/dev/null")
        .status()
        .context("ffmpeg pass 1 failed to run")?;
    if !status.success() { bail!("ffmpeg pass 1 exited with an error"); }

    let status = Command::new("ffmpeg")
        .args(["-y", "-i"]).arg(&args.input)
        .args(["-c:v", "libx264", "-b:v", &format!("{video_kbps}k"),
               "-pass", "2", "-c:a", "aac", "-b:a", &format!("{}k", args.audio_kbps)])
        .arg(&output_path)
        .status()
        .context("ffmpeg pass 2 failed to run")?;
    if !status.success() { bail!("ffmpeg pass 2 exited with an error"); }

    println!("Wrote {}", output_path.display());
    Ok(())
}
