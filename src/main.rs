mod typst_compile;

use poise::serenity_prelude as serenity;
use tokio;
use std::fs::File;

const WORKING_FILENAME: &str = "temp.typ";


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
