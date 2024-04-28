use crate::formats::xml::xlink::{get_xlink_attribute_value, XLinkFormatError};
use crate::formats::xml::XmlStartElement;

#[derive(Debug)]
enum XLinkType {
    Simple,
    Extended,
    Locator,
    Arc,
    Resource,
    Title
}

impl TryFrom<&str> for XLinkType {
    type Error = XLinkFormatError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "simple" => Ok(XLinkType::Simple),
            "extended" => Ok(XLinkType::Extended),
            "locator" => Ok(XLinkType::Locator),
            "arc" => Ok(XLinkType::Arc),
            "resource" => Ok(XLinkType::Resource),
            "title" => Ok(XLinkType::Title),
            _ => Err(XLinkFormatError::UnknownTypeError(value.to_string()))
        }
    }
}

pub enum XlinkElement<'a> {
    Simple(XlinkSimpleElement<'a>),
    Extended(XlinkExtendedElement<'a>),
    Locator(XlinkLocatorElement<'a>),
    Arc(XlinkArcElement<'a>),
    Resource(XlinkResourceElement<'a>),
    Title(XlinkTitleElement<'a>)
}

impl<'a> XlinkElement<'a> {
    pub fn try_from_xml_start_element(xml_start_element: XmlStartElement<'a>) -> Result<Option<Self>, XLinkFormatError> {
        let xlink_href = get_xlink_attribute_value("href", xml_start_element.attributes);
        let mut xlink_type_option = get_xlink_attribute_value("type", xml_start_element.attributes)
            .map(|type_value| XLinkType::try_from(type_value.as_str()))
            .transpose()?;
        if xlink_href.is_some() && xlink_type_option.is_none() {
            xlink_type_option = Some(XLinkType::Simple);
        }
        if xlink_type_option.is_none() { return Ok(None) }
        match xlink_type_option {
            None => Ok(None),
            Some(xlink_type) => {
                match xlink_type {
                    XLinkType::Simple => Ok(Some(XlinkElement::Simple(xml_start_element.try_into()?))),
                    XLinkType::Extended => Ok(Some(XlinkElement::Extended(xml_start_element.try_into()?))),
                    XLinkType::Locator => Ok(Some(XlinkElement::Locator(xml_start_element.try_into()?))),
                    XLinkType::Arc => Ok(Some(XlinkElement::Arc(xml_start_element.try_into()?))),
                    XLinkType::Resource => Ok(Some(XlinkElement::Resource(xml_start_element.try_into()?))),
                    XLinkType::Title => Ok(Some(XlinkElement::Title(xml_start_element.try_into()?)))
                }
            }
        }
    }
}

pub struct XlinkSimpleElement<'a> {
    pub href: Option<String>,
    pub role: Option<String>,
    pub arcrole: Option<String>,
    pub title: Option<String>,
    pub show: Option<String>,
    pub actuate: Option<String>,
    pub xml: XmlStartElement<'a>
}
impl<'a> TryFrom<XmlStartElement<'a>> for XlinkSimpleElement<'a> {
    type Error = XLinkFormatError;

    fn try_from(xml_start_element: XmlStartElement<'a>) -> Result<Self, Self::Error> {
        Ok(XlinkSimpleElement {
            href: get_xlink_attribute_value("href", xml_start_element.attributes),
            role: get_xlink_attribute_value("role", xml_start_element.attributes),
            arcrole: get_xlink_attribute_value("arcrole", xml_start_element.attributes),
            title: get_xlink_attribute_value("title", xml_start_element.attributes),
            show: get_xlink_attribute_value("show", xml_start_element.attributes),
            actuate: get_xlink_attribute_value("actuate", xml_start_element.attributes),
            xml: xml_start_element
        })
    }
}

pub struct XlinkExtendedElement<'a> {
    pub role: Option<String>,
    pub title: Option<String>,
    pub xml: XmlStartElement<'a>
}
impl<'a> TryFrom<XmlStartElement<'a>> for XlinkExtendedElement<'a> {
    type Error = XLinkFormatError;

    fn try_from(xml_start_element: XmlStartElement<'a>) -> Result<Self, Self::Error> {
        Ok(XlinkExtendedElement {
            role: get_xlink_attribute_value("role", xml_start_element.attributes),
            title: get_xlink_attribute_value("title", xml_start_element.attributes),
            xml: xml_start_element
        })
    }
}

pub struct XlinkLocatorElement<'a> {
    pub href: String,
    pub role: Option<String>,
    pub title: Option<String>,
    pub xml: XmlStartElement<'a>
}
impl<'a> TryFrom<XmlStartElement<'a>> for XlinkLocatorElement<'a> {
    type Error = XLinkFormatError;

    fn try_from(xml_start_element: XmlStartElement<'a>) -> Result<Self, Self::Error> {
        Ok(XlinkLocatorElement {
            href: get_xlink_attribute_value("href", xml_start_element.attributes)
                .ok_or(XLinkFormatError::MissingRequiredAttributeError("href".to_string()))?,
            role: get_xlink_attribute_value("role", xml_start_element.attributes),
            title: get_xlink_attribute_value("title", xml_start_element.attributes),
            xml: xml_start_element
        })
    }
}

pub struct XlinkArcElement<'a> {
    pub arcrole: Option<String>,
    pub title: Option<String>,
    pub show: Option<String>,
    pub actuate: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub xml: XmlStartElement<'a>
}
impl<'a> TryFrom<XmlStartElement<'a>> for XlinkArcElement<'a> {
    type Error = XLinkFormatError;

    fn try_from(xml_start_element: XmlStartElement<'a>) -> Result<Self, Self::Error> {
        Ok(XlinkArcElement {
            arcrole: get_xlink_attribute_value("arcrole", xml_start_element.attributes),
            title: get_xlink_attribute_value("title", xml_start_element.attributes),
            show: get_xlink_attribute_value("show", xml_start_element.attributes),
            actuate: get_xlink_attribute_value("actuate", xml_start_element.attributes),
            from: get_xlink_attribute_value("from", xml_start_element.attributes),
            to: get_xlink_attribute_value("to", xml_start_element.attributes),
            xml: xml_start_element
        })
    }
}

pub struct XlinkResourceElement<'a> {
    pub role: Option<String>,
    pub title: Option<String>,
    pub label: Option<String>,
    pub xml: XmlStartElement<'a>
}
impl<'a> TryFrom<XmlStartElement<'a>> for XlinkResourceElement<'a> {
    type Error = XLinkFormatError;

    fn try_from(xml_start_element: XmlStartElement<'a>) -> Result<Self, Self::Error> {
        Ok(XlinkResourceElement {
            role: get_xlink_attribute_value("role", xml_start_element.attributes),
            title: get_xlink_attribute_value("title", xml_start_element.attributes),
            label: get_xlink_attribute_value("label", xml_start_element.attributes),
            xml: xml_start_element
        })
    }
}

pub struct XlinkTitleElement<'a> {
    pub xml: XmlStartElement<'a>
}
impl<'a> TryFrom<XmlStartElement<'a>> for XlinkTitleElement<'a> {
    type Error = XLinkFormatError;

    fn try_from(_xml_start_element: XmlStartElement<'a>) -> Result<Self, Self::Error> {
        Ok(XlinkTitleElement {
            xml: _xml_start_element
        })
    }
}