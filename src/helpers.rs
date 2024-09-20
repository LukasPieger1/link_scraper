use linkify::LinkFinder;
use linkify::LinkKind::Url;

#[cfg(feature = "any_format")]
pub use crate::any_format_scraper::scrape;

/// Finds all URLs in a given string
/// # Example
/// ```
/// use crate::link_scraper::helpers::find_urls;
/// let urls = find_urls("dfjaoijewfj oijoiwfjoiwjoi j´21214https://www.google.com .äwä.f.f.wä ");
/// assert_eq!(urls.first().unwrap().as_str(), "https://www.google.com")
/// ```
pub fn find_urls(content: &str) -> Vec<linkify::Link> {
    LinkFinder::new()
        .links(content)
        .filter(|link| link.kind().eq(&Url))
        .collect()
}

#[macro_export]
macro_rules! gen_scrape_from_slice {
    ($function_name:ident(Read) -> $output_type:ty) => {
        /// Convenience function, that uses [`scrape`] to scrape links from a buffer.
        pub fn scrape_from_slice<T>(buffer: T) -> $output_type
        where
            T: AsRef<[u8]>,
        {
            $function_name(std::io::Cursor::new(buffer.as_ref()))
        }
    };
}

#[macro_export]
macro_rules! gen_scrape_from_file {
    ($function_name:ident(AsRef<[u8]>) -> $output_type:ty) => {
        /// Convenience function, that reads a file and uses [`scrape`] to scrape links from its content.
        pub fn scrape_from_file<P>(path: P) -> $output_type
        where
            P: AsRef<std::path::Path>,
        {
            let bytes: Vec<u8> = {
                let mut f = std::fs::File::open(&path)?;
                let metadata = std::fs::metadata(path)?;
                let mut buffer = Vec::with_capacity(metadata.len() as usize);
                f.read_to_end(&mut buffer)?;
                buffer
            };
            $function_name(bytes)
        }
    };

    ($function_name:ident(Read) -> $output_type:ty) => {
        /// Convenience function, that reads a file and uses [`scrape`] to scrape links from its content.
        pub fn scrape_from_file<P>(path: P) -> $output_type
        where
            P: AsRef<std::path::Path>,
        {
            $function_name(std::io::BufReader::new(std::fs::File::open(path)?))
        }
    };
}
