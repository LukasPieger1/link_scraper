use std::hash::Hash;

use itertools::Itertools;
use linkify::LinkFinder;
use linkify::LinkKind::Url;

#[cfg(feature = "generic_file")]
pub use crate::generic_link_extractor::extract_links;

/// Finds all URLs in a given string
/// # Example
/// ```
/// use crate::untitled_rust_parser::link_extractor::find_urls;
/// let urls = find_urls("dfjaoijewfj oijoiwfjoiwjoi j´21214https://www.google.com .äwä.f.f.wä ");
/// assert_eq!(urls, vec!["https://www.google.com"])
/// ```
pub fn find_urls(content: &str) -> Vec<linkify::Link> {
    LinkFinder::new().links(content)
        .filter(|link| link.kind().eq(&Url))
        .collect()
}

pub(crate) fn unique_and_sort<T:Hash + Ord>(arr: &[T]) -> Vec<&T> {
    arr
        .into_iter()
        .sorted()
        .dedup()
        .collect::<Vec<_>>()
}

#[macro_export] macro_rules! gen_scrape_from_file {
    ($output_type:ty) => {
        use std::fs::File;
        use std::io::Read;
        use std::path::Path;
        use std::fs;
        
        pub fn scrape_from_file(path: &Path) -> $output_type {
            let bytes: Vec<u8> = {
                let mut f = File::open(path).expect("no file found");
                let metadata = fs::metadata(path).expect("unable to read metadata");
                let mut buffer = vec![0; metadata.len() as usize];
                f.read(&mut buffer).expect("buffer overflow");

                buffer
            };
            extract_links(&bytes)
        }
    }
}

// TODO DTD <- main next feature
// TODO for DTD check nom parser
// TODO metadaten für jpg/mp3/...
// TODO more doc
// TODO Readme
// TODO rename project "link_scraper"
// TODO refactor ooxml & odf to use xml-sturcture?
// TODO unify entry methods "scrape"/"scrape_file" - use macro