use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use super::{
    ArchiveExt,
    BasicUpdater,
    BasicEntry
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Archive {
    path: PathBuf
}

impl ArchiveExt for Archive {
    type Error = std::io::Error;
    type Entry = BasicEntry;
    type Updater = BasicUpdater;

    #[inline]
    fn open(file: impl AsRef<Path>) -> Result<Self, Self::Error> where Self: Sized {
        Ok(Self {
            path: file.as_ref().to_path_buf()
        })
    }

    // TODO: cache

    fn entries(&self) -> Result<Vec<Self::Entry>, Self::Error> {
        let output = Command::new("tar")
            .arg("-tvf")
            .arg(&self.path)
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output()?;

        let output = String::from_utf8_lossy(&output.stdout);

        Ok(output.trim()
            .split('\n')
            .take_while(|line| !line.starts_with("---------"))
            .map(|line| line.split(' ').filter_map(|word| {
                let word = word.trim();

                if word.is_empty() {
                    None
                } else {
                    Some(word)
                }
            }))
            .flat_map(|mut words| {
                let flags = words.next();
                let size = words.nth(1).map(|size| size.parse());
                let path = words.last().map(PathBuf::from);

                if let (Some(flags), Some(path), Some(Ok(size))) = (flags, path, size) {
                    // Skip symlinks
                    // FIXME: parse them as well
                    if flags.starts_with('l') {
                        None
                    } else {
                        Some(BasicEntry {
                            path,
                            size
                        })
                    }
                } else {
                    None
                } 
            })
            .collect::<Vec<_>>())
    }

    fn extract(&self, folder: impl AsRef<Path>) -> Result<Self::Updater, Self::Error> {
        let files = HashMap::<String, u64>::from_iter(self.entries()?
            .into_iter()
            .map(|entry| (
                entry.path.to_string_lossy().to_string(),
                entry.size
            )));

        let total_size = files.values().sum::<u64>();

        std::fs::create_dir_all(folder.as_ref())?;

        let child = Command::new("tar")
            .stdout(Stdio::piped())
            .arg("-xhvf")
            .arg(&self.path)
            .arg("-C")
            .arg(folder.as_ref())
            .spawn()?;

        Ok(BasicUpdater::new(child, total_size, move |file| files.get(&file).copied()))
    }
}