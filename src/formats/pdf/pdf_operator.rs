#![allow(non_snake_case)]

use std::char::{decode_utf16, DecodeUtf16Error};
use std::str::from_utf8;
use lopdf::{Object};
use strum_macros::EnumString;
use crate::formats::pdf::pdf_operator::PdfOperator::{TJ, Tj};
use crate::formats::pdf::PdfExtractionError;
use crate::formats::pdf::PdfExtractionError::UnexpectedPdfOperandError;

#[allow(clippy::upper_case_acronyms)]
#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, EnumString)]
pub enum PdfOperator {
    // General graphics state
    /// Save the current graphics state on the graphics state stack
    /// (see 8.4.2, "Graphics State Stack").
    #[strum(serialize = "w")] w,
    ///Set the line cap style in the graphics state
    /// (see 8.4.3.3, "Line Cap Style").
    #[strum(serialize = "J")] J,
    /// Set the line join style in the graphics state
    /// (see 8.4.3.4, "Line Join Style").
    #[strum(serialize = "j")] j,
    /// Set the miter limit in the graphics state (see 8.4.3.5, "Miter Limit").
    #[strum(serialize = "M")] M,
    /// Set the line dash pattern in the graphics state (see 8.4.3.6, "Line
    /// Dash Pattern").
    #[strum(serialize = "d")] d,
    /// (PDF 1.1) Set the colour rendering intent in the graphics state (see
    /// 8.6.5.8, "Rendering Intents").
    #[strum(serialize = "ri")] ri,
    /// Set the flatness tolerance in the graphics state (see 10.6.2,
    /// "Flatness Tolerance"). flatness is a number in the range 0 to 100; a
    /// value of 0 shall specify the output device’s default flatness
    /// tolerance.
    #[strum(serialize = "i")] i,
    /// (PDF 1.2) Set the specified parameters in the graphics state.
    /// dictName shall be the name of a graphics state parameter
    /// dictionary in the ExtGState subdictionary of the current resource
    /// dictionary (see the next sub-clause).
    #[strum(serialize = "gs")] gs,

    // Special graphics state
    /// Save the current graphics state on the graphics state stack
    /// (see 8.4.2, "Graphics State Stack").
    #[strum(serialize = "q")] q,
    /// Restore the graphics state by removing the most recently saved
    /// state from the stack and making it the current state (see 8.4.2,
    /// "Graphics State Stack").
    #[strum(serialize = "Q")] Q,
    /// Modify the current transformation matrix (CTM) by concatenating
    /// the specified matrix (see 8.3.2, "Coordinate Spaces"). Although the
    /// operands specify a matrix, they shall be written as six separate
    /// numbers, not as an array.
    #[strum(serialize = "cm")] cm,

    // Path construction
    /// Begin a new subpath by moving the current point to
    /// coordinates (x, y), omitting any connecting line segment. If
    /// the previous path construction operator in the current path
    /// was also m, the new m overrides it; no vestige of the
    /// previous m operation remains in the path.
    #[strum(serialize = "m")] m,
    /// Append a straight line segment from the current point to the
    /// point (x, y). The new current point shall be (x, y).
    #[strum(serialize = "l")] l,
    /// Append a cubic Bézier curve to the current path. The curve
    /// shall extend from the current point to the point (x3 , y3 ), using
    /// (x1 , y1 ) and (x2 , y2 ) as the Bézier control points (see 8.5.2.2,
    /// "Cubic Bézier Curves"). The new current point shall be
    /// (x3 , y3 ).
    #[strum(serialize = "c")] c,
    /// Append a cubic Bézier curve to the current path. The curve
    /// shall extend from the current point to the point (x3 , y3 ), using
    /// the current point and (x2 , y2 ) as the Bézier control points (see
    /// 8.5.2.2, "Cubic Bézier Curves"). The new current point shall
    /// be (x3 , y3 )
    #[strum(serialize = "v")] v,
    /// Append a cubic Bézier curve to the current path. The curve
    /// shall extend from the current point to the point (x3 , y3 ), using
    /// (x1 , y1 ) and (x3 , y3 ) as the Bézier control points (see 8.5.2.2,
    /// "Cubic Bézier Curves"). The new current point shall be
    /// (x3 , y3 )
    #[strum(serialize = "y")] y,
    /// Close the current subpath by appending a straight line
    /// segment from the current point to the starting point of the
    /// subpath. If the current subpath is already closed, h shall do
    /// nothing.
    ///
    /// This operator terminates the current subpath. Appending
    /// another segment to the current path shall begin a new
    /// subpath, even if the new segment begins at the endpoint
    /// reached by the h operation.
    #[strum(serialize = "h")] h,
    /// Append a rectangle to the current path as a complete
    /// subpath, with lower-left corner (x, y) and dimensions width
    /// and height in user space.
    #[strum(serialize = "re")] re,

    // Path painting
    /// Stroke the path.
    #[strum(serialize = "S")] S,
    /// Close and stroke the path. This operator shall have the same effect as the
    /// sequence h S.
    #[strum(serialize = "s")] s,
    /// Fill the path, using the nonzero winding number rule to determine the region
    /// to fill (see 8.5.3.3.2, "Nonzero Winding Number Rule"). Any subpaths that
    /// are open shall be implicitly closed before being filled.
    #[strum(serialize = "f")] f,
    /// Equivalent to f; included only for compatibility. Although PDF reader
    /// applications shall be able to accept this operator, PDF writer applications
    /// should use f instead.
    #[strum(serialize = "F")] F,
    /// Fill the path, using the even-odd rule to determine the region to fill (see
    /// 8.5.3.3.3, "Even-Odd Rule").
    #[strum(serialize = "f*")] fStar,
    /// Fill and then stroke the path, using the nonzero winding number rule to
    /// determine the region to fill. This operator shall produce the same result as
    /// constructing two identical path objects, painting the first with f and the
    /// second with S.
    ///
    /// NOTE:
    /// The filling and stroking portions of the operation consult
    /// different values of several graphics state parameters, such as
    /// the current colour. See also 11.7.4.4, "Special Path-Painting
    /// Considerations"
    #[strum(serialize = "B")] B,
    /// Fill and then stroke the path, using the even-odd rule to determine the region
    /// to fill. This operator shall produce the same result as B, except that the path
    /// is filled as if with f* instead of f. See also 11.7.4.4, "Special Path-Painting
    /// Considerations".
    #[strum(serialize = "B*")] BStar,
    /// Close, fill, and then stroke the path, using the nonzero winding number rule
    /// to determine the region to fill. This operator shall have the same effect as the
    /// sequence h B. See also 11.7.4.4, "Special Path-Painting Considerations".
    #[strum(serialize = "b")] b,
    /// Close, fill, and then stroke the path, using the even-odd rule to determine the
    /// region to fill. This operator shall have the same effect as the sequence h B*.
    /// See also 11.7.4.4, "Special Path-Painting Considerations".
    #[strum(serialize = "b*")] bStar,
    /// End the path object without filling or stroking it. This operator shall be a path-
    /// painting no-op, used primarily for the side effect of changing the current
    /// clipping path (see 8.5.4, "Clipping Path Operators").
    #[strum(serialize = "n")] n,

    // Clipping paths
    /// Modify the current clipping path by intersecting it with the current path, using
    /// the nonzero winding number rule to determine which regions lie inside the
    /// clipping path.
    #[strum(serialize = "W")] W,
    /// Modify the current clipping path by intersecting it with the current path, using
    /// the even-odd rule to determine which regions lie inside the clipping path.
    #[strum(serialize = "W*")] WStar,

    // Text object
    /// Begin a text object, initializing the text matrix, Tm , and the text line matrix,
    /// Tlm , to the identity matrix. Text objects shall not be nested; a second BT shall
    /// not appear before an ET.
    #[strum(serialize = "BT")] BT,
    /// End a text object, discarding the text matrix.
    #[strum(serialize = "ET")] ET,

    // Text state
    /// Set the character spacing, Tc , to charSpace, which shall be a number
    /// expressed in unscaled text space units. Character spacing shall be used
    /// by the Tj, TJ, and ' operators. Initial value: 0.
    #[strum(serialize = "Tc")] Tc,
    /// Set the word spacing, Tw, to wordSpace, which shall be a number
    /// expressed in unscaled text space units. Word spacing shall be used by
    /// the Tj, TJ, and ' operators. Initial value: 0.
    #[strum(serialize = "Tw")] Tw,
    /// Set the horizontal scaling, Th , to (scale ÷ 100). scale shall be a number
    /// specifying the percentage of the normal width. Initial value: 100 (normal
    /// width).
    #[strum(serialize = "Tz")] Tz,
    /// Set the text leading, Tl , to leading, which shall be a number expressed in
    /// unscaled text space units. Text leading shall be used only by the T*, ', and
    /// " operators. Initial value: 0.
    #[strum(serialize = "TL")] TL,
    /// Set the text font, Tf , to font and the text font size, Tfs , to size. font shall be
    /// the name of a font resource in the Font subdictionary of the current
    /// resource dictionary; size shall be a number representing a scale factor.
    /// There is no initial value for either font or size; they shall be specified
    /// explicitly by using Tf before any text is shown.
    #[strum(serialize = "Tf")] Tf,
    /// Set the text rendering mode, Tmode , to render, which shall be an integer.
    /// Initial value: 0.
    #[strum(serialize = "Tr")] Tr,
    /// Set the text rise, Trise , to rise, which shall be a number expressed in
    /// unscaled text space units. Initial value: 0.
    #[strum(serialize = "Ts")] Ts,

    // Text positioning
    /// Move to the start of the next line, offset from the start of the current line by
    /// (tx , ty ). tx and ty shall denote numbers expressed in unscaled text space
    /// units.
    ///
    /// __More info in the book__
    #[strum(serialize = "Td")] Td,

    /// Move to the start of the next line, offset from the start of the current line by
    /// (tx , ty ). As a side effect, this operator shall set the leading parameter in
    /// the text state.
    ///
    /// __More info in the book__
    #[strum(serialize = "TD")] TD,
    /// Set the text matrix, Tm , and the text line matrix, Tlm
    ///
    /// __More info in the book__
    ///
    /// The operands shall all be numbers, and the initial value for Tm and Tlm
    /// shall be the identity matrix, [ 1 0 0 1 0 0 ]. Although the operands
    /// specify a matrix, they shall be passed to Tm as six separate numbers, not
    /// as an array.
    /// The matrix specified by the operands shall not be concatenated onto the
    /// current text matrix, but shall replace it.
    #[strum(serialize = "Tm")] Tm,
    /// Move to the start of the next line. This operator has the same effect as the
    /// code
    ///
    /// 0 -Tl Td
    ///
    /// where Tl denotes the current leading parameter in the text state. The
    /// negative of Tl is used here because Tl is the text leading expressed as a
    /// positive number. Going to the next line entails decreasing the
    /// y coordinate.
    #[strum(serialize = "T*")] TStar,

    // Text showing
    /// Show a text string.
    #[strum(serialize = "Tj")] Tj,
    /// Show one or more text strings, allowing individual glyph positioning. Each
    /// element of array shall be either a string or a number. If the element is a
    /// string, this operator shall show the string. If it is a number, the operator
    /// shall adjust the text position by that amount; that is, it shall translate the
    /// text matrix, Tm . The number shall be expressed in thousandths of a unit
    /// of text space (see 9.4.4, "Text Space Details"). This amount shall be
    /// subtracted from the current horizontal or vertical coordinate, depending
    /// on the writing mode. In the default coordinate system, a positive
    /// adjustment has the effect of moving the next glyph painted either to the
    /// left or down by the given amount. Figure 46 shows an example of the
    /// effect of passing offsets to TJ.
    #[strum(serialize = "TJ")] TJ,
    /// Move to the next line and show a text string.
    ///
    /// __More info in the book__
    #[strum(serialize = "'")] SingleQuote,
    /// Move to the next line and show a text string, using a_w as the word spacing
    /// and a_c as the character spacing (setting the corresponding parameters in
    /// the text state). a_w and ac shall be numbers expressed in unscaled text
    /// space units.
    ///
    /// __More info in the book__
    #[strum(serialize = "\"")] DoubleQuote,

    // Type 3 fonts
    /// Set width information for the glyph and declare that the glyph
    /// description specifies both its shape and its colour.
    ///
    /// NOTE
    /// This operator name ends in the digit 0.
    ///
    /// wx denotes the horizontal displacement in the glyph coordinate
    /// system; it shall be consistent with the corresponding width in the
    /// font’s Widths array. wy shall be 0 (see 9.2.4, "Glyph Positioning
    /// and Metrics").
    ///
    /// This operator shall only be permitted in a content stream
    /// appearing in a Type 3 font’s CharProcs dictionary. It is typically
    /// used only if the glyph description executes operators to set the
    /// colour explicitly.
    #[strum(serialize = "d0")] d0,
    /// Set width and bounding box information for the glyph and declare
    /// that the glyph description specifies only shape, not colour.
    ///
    /// __More info in the book__
    #[strum(serialize = "d1")] d1,

    // Color
    /// (PDF 1.1) Set the current colour space to use for stroking operations. The
    /// operand name shall be a name object. If the colour space is one that can
    /// be specified by a name and no additional parameters (DeviceGray,
    /// DeviceRGB, DeviceCMYK, and certain cases of Pattern), the name may
    /// be specified directly. Otherwise, it shall be a name defined in the
    /// ColorSpace subdictionary of the current resource dictionary (see 7.8.3,
    /// "Resource Dictionaries"); the associated value shall be an array
    /// describing the colour space (see 8.6.3, "Colour Space Families").
    ///
    /// The names DeviceGray, DeviceRGB, DeviceCMYK, and Pattern
    /// always identify the corresponding colour spaces directly; they never refer
    /// to resources in the ColorSpace subdictionary.
    ///
    /// The CS operator shall also set the current stroking colour to its initial
    /// value, which depends on the colour space:
    ///
    /// In a DeviceGray, DeviceRGB, CalGray, or CalRGB colour space, the
    /// initial colour shall have all components equal to 0.0.
    ///
    /// In a DeviceCMYK colour space, the initial colour shall be
    /// [ 0.0 0.0 0.0 1.0 ].
    ///
    /// In a Lab or ICCBased colour space, the initial colour shall have all
    /// components equal to 0.0 unless that falls outside the intervals specified
    /// by the space’s Range entry, in which case the nearest valid value shall be
    /// substituted.
    ///
    /// In an Indexed colour space, the initial colour value shall be 0.
    /// In a Separation or DeviceN colour space, the initial tint value shall be 1.0
    /// for all colorants.
    ///
    /// In a Pattern colour space, the initial colour shall be a pattern object that
    /// causes nothing to be painted.
    #[strum(serialize = "CS")] CS,
    /// (PDF 1.1) Same as CS but used for nonstroking operations.
    #[strum(serialize = "cs")] cs,
    /// (PDF 1.1) Set the colour to use for stroking operations in a device, CIE-
    /// based (other than ICCBased), or Indexed colour space. The number of
    /// operands required and their interpretation depends on the current
    /// stroking colour space:
    ///
    /// For DeviceGray, CalGray, and Indexed colour spaces, one operand
    /// shall be required (n = 1).
    ///
    /// For DeviceRGB, CalRGB, and Lab colour spaces, three operands shall
    /// be required (n = 3).
    ///
    /// For DeviceCMYK, four operands shall be required (n = 4).
    #[strum(serialize = "SC")] SC,
    /// (PDF 1.2) Same as SC but also supports Pattern, Separation, DeviceN
    /// and ICCBased colour spaces.
    ///
    /// If the current stroking colour space is a Separation, DeviceN, or
    /// ICCBased colour space, the operands c1… cn shall be numbers. The
    /// number of operands and their interpretation depends on the colour space.
    ///
    /// If the current stroking colour space is a Pattern colour space, name shall
    /// be the name of an entry in the Pattern subdictionary of the current
    /// resource dictionary (see 7.8.3, "Resource Dictionaries"). For an
    /// uncoloured tiling pattern (PatternType = 1 and PaintType = 2), c1… cn
    /// shall be component values specifying a colour in the pattern’s underlying
    /// colour space. For other types of patterns, these operands shall not be
    /// specified.
    #[strum(serialize = "SCN")] SCN,
    /// (PDF 1.2) Same as SC but used for nonstroking operations.
    #[strum(serialize = "sc")] sc,
    /// (PDF 1.1) Same as SCN but used for nonstroking operations.
    #[strum(serialize = "scn")] scn,
    /// Set the stroking colour space to DeviceGray (or the DefaultGray colour
    /// space; see 8.6.5.6, "Default Colour Spaces") and set the gray level to use
    /// for stroking operations. gray shall be a number between 0.0 (black) and
    /// 1.0 (white).
    #[strum(serialize = "G")] G,
    /// Same as G but used for nonstroking operations.
    #[strum(serialize = "g")] g,
    /// Set the stroking colour space to DeviceRGB (or the DefaultRGB colour
    /// space; see 8.6.5.6, "Default Colour Spaces") and set the colour to use for
    /// stroking operations. Each operand shall be a number between 0.0
    /// (minimum intensity) and 1.0 (maximum intensity).
    #[strum(serialize = "RG")] RG,
    /// Same as RG but used for nonstroking operations.
    #[strum(serialize = "rg")] rg,
    /// Set the stroking colour space to DeviceCMYK (or the DefaultCMYK
    /// colour space; see 8.6.5.6, "Default Colour Spaces") and set the colour to
    /// use for stroking operations. Each operand shall be a number between 0.0
    /// (zero concentration) and 1.0 (maximum concentration). The behaviour of
    /// this operator is affected by the overprint mode (see 8.6.7, "Overprint
    /// Control").
    #[strum(serialize = "K")] K,
    /// Same as K but used for nonstroking operations.
    #[strum(serialize = "k")] k,

    // Shading patterns
    /// (PDF 1.3) Paint the shape and colour shading described by a shading
    /// dictionary, subject to the current clipping path. The current colour in the
    /// graphics state is neither used nor altered. The effect is different from that of
    /// painting a path using a shading pattern as the current colour.
    ///
    /// name is the name of a shading dictionary resource in the Shading
    /// subdictionary of the current resource dictionary (see 7.8.3, "Resource
    /// Dictionaries"). All coordinates in the shading dictionary are interpreted
    /// relative to the current user space. (By contrast, when a shading dictionary is
    /// used in a type 2 pattern, the coordinates are expressed in pattern space.) All
    /// colours are interpreted in the colour space identified by the shading
    /// dictionary’s ColorSpace entry (see Table 78). The Background entry, if
    /// present, is ignored.
    ///
    /// This operator should be applied only to bounded or geometrically defined
    /// shadings. If applied to an unbounded shading, it paints the shading’s
    /// gradient fill across the entire clipping region, which may be time-consuming.
    #[strum(serialize = "sh")]  sh,

    // Inline images
    /// Begin an inline image object.
    #[strum(serialize = "BI")] BI,
    /// Begin the image data for an inline image object.
    #[strum(serialize = "ID")] ID,
    /// End an inline image object.
    #[strum(serialize = "EI")] EI,

    // XObjects
    /// Paint the specified XObject. The operand name shall appear as a key in
    /// the XObject subdictionary of the current resource dictionary (see 7.8.3,
    /// "Resource Dictionaries"). The associated value shall be a stream whose
    /// Type entry, if present, is XObject. The effect of Do depends on the value
    /// of the XObject’s Subtype entry, which may be Image (see 8.9.5, "Image
    /// Dictionaries"), Form (see 8.10, "Form XObjects"), or PS (see 8.8.2,
    /// "PostScript XObjects").
    #[strum(serialize = "Do")] Do,

    // Marked content
    /// Designate a marked-content point. tag shall be a name object indicating
    /// the role or significance of the point.
    #[strum(serialize = "MP")] MP,
    /// Designate a marked-content point with an associated property list. tag
    /// shall be a name object indicating the role or significance of the point.
    /// properties shall be either an inline dictionary containing the property list or
    /// a name object associated with it in the Properties subdictionary of the
    /// current resource dictionary (see 14.6.2, “Property Lists”).
    #[strum(serialize = "DP")] DP,
    /// Begin a marked-content sequence terminated by a balancing EMC
    /// operator. tag shall be a name object indicating the role or significance of
    /// the sequence.
    #[strum(serialize = "BMZ")] BMC,
    /// Begin a marked-content sequence with an associated property list,
    /// terminated by a balancing EMC operator. tag shall be a name object
    /// indicating the role or significance of the sequence. properties shall be
    /// either an inline dictionary containing the property list or a name object
    /// associated with it in the Properties subdictionary of the current resource
    /// dictionary (see 14.6.2, “Property Lists”).
    #[strum(serialize = "BDC")] BDC,
    /// End a marked-content sequence begun by a BMC or BDC operator.
    #[strum(serialize = "EMC")] EMC,

    // Compatibility
    /// (PDF 1.1) Begin a compatibility section. Unrecognized operators (along with
    /// their operands) shall be ignored without error until the balancing EX operator
    /// is encountered.
    #[strum(serialize = "BX")] BX,
    /// (PDF 1.1) End a compatibility section begun by a balancing BX operator.
    /// Ignore any unrecognized operands and operators from previous matching
    /// BX onward.
    #[strum(serialize = "EX")] EX
}

pub enum TJArrayEntries { // TODO This is just a subset of lopdf::Object Enum-variants...is there a better way to solve this? ...this is gonna get annoying with more and more arrays of possible types
    Integer(i64),
    String(String)
}

fn from_utf16(slice: &[u8]) -> Result<String, PdfExtractionError> {
    let iter = (0..slice.len()/2)
        .map(|i| u16::from_be_bytes([slice[2*i], slice[2*i+1]]));
    Ok(decode_utf16(iter)
        .map(|res| res)
        .collect::<Result<String, DecodeUtf16Error>>()?)

}

pub fn get_operands_Tj(operands: &Vec<Object>) -> Result<String, PdfExtractionError> { // TODO is there a better solution than to_owned here?
    Ok(from_utf8(operands.first().ok_or(UnexpectedPdfOperandError(Tj, operands.to_owned()))?.as_str()?).unwrap_or(&*format!("ERROR {:?}", operands)).parse()?)
}
pub fn get_operands_TJ(operands: &Vec<Object>) -> Result<Vec<TJArrayEntries>, PdfExtractionError> { // TODO is there a better solution than to_owned here?
    let ret = operands
        .first().ok_or(UnexpectedPdfOperandError(Tj, operands.to_owned()))?
        .as_array()?.iter()
        .map(|value| match value {
            Object::Integer(i) => Ok(TJArrayEntries::Integer(*i)),
            Object::String(s, _format) => Ok(TJArrayEntries::String(from_utf8(s).unwrap_or(&*format!("ERROR {:?}", operands)).parse()?)),
            _ => Err(UnexpectedPdfOperandError(TJ, operands.to_owned()))
        }).collect();
    ret
}

impl PdfOperator {
    fn allowed_operands(&self) -> Vec<String> { // TODO replace this with a more sensible implementation
        match self {
            //General graphics state
            PdfOperator::w => {unimplemented!()} // lineWidth
            PdfOperator::J => {unimplemented!()} // lineCap
            PdfOperator::j => {unimplemented!()} // lineJoin
            PdfOperator::M => {unimplemented!()} // miterLimit
            PdfOperator::d => {unimplemented!()} // dashArray dashPhase
            PdfOperator::ri => {unimplemented!()} // intent
            PdfOperator::i => {unimplemented!()} // flatness
            PdfOperator::gs => {unimplemented!()} // dictName

            //Special graphics state
            PdfOperator::q => {unimplemented!()} // -
            PdfOperator::Q => {unimplemented!()} // -
            PdfOperator::cm => {unimplemented!()} // a b c d e f

            // Path construction
            PdfOperator::m => {unimplemented!()} // x y
            PdfOperator::l => {unimplemented!()} // x y
            PdfOperator::c => {unimplemented!()} // x1 y1 x2 y2 x3 y3
            PdfOperator::v => {unimplemented!()} // x2 y2 x3 y3
            PdfOperator::y => {unimplemented!()} // x1 y1 x3 y3
            PdfOperator::h => {unimplemented!()} // -
            PdfOperator::re => {unimplemented!()} // x y width height

            // Path painting
            PdfOperator::S => {vec![]} // -
            PdfOperator::s => {vec![]} // -
            PdfOperator::f => {vec![]} // -
            PdfOperator::F => {vec![]} // -
            PdfOperator::fStar => {vec![]} // -
            PdfOperator::B => {vec![]} // -
            PdfOperator::BStar => {vec![]} // -
            PdfOperator::b => {vec![]} // -
            PdfOperator::bStar => {vec![]} // -
            PdfOperator::n => {vec![]} // -

            // Clipping paths
            PdfOperator::W => {unimplemented!()} // -
            PdfOperator::WStar => {unimplemented!()} // -

            // Text objects
            PdfOperator::BT => {vec![]} // -
            PdfOperator::ET => {vec![]} // -

            // Text positioning
            PdfOperator::Tc => {unimplemented!()} // charSpace
            PdfOperator::Tw => {unimplemented!()} // wordSpace
            PdfOperator::Tz => {unimplemented!()} // scale
            PdfOperator::TL => {unimplemented!()} // leading
            PdfOperator::Tf => {unimplemented!()} // font size
            PdfOperator::Tr => {unimplemented!()} // render
            PdfOperator::Ts => {unimplemented!()} // rise

            // Text positioning
            PdfOperator::Td => {unimplemented!()} // t_x t_y
            PdfOperator::TD => {unimplemented!()} // t_x t_y
            PdfOperator::Tm => {unimplemented!()} // a b c d e f
            PdfOperator::TStar => {unimplemented!()} // -

            // Text showing
            PdfOperator::Tj => {todo!()} // string
            PdfOperator::TJ => {todo!()} // string
            PdfOperator::SingleQuote => {todo!()} // a_w a_c string
            PdfOperator::DoubleQuote => {todo!()} // array

            // Type 3 fonts
            PdfOperator::d0 => {unimplemented!()} // w_x w_y
            PdfOperator::d1 => {unimplemented!()} // w_x w_y ll_x ll_y ur_x ur_y

            // Color
            PdfOperator::CS => {unimplemented!()} // name
            PdfOperator::cs => {unimplemented!()} // name
            PdfOperator::SC => {unimplemented!()} // c_1, ..., c_n
            PdfOperator::SCN => {unimplemented!()} // c_1, ..., c_n c_1, ..., c_n name
            PdfOperator::sc => {unimplemented!()} // c_1, ..., c_n
            PdfOperator::scn => {unimplemented!()} // c_1, ..., c_n c_1, ..., c_n name
            PdfOperator::G => {unimplemented!()} // gray
            PdfOperator::g => {unimplemented!()} // gray
            PdfOperator::RG => {unimplemented!()} // r g b
            PdfOperator::rg => {unimplemented!()} // r g b
            PdfOperator::K => {unimplemented!()} // c m y k
            PdfOperator::k => {unimplemented!()} // c m y k

            // Shading patterns
            PdfOperator::sh => {unimplemented!()} // name

            // Inline images
            PdfOperator::BI => {vec![]} // -
            PdfOperator::ID => {vec![]} // -
            PdfOperator::EI => {vec![]} // -

            // XObjects
            PdfOperator::Do => {todo!()} // name

            // Marked content
            PdfOperator::MP => {todo!()} // tag
            PdfOperator::DP => {todo!()} // tag properties
            PdfOperator::BMC => {todo!()} // tag
            PdfOperator::BDC => {todo!()} // tag properties
            PdfOperator::EMC => {vec![]} // -

            // Compatibility
            PdfOperator::BX => {vec![]} // -
            PdfOperator::EX => {vec![]} // -
        }
    }
}

#[allow(non_snake_case)]
#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use crate::formats::pdf::pdf_operator::PdfOperator;

    #[test]
    fn check_text_object_deserialization() {
        let op_BT = PdfOperator::from_str("BT").unwrap();
        assert_eq!(op_BT, PdfOperator::BT);
        let op_ET = PdfOperator::from_str("ET").unwrap();
        assert_eq!(op_ET, PdfOperator::ET);
    }
}