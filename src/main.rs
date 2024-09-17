use std::fs::{self, Metadata};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::SystemTime;
use std::io;

/// Helper function to get file creation or modification time in seconds
fn get_file_timestamp(metadata: &Metadata) -> Option<SystemTime> {
    metadata.created().or_else(|_| metadata.modified()).ok()
}

/// Function to calculate the average framerate based on file timestamps
fn calculate_framerate(image_files: &[PathBuf]) -> Option<f64> {
    if image_files.len() < 2 {
        return None;
    }

    let mut time_diffs = Vec::new();
    
    for i in 1..image_files.len() {
        let meta_prev = fs::metadata(&image_files[i - 1]).ok()?;
        let meta_curr = fs::metadata(&image_files[i]).ok()?;

        let timestamp_prev = get_file_timestamp(&meta_prev)?.duration_since(SystemTime::UNIX_EPOCH).ok()?.as_secs_f64();
        let timestamp_curr = get_file_timestamp(&meta_curr)?.duration_since(SystemTime::UNIX_EPOCH).ok()?.as_secs_f64();
        
        let time_diff = timestamp_curr - timestamp_prev;
        time_diffs.push(time_diff);
    }

    let avg_time_diff = time_diffs.iter().sum::<f64>() / time_diffs.len() as f64;
    let framerate = 1.0 / avg_time_diff; // Framerate is 1 divided by the average time between frames

    Some(framerate)
}

/// Function to create a video from images in a directory, calculating the framerate
fn create_webm_from_images(event_id_dir: &Path, output_dir: &Path) -> io::Result<()> {
    let mut image_files: Vec<PathBuf> = fs::read_dir(event_id_dir)?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("jpg"))
        .collect();
    
    image_files.sort(); // Ensure the images are in the correct order

    // Calculate framerate based on timestamps
    let framerate = calculate_framerate(&image_files).unwrap_or(24.0);  // Default to 24 fps if framerate can't be calculated

    // Prepare the input pattern and output file path
    let images_pattern = event_id_dir.join("%d-capture.jpg");  // Input images
    let output_file = output_dir.join(format!(
        "{}-video.webm",
        event_id_dir.file_name().unwrap().to_string_lossy()
    ));

    // Run ffmpeg to combine images into a .webm video
    let status = Command::new("ffmpeg")
        .args(&[
            "-framerate", &framerate.to_string(),   // Use calculated framerate
            "-i", &images_pattern.to_string_lossy(),  // Input pattern
            "-c:v", "libvpx-vp9",             // Use VP9 codec for .webm
            "-pix_fmt", "yuv420p",            // Set pixel format
            &output_file.to_string_lossy()    // Output video file
        ])
        .status()?;

    if status.success() {
        println!("Created video: {:?}", output_file);
    } else {
        eprintln!("Failed to create video for {:?}", event_id_dir);
    }

    Ok(())
}

/// Main function to process all event directories
fn process_event_directories(base_dir: &Path) -> io::Result<()> {
    let videos_dir = base_dir.join("videos");
    fs::create_dir_all(&videos_dir)?;  // Ensure the "videos" directory exists

    // Iterate over all directories in the base directory
    for entry in fs::read_dir(base_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            // If the directory name consists only of digits, it's an event directory
            if path.file_name()
                .and_then(|name| name.to_str())
                .map_or(false, |name| name.chars().all(char::is_numeric)) 
            {
                create_webm_from_images(&path, &videos_dir)?;
            }
        }
    }

    Ok(())
}

fn main() -> io::Result<()> {
    // Set the base directory for your project
    let base_directory = Path::new("/path/to/top-level-directory");

    // Process all event directories
    process_event_directories(base_directory)?;

    Ok(())
}

