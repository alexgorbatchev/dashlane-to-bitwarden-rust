use anyhow::{Context, Result, bail};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

pub const FILES: [&str; 5] = [
    "credentials.csv",
    "payments.csv",
    "personalInfo.csv",
    "ids.csv",
    "securenotes.csv",
];
pub type Row = BTreeMap<String, String>;

pub enum Input {
    Directory(PathBuf),
    Archive(Box<zip::ZipArchive<File>>),
}

impl Input {
    pub fn open(path: &Path) -> Result<Self> {
        let shown = cli_command_kit::display_path(path);
        let metadata = path
            .metadata()
            .with_context(|| format!("inspect source {shown}"))?;
        if metadata.is_dir() {
            return Ok(Self::Directory(path.into()));
        }
        if !metadata.is_file() {
            bail!("source {shown} is neither a directory nor a regular ZIP file");
        }
        if !path
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("zip"))
        {
            bail!("source {shown} must be a directory or .zip file");
        }
        let file = File::open(path).with_context(|| format!("open source {shown}"))?;
        let archive = zip::ZipArchive::new(file).context("open ZIP archive")?;
        Ok(Self::Archive(Box::new(archive)))
    }

    pub fn rows(&mut self, name: &str) -> Result<Vec<Row>> {
        match self {
            Self::Directory(path) => {
                let file = match File::open(path.join(name)) {
                    Ok(file) => file,
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                        return Ok(Vec::new());
                    }
                    Err(error) => return Err(error).with_context(|| format!("open {name}")),
                };
                parse(file, name)
            }
            Self::Archive(archive) => {
                let matches = archive
                    .file_names()
                    .enumerate()
                    .filter_map(|(index, entry)| {
                        (entry.rsplit('/').next() == Some(name)).then_some(index)
                    })
                    .collect::<Vec<_>>();
                match matches.as_slice() {
                    [] => Ok(Vec::new()),
                    [index] => parse(
                        archive
                            .by_index(*index)
                            .with_context(|| format!("open ZIP entry {name}"))?,
                        name,
                    ),
                    _ => bail!("ZIP contains multiple entries named {name}; source is ambiguous"),
                }
            }
        }
    }
}

fn parse(reader: impl Read, name: &str) -> Result<Vec<Row>> {
    let mut reader = csv::ReaderBuilder::new()
        .flexible(false)
        .from_reader(reader);
    let headers = reader
        .headers()
        .with_context(|| format!("read {name} headers"))?
        .clone();
    let mut seen = BTreeSet::new();
    for header in &headers {
        if !seen.insert(header) {
            bail!("duplicate column in {name}");
        }
    }
    reader
        .records()
        .map(|record| {
            let record = record.with_context(|| format!("read {name} record"))?;
            Ok(headers
                .iter()
                .zip(record.iter())
                .map(|(key, value)| (key.into(), value.into()))
                .collect())
        })
        .collect()
}
