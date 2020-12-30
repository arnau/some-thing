use directories::ProjectDirs;
use rustyline::{error::ReadlineError, Editor};
use skim::prelude::*;
use std::fs;
use std::path::PathBuf;

pub mod add;
pub mod build;
pub mod destroy;
pub mod init;

use crate::{Result, SomeError};

const HISTORY_PATH: &str = "history.txt";
const PROJECT_TRIPLE: (&str, &str, &str) = ("", "seachess", "some");

/// A support for interacting with the user.
#[derive(Debug)]
pub struct Prompter {
    editor: Editor<()>,
    history_path: Option<PathBuf>,
}

impl Prompter {
    pub fn new() -> Result<Self> {
        let mut editor = Editor::<()>::new();
        let project_dirs = ProjectDirs::from(PROJECT_TRIPLE.0, PROJECT_TRIPLE.1, PROJECT_TRIPLE.2)
            .ok_or(SomeError::ProjectDir);
        let history_path = project_dirs.and_then(|pd| {
            let dir = pd.config_dir();
            let file = dir.join(HISTORY_PATH);

            if !dir.exists() {
                fs::create_dir_all(dir)?;
            }

            if !file.exists() {
                let _ = fs::File::create(&file)?;
            }

            Ok(file)
        });

        if let Ok(path) = &history_path {
            let _ = editor.load_history(&path);
        }

        let prompter = Self {
            editor,
            history_path: history_path.ok(),
        };

        Ok(prompter)
    }

    pub fn history_path(&self) -> Option<&PathBuf> {
        self.history_path.as_ref()
    }

    pub fn editor(&mut self) -> &mut Editor<()> {
        &mut self.editor
    }

    /// Ask for an input a `limit` amount of times, but allow no answer after that.
    pub fn read_line_times(&mut self, field: &str, limit: Option<u32>) -> Result<Option<String>> {
        let mut answer = None;
        let mut count = 0;
        let prompt = |count| match limit {
            Some(limit) => {
                if limit == 0 {
                    format!("{} (optional): ", &field)
                } else {
                    format!("{} ({}): ", &field, limit - count)
                }
            }
            None => format!("{} (required): ", &field),
        };

        loop {
            let readline = self.editor.readline(&prompt(count + 1));
            match readline {
                Ok(line) => {
                    self.keep_line(line.as_str());

                    if !line.trim().is_empty() {
                        answer = Some(line.trim().into());
                        break;
                    }

                    if let Some(limit) = limit {
                        if count >= limit {
                            break;
                        }

                        count = count + 1;
                    }
                }
                Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
                    break;
                }
                Err(err) => return Err(SomeError::Readline(err)),
            }
        }

        Ok(answer)
    }

    /// Require for an input. Error if the user triggers `CTRL-C` or `CTRL-D`.
    pub fn demand(&mut self, field: &str) -> Result<String> {
        self.read_line_times(field, None)?
            .ok_or(SomeError::FieldRequired(field.to_string()))
    }

    /// Ask for an input, just once.
    pub fn ask_once(&mut self, field: &str) -> Result<Option<String>> {
        self.read_line_times(&field, Some(0))
    }

    /// Ask for an input, many times.
    pub fn ask_times(&mut self, field: &str, times: u32) -> Result<Option<String>> {
        self.read_line_times(&field, Some(times))
    }

    pub fn read_choices<T: Into<SkimItemReceiver>>(
        &mut self,
        items: T,
        field: &str,
    ) -> Result<Vec<String>> {
        let prompt = format!("{} (select many): ", field);
        let options = SkimOptionsBuilder::default()
            .height(Some("100%"))
            .multi(true)
            .preview(Some(""))
            .preview_window(Some("down:10%"))
            .prompt(Some(&prompt))
            .build()
            .map_err(SomeError::Unknown)?;

        let selected_items = Skim::run_with(&options, Some(items.into()))
            .map(|out| out.selected_items)
            .unwrap_or_else(|| Vec::new())
            .iter()
            .map(|item| item.output().to_string())
            .collect();

        Ok(selected_items)
    }

    /// Ask to pick a single choice.
    pub fn read_choice<T: Into<SkimItemReceiver>>(
        &mut self,
        items: T,
        field: &str,
    ) -> Result<Option<String>> {
        let prompt = format!("{} (select one): ", field);
        let options = SkimOptionsBuilder::default()
            .height(Some("100%"))
            .multi(false)
            .preview(Some(""))
            .preview_window(Some("down:10%"))
            .prompt(Some(&prompt))
            .build()
            .map_err(SomeError::Unknown)?;

        let selected_item = Skim::run_with(&options, Some(items.into()))
            .map(|out| out.selected_items)
            .unwrap_or_else(|| Vec::new())
            .first()
            .map(|item| item.output().to_string());

        Ok(selected_item)
    }

    /// Record a history line.
    ///
    /// To persist history records you need to use `flush`.
    pub fn keep_line<S: AsRef<str> + Into<String>>(&mut self, line: S) {
        self.editor.add_history_entry(line);
    }

    pub fn flush(&mut self) -> Result<()> {
        if let Some(path) = &self.history_path {
            let _ = self.editor.save_history(path)?;
        }

        Ok(())
    }
}
