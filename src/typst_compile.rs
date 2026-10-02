use crate::OutputType;
use std::{collections::HashMap, path::PathBuf};

use typst::{
    diag::{FileError, FileResult},
    foundations::{Bytes, Datetime},
    syntax::{FileId, RootedPath, Source, VirtualPath, VirtualRoot},
    text::{Font, FontBook},
    utils::LazyHash,
    Library, LibraryExt, World,
};

use typst_assets;
use typst_layout::PagedDocument;

use tiny_skia::Pixmap;
use typst_pdf::PdfOptions;
use typst_render;
use typst_svg;

// import template files
const PDF_TEMPLATE: &str = include_str!("./assets/pdf.typ");
const PNG_SVG_TEMPLATE: &str = include_str!("./assets/svg-png.typ");

// this is a multiplier (*72)
const PNG_PIXELS_PER_PT: f64 = 2.0;

// a barebones world struct
struct InMemoryWorld {
    library: LazyHash<Library>,
    fonts: LazyHash<FontBook>,
    font_data: Vec<Font>,
    main: FileId,
    sources: HashMap<FileId, Source>,
    files: HashMap<FileId, Bytes>,
}

impl InMemoryWorld {
    fn new(source_text: String) -> Self {
        let main = RootedPath::new(
            VirtualRoot::Project,
            VirtualPath::new("main.typ").expect("Could not create virtual path"),
        )
        .intern();

        let mut sources = HashMap::new();
        sources.insert(main, Source::new(main, source_text));

        let font_data: Vec<Font> = typst_assets::fonts()
            .flat_map(|data| Font::iter(Bytes::new(data)))
            .collect();

        let font_book = FontBook::from_fonts(&font_data);

        Self {
            library: LazyHash::new(Library::builder().build()),
            fonts: LazyHash::new(font_book),
            font_data,
            main,
            sources,
            files: HashMap::new(),
        }
    }
}

impl World for InMemoryWorld {
    fn library(&self) -> &LazyHash<Library> {
        &self.library
    }

    fn book(&self) -> &LazyHash<FontBook> {
        &self.fonts
    }

    fn main(&self) -> FileId {
        self.main
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        self.sources
            .get(&id)
            .cloned()
            .ok_or_else(|| missing_file(id))
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        self.files.get(&id).cloned().ok_or_else(|| missing_file(id))
    }

    fn font(&self, index: usize) -> Option<typst::text::Font> {
        self.font_data.get(index).cloned()
    }

    fn today(&self, _offset: Option<typst::foundations::Duration>) -> Option<Datetime> {
        None
    }
}

// collect the output in different types
pub enum CompileOutput {
    Pdf(Vec<u8>),
    Png(Vec<Vec<u8>>),
    Svg(Vec<String>),
}

// wrap the error from the typst compilation
pub enum TypstError {
    CompileError(String),
    RenderError,
}

// helper for "missing" virtual files
fn missing_file(id: FileId) -> FileError {
    FileError::NotFound(PathBuf::from(format!("{id:?}")))
}

fn compile_document(input: &str) -> Result<PagedDocument, TypstError> {
    let world = InMemoryWorld::new(input.to_owned());
    match typst::compile(&world).output {
        Ok(data) => Ok(data),
        Err(errors) => Err(TypstError::CompileError(format!("{:#?}", errors))),
    }
}

// compiles an input to a pdf
fn compile_pdf(input: &str) -> Result<CompileOutput, TypstError> {
    let document = compile_document(input)?;

    match typst_pdf::pdf(&document, &PdfOptions::default()) {
        Ok(data) => Ok(CompileOutput::Pdf(data)),
        Err(_) => Err(TypstError::RenderError),
    }

}

fn compile_png(input: &str) -> Result<CompileOutput, TypstError> {
    let document = compile_document(input)?;

    let mut pngs: Vec<Vec<u8>> = vec![];

    for page in document.pages() {
        let render_options = typst_render::RenderOptions {
            pixel_per_pt: typst::utils::Scalar::new(PNG_PIXELS_PER_PT),
            ..Default::default()
        };
        let this_pixmap: Pixmap =
            typst_render::render(page, &render_options);
        let this_png: Vec<u8> = match this_pixmap.encode_png() {
            Ok(data) => data,
            Err(_) => return Err(TypstError::RenderError),
        };
        pngs.push(this_png);
    }

    Ok(CompileOutput::Png(pngs))
}

fn compile_svg(input: &str) -> Result<CompileOutput, TypstError> {
    let document = compile_document(input)?;

    let mut svgs: Vec<String> = vec![];
    for page in document.pages() {
        svgs.push(typst_svg::svg(page, &typst_svg::SvgOptions::default()));
    }

    Ok(CompileOutput::Svg(svgs))
}

pub fn typst_compile(input: &str, output_type: OutputType) -> Result<CompileOutput, TypstError> {
    if input.contains("#import") {
        return Err(TypstError::CompileError(String::from(
            "Packages are not supported in this version, #import statements are disallowed.",
        )));
    }
    let output = match output_type {
        OutputType::Pdf => compile_pdf(&PDF_TEMPLATE.replace("{content}", input)),
        OutputType::Png => compile_png(&PNG_SVG_TEMPLATE.replace("{content}", input)),
        OutputType::Svg => compile_svg(&PNG_SVG_TEMPLATE.replace("{content}", input)),
    };

    return output;
}
