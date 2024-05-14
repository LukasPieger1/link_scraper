use std::io::{Cursor, Read};
use itertools::Itertools;
use zip::result::ZipError;
use crate::link_scraper::find_urls;

/// Scrapes all links from a given compressed file.
///
/// To avoid getting urls related to the ooxml-functionalities use [`scrape`] instead.
pub(crate) fn scrape_unfiltered(bytes: &[u8]) -> Result<Vec<String>, ZipError> {
    let cur = Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(cur)?;

    let mut links: Vec<String> = vec![];
    for file_name in archive.file_names().map(|name| name.to_owned()).collect_vec() {
        let mut file_content = String::new();
        if archive.by_name(&file_name)?.read_to_string(&mut file_content).is_err() {continue}

        find_urls(&file_content).iter().for_each(|link| links.push(link.as_str().to_string()))
    }

    Ok(links)
}