#![allow(unused)]

use std::env::{self, VarError};
use std::error::Error;
use std::fmt::Display;
use std::fs::{self, OpenOptions};
use std::io::{Write, Read, BufReader, Seek, SeekFrom, BufWriter};
use std::path::Path;
use crate::model::{self, *};

pub fn add_item(item: Item) -> Result<()> {
  let mut csv = Csv::new()?;
  csv.file.seek(SeekFrom::End(0))?;
  writeln!(csv.file, "{}", item.to_string())?;

  Ok(())
}

pub fn update_item(item: Item) -> Result<()> {
  let to_update_item = get_item_by_id(item.id())?;
  let offset = get_offset_by_id(item.id())?;
  Csv::new()?.splice(
      offset as u64,
      to_update_item.to_string().len() as u64,
      &item.to_string(),
  )
}

pub fn delete_item(id: u32) -> Result<()> {
  let to_delete_item = get_item_by_id(id)?;
  let offset = get_offset_by_id(id)?;
  Csv::new()?.splice(
      offset as u64,
      to_delete_item.to_string().len() as u64 + 1,
      "",
  )
}

/// It is not an efficient way to find the largest id for all records,
/// but it works while ensuring the simplicity of the tutorial.
/// In actual production projects,
/// please use other efficient ways to generate unique identifiers,
/// such as using `uuid`, etc.
pub fn get_max_id() -> Result<u32> {
  let max_id = get_all()?.iter().map(|item| item.id()).reduce(u32::max);

  if let Some(max_id) = max_id {
      Ok(max_id)
  } else {
      Ok(0)
  }
}

pub fn get_all() -> Result<Vec<Item>> {
  Ok(Csv::new()?
      .content()?
      .lines()
      .filter_map(|line| line.parse::<Item>().ok())
      .collect())
}

pub fn get_item_by_id(id: u32) -> Result<Item> {
  let content = Csv::new()?.content()?;
  let item_str = content
    .lines()
    .find(|line| {
      if let Ok(item) = line.parse::<Item>() {
        item.id() == id
      } else {
        false
      }
    });

  if let Some(item_str) = item_str {
    Ok(item_str.parse().unwrap())
  } else {
    Err(StorageError::ItemNoExist(id))
  }
}

pub const FILE_NAME: &str = ".todo.csv";
struct Csv {
  filename: String,
  file: fs::File,
}

impl Csv {
  fn new() -> Result<Self> {
    let filename = Csv::filename()?;
    let path = Path::new(&filename);

    // 文件或者文件夹是否存在，通过Path去判断
    if !path.exists() {
      let mut file = Csv::create(&path)?;
      // 写入表头
      file.write_all(b"id,name,completed,deleted,createdAt,completedAt,deletedAt\n")?;
      
      Ok(Self {
        filename: filename.to_string(),
        file,
      })
    } else {
      Ok(Self {
        filename: filename.to_string(),
        file: Csv::open(path)?,
      })
    }
  }

  fn filename() -> Result<String> {
    let home = env::var("HOME")?;
    let filename = home + "/" + FILE_NAME;

    Ok(filename)
  }

  fn create(path: &Path) -> Result<fs::File> {
    let csv = OpenOptions::new()
      .read(true)
      .write(true)
      .create(true)
      .open(path)?;

    Ok(csv)
  }

  fn open(path: &Path) -> Result<fs::File> {
    let csv = OpenOptions::new().read(true).write(true).open(path)?;

    Ok(csv)
  }

  fn content(&mut self) -> Result<String> {
    let mut contents = String::new();
    self.file.read_to_string(&mut contents);

    Ok(contents)
  }

  /// Specify any position in the file to delete the specified byte,
  /// and then insert any byte string
  fn splice(&mut self, offset: u64, delete_size: u64, write_content: &str) -> Result<()> {
    let file = &self.file;

    // Create a buffered reader form csv file
    let mut reader = BufReader::new(file);

    // Adjust the appropriate reading position
    reader.seek(SeekFrom::Start(offset + delete_size))?;

    // Save the rest of the file,
    // starting at the position after the last character that was deleted
    let mut rest_content = String::new();
    reader.read_to_string(&mut rest_content)?;

    // The final to be write content is spliced
    // by the `write_content` and the `rest_content`
    let write_content = write_content.to_owned() + &rest_content;

    // Create a buffered writer from csv file
    let mut writer = BufWriter::new(file);

    // Adjust the appropriate writing position
    writer.seek(SeekFrom::Start(offset))?;

    // Insert `write_content` and overwrite old file content
    writer.write_all(write_content.as_bytes())?;

    // Make sure there is no redundant old file content left
    file.set_len(offset + write_content.len() as u64)?;

    Ok(())
  }
}

type Result<T> = std::result::Result<T, StorageError>;

#[derive(Debug)]
pub enum StorageError {
    FileHandle(FileHandleError),
    ParseItem(ParseItemError),
    ItemNoExist(u32),
}

#[derive(Debug)]
pub enum FileHandleError {
    EnvVar(VarError),
    Io(std::io::Error),
}

impl Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use FileHandleError::*;
        use StorageError::*;
        match &self {
            FileHandle(source) => match source {
                EnvVar(e) => write!(f, "Rtd storage file handle env var error: {}", e),
                Io(e) => write!(f, "Rtd storage file handle io error: {}", e),
            },
            ParseItem(e) => write!(f, "Rtd storage parse todo error: {}", e),
            ItemNoExist(id) => write!(f, "Rtd storage todo no exist: {}", id),
        }
    }
}

impl Error for StorageError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        use FileHandleError::*;
        use StorageError::*;
        match self {
            FileHandle(source) => match source {
                EnvVar(source) => Some(source),
                Io(source) => Some(source),
            },
            ParseItem(e) => Some(e),
            ItemNoExist(_) => None,
        }
    }
}

impl From<VarError> for StorageError {
    fn from(value: VarError) -> Self {
        Self::FileHandle(FileHandleError::EnvVar(value))
    }
}

impl From<std::io::Error> for StorageError {
    fn from(value: std::io::Error) -> Self {
        Self::FileHandle(FileHandleError::Io(value))
    }
}

impl From<model::ParseItemError> for StorageError {
    fn from(value: model::ParseItemError) -> Self {
        Self::ParseItem(value)
    }
}

fn get_offset_by_id(id: u32) -> Result<usize> {
    let mut csv = Csv::new()?;
    let content = csv.content()?;
    let prev_lines = content.lines().take_while(|line| {
        if let Ok(item) = line.parse::<Item>() {
            item.id() != id
        } else {
            true
        }
    });
    let offset: usize = prev_lines.map(|line| line.len() + 1).sum();
    Ok(offset)
}