use log::error;
use crate::libs::goojara::{Genre, Goojara, GoojaraImpl};

mod libs;
mod pkg;

fn main() {
    env_logger::builder().filter_level(log::LevelFilter::Info).init();
    let mut goojara = Goojara::new();
    let movies = goojara.get_genre_movies(Genre::ACTION, 2).unwrap();
    let movie = movies.get(2).unwrap();
    match goojara.details((*movie).clone()) {
        Ok(details) => {
            let url =  goojara.woolty(details).unwrap().get_url().unwrap();
            println!("{}", url);
        }
        Err(e) => {
            error!("{}", e)
        }
    }
}
