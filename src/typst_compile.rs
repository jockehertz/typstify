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

use typst_render;
use typst_pdf::PdfOptions;

// import template files
const PDF_TEMPLATE: &str = include_str!("./assets/pdf.typ");
const PNG_SVG_TEMPLATE: &str = include_str!("./assets/svg-png.typ");

// a barebones world struct
struct InMemoryWorld {
    library: LazyHash<Library>,
    fonts: LazyHash<FontBook>,
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

        Self {
            library: LazyHash::new(Library::builder().build()),
            fonts: LazyHash::new(FontBook::new()),
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


    fn font(&self, _index: usize) -> Option<typst::text::Font> {
        None
    }

    fn today(
        &self,
        _offset: Option<typst::foundations::Duration>,
    ) -> Option<Datetime> {
        None
    }
}


enum CompileOutput<'a> {
    Pdf(Vec<u8>),
    Png(Vec<u8>),
    Svg(&'a str)
}

enum TypstError {
    CompileError(String)
}


fn missing_file(id: FileId) -> FileError {
    FileError::NotFound(PathBuf::from(format!("{id:?}")))
}

fn compile_pdf(input: &str) -> Result<CompileOutput, String> {
    let world = InMemoryWorld::new(input.to_owned());
    let document = typst::compile(&world)
        .output
        .map_err(|errors| format!("{errors:#?}"))?;

    let pdf = typst_pdf::pdf(&document, &PdfOptions::default()).map_err(|_| format!("Error compiling to PDF"))?;

    return Ok(CompileOutput::Pdf(pdf));

}

fn compile_png(input: &str) -> CompileOutput {
todo!()
}

fn compile_svg(input: &str) -> CompileOutput {
todo!()
}

pub fn typst_compile(input: &str, output_type: OutputType) -> CompileOutput {
    let output = match output_type {
        OutputType::Pdf => compile_pdf(&format!(PDF_TEMPLATE, content = input)),
        OutputType::Png => compile_png(&format!(PNG_SVG_TEMPLATE, content = input)),
        OutputType::Svg => compile_svg(&format!(PNG_SVG_TEMPLATE, content = input)),
    };

    return output
}
