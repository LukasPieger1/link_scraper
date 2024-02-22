use std::fmt::{Display, Formatter};
use thiserror::Error;

#[derive(Error, Debug)]
pub struct ExtractionError {
    msg: String,
    source: Option<Box<dyn std::error::Error>>, // TODO what am I doing wrong here? I think I can't make this an enum because my other mods want to expand what an ExtractionError can be.
}

impl ExtractionError {
    pub fn new(msg: Option<&str>, source: Option<Box<dyn std::error::Error>>) -> Self {
        let message: String = {
            if let Some(text) = msg { text.to_string() }
            else if let Some(err) = &source { format!("Raised by {}", err) }
            else { "No further information available.".to_string() }
        };
        Self {
            msg: message,
            source
        }
    }
}

impl Display for ExtractionError {
    // TODO I think I'm doing something wrong here ...do I actually need to implement Display with thiserror?
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(src) = &self.source {
            write!(f, "ExtractionError: {}; Source: {}", self.msg, src)
        } else {
            write!(f, "ExtractionError: {}", self.msg)
        }
    }
}