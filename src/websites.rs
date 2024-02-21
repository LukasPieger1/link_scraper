
use crate::parser::{find_urls};

#[cfg(test)]
mod tests {
    use reqwest::{blocking, Error, Url};
    use crate::parser::ExtractionError;
    use super::*;

    impl From<reqwest::Error> for ExtractionError {
        fn from(value: Error) -> Self {
            ExtractionError::new(Some("during request to website"), Some(Box::new(value)))
        }
    }
    impl From<std::io::Error> for ExtractionError {
        fn from(value: std::io::Error) -> Self {
            ExtractionError::new(Some("During parsing "), Some(Box::new(value)))
        }
    }

    #[test]
    fn get_some_website() {
        let url = Url::parse("https://github.com/llvm/llvm-project/issues/55760").unwrap();
        // let url = Url::parse("https://www.google.com").unwrap();
        let result = get(url);
        result.unwrap();
    }

    #[test]
    fn find_urls_in_website() {
        use itertools::Itertools;

        // let url = Url::parse("https://github.com/llvm/llvm-project/issues/55760").unwrap();
        let url = Url::parse("https://www.google.com").unwrap();
        let site_content = get(url).unwrap();
        let urls = find_urls(&site_content).into_iter().unique().collect::<Vec<_>>();
        println!("Found URLs: {:?}", urls)
    }

    pub fn get(url: Url) -> Result<String, ExtractionError> {
        let res = blocking::get(url)?;
        let body = res.text()?;

        Ok(body)
    }
}