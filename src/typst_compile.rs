use crate::OutputType;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use typst::{
    foundations::{Bytes, Datetime, Smart},
    model::Document,
    utils::LazyHash,
    Library,
    LibraryExt,
    World,
    diag::{FileError, FileResult},
    syntax::{FileId, Source, VirtualPath, VirtualRoot, RootedPath},
    text::{Font, FontBook},
};

use typst_layout::PagedDocument;
use typst_assets;

use typst_render;
use typst_pdf::PdfOptions;
use typst_svg;

// import template files
const PDF_TEMPLATE: &str = include_str!("./assets/pdf.typ");
const PNG_SVG_TEMPLATE: &str = include_str!("./assets/svg-png.typ");

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
        self.files
            .get(&id)
            .cloned()
            .ok_or_else(|| missing_file(id))
    }


    fn font(&self, index: usize) -> Option<typst::text::Font> {
        self.font_data.get(index).cloned()
    }

    fn today(
        &self,
        _offset: Option<typst::foundations::Duration>,
    ) -> Option<Datetime> {
        None
    }
}

// collect the output in different types
enum CompileOutput {
    Pdf(Vec<u8>),
    Png(Vec<u8>),
    Svgs(Vec<String>)
}

// wrap the error from the typst compilation
enum TypstError {
    CompileError(String)
}

// helper for "missing" virtual files
fn missing_file(id: FileId) -> FileError {
    FileError::NotFound(PathBuf::from(format!("{id:?}")))
}

// compiles an input to a pdf
fn compile_pdf(input: &str) -> Result<CompileOutput, TypstError> {
    let world = InMemoryWorld::new(input.to_owned());
    let document = match typst::compile(&world).output {
        Ok(data) => data,
        Err(errors) => return Err(TypstError::CompileError(format!("{:#?}", errors)))
    };

    let pdf = match typst_pdf::pdf(&document, &PdfOptions::default()) {
        Ok(data) => data,
        Err(_) => return Err(TypstError::CompileError(String::from("Could not compile to PDF"))),
    };

    return Ok(CompileOutput::Pdf(pdf));

}

fn compile_png(input: &str) -> Result<CompileOutput, TypstError> {
    let world = InMemoryWorld::new(input.to_owned());
    let document: PagedDocument = match typst::compile(&world).output {
        Ok(data) => data,
        Err(errors) => return Err(TypstError::CompileError(format!("{:#?}", errors)))
    };

    let mut svgs: Vec<String> = vec![];
    for page in document.pages() {
        svgs.push(typst_svg::svg(page, &typst_svg::SvgOptions::default()));
    };

    Ok(CompileOutput::Svgs(svgs))
}

fn compile_svg(input: &str) -> Result<CompileOutput, TypstError> {
todo!()
}

pub fn typst_compile(input: &str, output_type: OutputType) -> Result<CompileOutput, TypstError> {
    let output = match output_type {
        OutputType::Pdf => compile_pdf(&format!(PDF_TEMPLATE, content = input)),
        OutputType::Png => compile_png(&format!(PNG_SVG_TEMPLATE, content = input)),
        OutputType::Svg => compile_svg(&format!(PNG_SVG_TEMPLATE, content = input)),
    };

    return output
}
