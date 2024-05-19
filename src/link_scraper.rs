use linkify::LinkFinder;
use linkify::LinkKind::Url;

#[cfg(feature = "any_format")]
pub use crate::any_format_scraper::scrape;

/// Finds all URLs in a given string
/// # Example
/// ```
/// use crate::link_scraper::link_scraper::find_urls;
/// let urls = find_urls("dfjaoijewfj oijoiwfjoiwjoi j´21214https://www.google.com .äwä.f.f.wä ");
/// assert_eq!(urls.first().unwrap().as_str(), "https://www.google.com")
/// ```
pub fn find_urls(content: &str) -> Vec<linkify::Link> {
    LinkFinder::new().links(content)
        .filter(|link| link.kind().eq(&Url))
        .collect()
}

#[macro_export] macro_rules! gen_scrape_from_file {
    ($output_type:ty) => {
        use std::fs::File;
        use std::io::Read;
        use std::path::Path;
        use std::fs;

        /// Convenience function, that reads a file and uses [`scrape`] to scrape links from its content.
        pub fn scrape_from_file(path: &Path) -> $output_type {
            let bytes: Vec<u8> = {
                let mut f = File::open(path)?;
                let metadata = fs::metadata(path)?;
                let mut buffer = vec![0; metadata.len() as usize];
                f.read(&mut buffer).expect("buffer overflow");

                buffer
            };
            scrape(&bytes)
        }
    }
}