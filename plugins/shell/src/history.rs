use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom, Write};

use indexmap::IndexSet;

use crate::HistoryConfig;

pub struct History {
    pub store: File,
    pub elements: IndexSet<String>,
    pub cap: usize,
}

impl History {
    pub fn new(history_config: &HistoryConfig) -> Result<History, std::io::Error> {
        let maybe_history_path =
            dirs::state_dir().map(|s| s.join("anyrun").join("shell").join("shell_history.txt"));

        if let Some(history_path) = maybe_history_path {
            if let Some(dir) = history_path.parent() {
                std::fs::create_dir_all(dir)?;
            }

            let file = File::options()
                .read(true)
                .write(true)
                .create(true)
                .truncate(false)
                .open(&history_path)?;

            History::from_file(history_config.capacity, file)
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "failed to get the user state directory",
            ))
        }
    }

    pub fn push(&mut self, value: String) -> Result<(), std::io::Error> {
        // insert_before ensures new usages of existing commands bubble up to the top of the history, simple `insert` does not
        self.elements.insert_before(self.elements.len(), value);

        if self.elements.len() > self.cap {
            let remove_count = self.elements.len().saturating_sub(self.cap);
            self.elements.drain(0..remove_count);
        }

        self.store.seek(SeekFrom::Start(0))?;
        self.store.set_len(0)?;
        for line in &self.elements {
            writeln!(self.store, "{}", line)?;
        }
        self.store.flush()?;

        Ok(())
    }

    fn from_file(cap: usize, file: File) -> Result<History, std::io::Error> {
        let elements: IndexSet<String> = BufReader::new(&file)
            .lines()
            .collect::<std::io::Result<_>>()?;

        Ok(History {
            store: file,
            elements,
            cap,
        })
    }
}
