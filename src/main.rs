mod typst_compile;

use poise::serenity_prelude as serenity;
use tokio;
use std::fs::File;

const WORKING_FILENAME: &str = "temp.typ";

const SVG_PNG_MATH: &str = include_str!("./assets/svg_png_math_template.typ");
const SVG_PNG: &str = include_str!("./assets/svg_png_template.typ");
const PDF_MATH: &str = include_str!("./assets/pdf_math_template.typ");
const PDF: &str = include_str!("./assets/pdf_template.typ");

enum OutputType {
    Pdf,
    Svg,
    Png,
}

enum CompileError {
    FileOperationsFault,
}


fn main() -> () {
}
