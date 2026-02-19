use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom, Write};

use indexmap::IndexSet;

use crate::HistoryConfig;

pub enum HistoryBackingStore {
    File(File),
    Memory,
}

pub struct History {
    pub backing_store: HistoryBackingStore,
    pub elements: IndexSet<String>,
    pub cap: usize,
}

impl History {
    pub fn new(history_config: &HistoryConfig) -> History {
        let maybe_history_file =
            dirs::state_dir().map(|s| s.join("anyrun").join("shell").join("shell_history.txt"));

        let backing_store = if let Some(history_file) = maybe_history_file {
            let file = (|| {
                if let Some(dir) = history_file.parent() {
                    std::fs::create_dir_all(dir)?;
                }

                File::options()
                    .read(true)
                    .write(true)
                    .create(true)
                    .truncate(false)
                    .open(&history_file)
            })();

            match file {
                Ok(f) => HistoryBackingStore::File(f),
                Err(ref err) => {
                    eprintln!("[shell] Failed to create file {} to persist shell plugin history, falling back to in-memory: {}", &history_file.to_string_lossy(), err.kind());
                    HistoryBackingStore::Memory
                }
            }
        } else {
            HistoryBackingStore::Memory
        };

        match backing_store {
            HistoryBackingStore::File(file) => History::from_file(history_config.capacity, file)
                .unwrap_or_else(|err| {
                    eprintln!("[shell] Failed to initialize history from file: {:?}", err);
                    History::from_mem(history_config.capacity)
                }),
            HistoryBackingStore::Memory => History::from_mem(history_config.capacity),
        }
    }

    pub fn push(&mut self, value: String) -> Result<(), std::io::Error> {
        // insert_before ensures new usages of existing commands bubble up to the top of the history, simple `insert` does not
        self.elements.insert_before(self.elements.len(), value);

        if self.elements.len() > self.cap {
            let remove_count = self.elements.len().saturating_sub(self.cap);
            self.elements.drain(0..remove_count);
        }

        if let HistoryBackingStore::File(file) = &mut self.backing_store {
            file.seek(SeekFrom::Start(0))?;
            file.set_len(0)?;
            for line in &self.elements {
                writeln!(file, "{}", line)?;
            }
            file.flush()?;
        }

        Ok(())
    }

    fn from_mem(cap: usize) -> History {
        History {
            backing_store: HistoryBackingStore::Memory,
            elements: IndexSet::new(),
            cap,
        }
    }

    fn from_file(cap: usize, file: File) -> Result<History, std::io::Error> {
        let elements: IndexSet<String> = BufReader::new(&file)
            .lines()
            .collect::<std::io::Result<_>>()?;

        Ok(History {
            backing_store: HistoryBackingStore::File(file),
            elements,
            cap,
        })
    }
}
