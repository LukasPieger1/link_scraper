use exif::Value;
use std::fmt::{Display, Formatter};
use std::io;
use thiserror::Error;

use crate::gen_scrape_froms;
use crate::helpers::find_urls;

pub fn scrape<R>(mut reader: R) -> Result<Vec<ImageLink>, ImageScrapingError>
where
    R: io::BufRead + io::Seek,
{
    let exif_res = exif::Reader::new().read_from_container(&mut reader);

    if let Err(exif::Error::NotFound(_)) = exif_res {
        return Ok(vec![]);
    }
    let exif = exif_res?;

    Ok(exif
        .fields()
        .map(|field| {
            if let Value::Ascii(_) = &field.value {
                find_urls(&field.display_value().to_string())
                    .iter()
                    .map(|link| ImageLink {
                        url: link.as_str().to_string(),
                        exif_field: field.tag.to_string(),
                    })
                    .collect()
            } else {
                vec![]
            }
        })
        .flatten()
        .collect())
}
gen_scrape_froms!(scrape(Read) -> Result<Vec<ImageLink>, ImageScrapingError>);

#[derive(Error, Debug)]
pub enum ImageScrapingError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    ExifError(#[from] exif::Error),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImageLink {
    pub url: String,
    pub exif_field: String,
}

impl Display for ImageLink {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_JPG: &[u8] = include_bytes!("../../test_files/images/exif_test.jpg");
    const TEST_JPG_NO_EXIF: &[u8] = include_bytes!("../../test_files/images/no_exif_test.jpg");

    #[test]
    fn scrape_exif_test() {
        let links = scrape_from_slice(TEST_JPG).unwrap();
        println!("{:?}", links);
        assert!(links.contains(&ImageLink {
            url: "https://test.exifdata.com".to_string(),
            exif_field: "ImageDescription".to_string()
        }));
        assert!(links.contains(&ImageLink {
            url: "https://test2.exifdata.com".to_string(),
            exif_field: "ImageDescription".to_string()
        }))
    }

    #[test]
    fn scrape_empty_exif_data_test() {
        let links = scrape_from_slice(TEST_JPG_NO_EXIF).unwrap();
        assert_eq!(links.len(), 0)
    }
}
