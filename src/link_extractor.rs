use std::fmt::{Display, Formatter};
use std::hash::Hash;
use itertools::{Itertools};
use lazy_static::lazy_static;
use linkify::LinkFinder;
use linkify::LinkKind::{Url};
#[cfg(feature = "generic_file")]
pub use crate::generic_link_extractor::extract_links;

lazy_static! {
    static ref FINDER: LinkFinder = LinkFinder::new();
}



impl Display for Link {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.href)
    }
}
/// Finds all URLs in a given string
/// # Example
/// ```
/// use crate::untitled_rust_parser::link_extractor::find_links;
/// let urls = find_links("dfjaoijewfj oijoiwfjoiwjoi j´21214https://www.google.com .äwä.f.f.wä ");
/// assert_eq!(urls, vec!["https://www.google.com"])
/// ```
pub fn find_links(content: &str) -> Vec<&str> {
    FINDER.links(content)
        .filter(|link| link.kind().eq(&Url))
        .map(|link| link.as_str()).collect()
}

pub(crate) fn unique_and_sort<T:Hash + Ord>(arr: &[T]) -> Vec<&T> {
    arr
        .into_iter()
        .sorted()
        .dedup()
        .collect::<Vec<_>>()
}

// TODO check Xlink xml-schema standard -> wplink /DTD <- main next feature
// TODO for DTD check nom parser
// TODO Optional filename for format-recognition
// TODO SVG-format
// TODO xml-schema mit location?
// TODO metadaten für jpg/mp3/...