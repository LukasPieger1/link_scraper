use crate::helpers::find_urls;
use itertools::Itertools;
use std::error::Error;
use std::io::{Read, Seek};
use zip::read::ZipFile;
use zip::result::ZipError;

/// Scrapes all links from a given compressed file.
///
/// To avoid getting urls related to the ooxml-functionalities use [`scrape`] instead.
pub(crate) fn scrape_unfiltered<R>(reader: R) -> Result<Vec<String>, ZipError>
where
    R: Read + Seek,
{
    let mut archive = zip::ZipArchive::new(reader)?;

    let mut links: Vec<String> = vec![];
    for file_name in archive
        .file_names()
        .map(|name| name.to_owned())
        .collect_vec()
    {
        let mut file_content = String::new();
        if archive
            .by_name(&file_name)?
            .read_to_string(&mut file_content)
            .is_err()
        {
            continue;
        }

        find_urls(&file_content)
            .iter()
            .for_each(|link| links.push(link.as_str().to_string()))
    }

    Ok(links)
}

pub(crate) fn unified_unzip_scrape<R, T, E, F>(reader: R, extractor: F) -> Result<Vec<T>, E>
where
    R: Read + Seek,
    E: Error + From<std::io::Error> + From<ZipError>,
    F: Fn(ZipFile<'_>, &str, &mut Vec<T>) -> Result<(), E>,
{
    let mut archive = zip::ZipArchive::new(reader)?;
    let mut links: Vec<T> = vec![];
    for file_name in archive
        .file_names()
        .map(|name| name.to_owned())
        .collect_vec()
    {
        let content = archive.by_name(&file_name)?;
        if content.size() == 0 {
            continue;
        }
        extractor(content, &file_name, &mut links)?;
    }
    Ok(links)
}
