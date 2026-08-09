use anyhow::{bail, Context, Result};
use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use std::io::{BufRead, BufReader, Read};
use std::path::PathBuf;
use std::process::{Command, Stdio};

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

    if !output.status.success() {
        bail!("ffprobe failed: {}", String::from_utf8_lossy(&output.stderr).trim());
    }

    String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse::<f64>()
        .context("could not parse video duration")
}



fn null_device() -> &'static str {
    if cfg!(windows) { "NUL" } else { "/dev/null" }
}

/// progress bar :3
fn run_ffmpeg_with_progress(mut cmd: Command, duration: f64, label: &str) -> Result<()> {
    cmd.args(["-progress", "pipe:1", "-nostats", "-loglevel", "error"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = cmd.spawn().with_context(|| format!("failed to start ffmpeg ({label})"))?;

    let stderr = child.stderr.take().expect("stderr was piped");
    let stderr_handle = std::thread::spawn(move || {
        let mut buf = String::new();
        let _ = BufReader::new(stderr).read_to_string(&mut buf);
        buf
    });

    let pb = ProgressBar::new((duration * 1000.0).round() as u64);
    pb.set_style(
        ProgressStyle::with_template("{msg} [{bar:40.cyan/blue}] {percent:>3}% ({elapsed_precise})")
            .unwrap()
            .progress_chars("=> "),
    );
    pb.set_message(label.to_string());

    let stdout = child.stdout.take().expect("stdout was piped");
    for line in BufReader::new(stdout).lines().map_while(Result::ok) {
        if let Some(value) = line.strip_prefix("out_time_us=") {
            if let Ok(us) = value.trim().parse::<i64>() {
                let secs = (us.max(0) as f64 / 1_000_000.0).min(duration);
                pb.set_position((secs * 1000.0).round() as u64);
            }
        } else if line.starts_with("progress=end") {
            pb.set_position(pb.length().unwrap_or(0));
        }
    }

    let status = child.wait().context("failed to wait on ffmpeg")?;
    pb.finish_and_clear();

    let stderr_output = stderr_handle.join().unwrap_or_default();
    if !status.success() {
        if !stderr_output.trim().is_empty() {
            eprintln!("{}", stderr_output.trim());
        }
        bail!("ffmpeg {label} exited with an error");
    }

    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();

    if args.target_mb <= 0.0 {
        bail!("--target-mb must be positive (got {})", args.target_mb);
    }
    if args.audio_kbps == 0 {
        bail!("--audio-kbps must be positive (got 0)");
    }

    let duration = get_duration_secs(&args.input)?;
    if duration <= 0.0 {
        bail!("video duration must be positive (got {duration:.2}s)");
    }
    println!("duration: {:.2}s", duration);

    let target_bits = args.target_mb * 8.0 * 1024.0 * 1024.0 * 0.98; // saftey margin (likely not needed but still ig)
    let audio_bits = args.audio_kbps as f64 * 1000.0 * duration;
    if audio_bits >= target_bits {
        bail!(
            "audio alone (~{:.1}MB at {}kbps) leaves no room in a {:.1}MB target; lower --audio-kbps or raise --target-mb",
            audio_bits / 8.0 / 1024.0 / 1024.0,
            args.audio_kbps,
            args.target_mb
        );
    }
    let mut video_kbps = (target_bits - audio_bits) / duration / 1000.0;
    if video_kbps < 100.0 {
        eprintln!("warning: computed video bitrate ({video_kbps:.0}kbps) is very low, using 100kbps floor instead");
        video_kbps = 100.0;
    }
    println!("Target video bitrate: {video_kbps:.0}kbps");

    let output_path = args.output.clone().unwrap_or_else(|| {
        let stem = args.input.file_stem().unwrap_or_default().to_string_lossy();
        PathBuf::from(format!("{stem}-compressed.mp4"))
    });
    if output_path == args.input {
        bail!("output path is the same as the input path ({}); choose a different output", output_path.display());
    }

    let mut pass1_cmd = Command::new("ffmpeg"); // if ffmpeg does it, might as well use it.. despite how it
                                                 // makes this entire project just a wrapper for it
    pass1_cmd
        .args(["-y", "-i"]).arg(&args.input)
        .args(["-c:v", "libx264", "-b:v", &format!("{video_kbps}k"),
               "-pass", "1", "-an", "-f", "mp4"])
        .arg(null_device());

    let mut pass2_cmd = Command::new("ffmpeg");
    pass2_cmd
        .args(["-y", "-i"]).arg(&args.input)
        .args(["-c:v", "libx264", "-b:v", &format!("{video_kbps}k"),
               "-pass", "2", "-c:a", "aac", "-b:a", &format!("{}k", args.audio_kbps)])
        .arg(&output_path);

    let result = (|| -> Result<()> {
        run_ffmpeg_with_progress(pass1_cmd, duration, "pass 1/2")?;
        run_ffmpeg_with_progress(pass2_cmd, duration, "pass 2/2")?;
        Ok(())
    })();

    let _ = std::fs::remove_file("ffmpeg2pass-0.log");
    let _ = std::fs::remove_file("ffmpeg2pass-0.log.mbtree");

    result?;

    println!("Wrote {}", output_path.display());
    Ok(())
}
