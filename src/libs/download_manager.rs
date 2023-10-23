extern crate curl;
extern crate indicatif;

use curl::easy::{Easy, WriteError};
use indicatif::{ProgressBar, ProgressStyle};
use std::fs::File;
use std::io::Write;
use log::{error, info};

pub fn download_manager(url: &str) -> Result<(), Box<dyn std::error::Error>> {

    // The path where you want to save the downloaded file
    let output_path = "movie.mp4";

    // Create a File to write the downloaded data
    let mut file = File::create(output_path)?;

    // Create a new Easy instance
    let mut easy = Easy::new();
    // Set the URL
    easy.url(url)?;
    easy.referer("https://www.wootly.ch/").unwrap();
    easy.useragent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/117.0.0.0 Safari/537.36").unwrap();
    // Initialize the progress bar
    let progress_bar = ProgressBar::new(0);
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template("[{wide_bar}] {bytes}/{total_bytes} ({eta})").unwrap()
    );
    // Set a callback to write the data to the file and update the progress bar
    let clone = progress_bar.clone();
    easy.write_function(move |data| {
        file.write_all(data).map(|_| {
            progress_bar.inc(data.len() as u64);
            data.len()
        }).map_err(|_| WriteError::Pause)
    })?;
    // Perform the download
    easy.perform()?;

    // Check for errors
    match easy.response_code() {
        Ok(code) if code == 200 => {
            info!("Download successful!");
        }
        Ok(code) => {
            error!("Server returned an error: {}", code);
        }
        Err(err) => {
            error!("Download failed: {:?}", err);
        }
    }
    clone.finish_and_clear();
    Ok(())
}