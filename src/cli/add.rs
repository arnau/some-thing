use clap::Clap;
use directories::{BaseDirs, ProjectDirs, UserDirs};
use rustyline::{error::ReadlineError, Editor};
use skim::prelude::*;
use std::fs;
use std::path::PathBuf;

use crate::lenses;
use crate::store::{Store, DEFAULT_PATH};
use crate::thing::NewThingBuilder;
use crate::{Report, Result, SomeError};

const HISTORY_PATH: &str = "history.txt";

/// Add a new item to the collection.
#[derive(Debug, Clap)]
pub struct Cmd {
    /// Store path
    #[clap(long, value_name = "path", default_value = DEFAULT_PATH)]
    store_path: PathBuf,
}

impl Cmd {
    // Steps:
    //
    // 1. Ask for URL.
    // 2. Check URL not exists.
    // 3. (opt) Ping URL.
    // 4. Ask for name.
    // 5. Ask for summary.
    // 6. Ask for category. Offer current options.
    // 7. Ask for tags. Offer current options.
    pub fn run(&self) -> Result<Report> {
        let project_dirs = ProjectDirs::from("", "seachess", "some").ok_or(SomeError::ProjectDir);
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

        let mut store = Store::open(&self.store_path)?;
        let tx = store.transaction()?;
        let mut thing = NewThingBuilder::new();

        let mut editor = Editor::<()>::new();

        if let Ok(path) = &history_path {
            let _ = editor.load_history(&path);
        }

        thing.with_url(required(&mut editor, "url")?);
        thing.with_name(required(&mut editor, "name")?);

        if let Some(summary) = optional(&mut editor, "summary (optional)", Some(0))? {
            thing.with_summary(summary);
        };

        let tagset = lenses::tag::full_set(&tx)?;
        let items = tagset.as_skim_buffer();

        thing.with_category_id(
            select(tagset.as_skim_buffer(), "category")?.unwrap_or("miscellaneous".into()),
        );
        thing.with_tags(&multi_select(items.clone(), "tags")?);

        dbg!(&thing);

        // let report = lenses::thing::add(&tx)?;

        if let Ok(path) = &history_path {
            let _ = editor.save_history(path)?;
        }

        tx.commit()?;

        Ok(Report::new("Success"))
    }
}

fn required(editor: &mut Editor<()>, field: &str) -> Result<String> {
    optional(editor, field, None)?.ok_or(SomeError::FieldRequired(field.to_string()))
}

fn optional(editor: &mut Editor<()>, field: &str, limit: Option<u32>) -> Result<Option<String>> {
    let mut answer = None;
    let prompt = format!("{}: ", &field);
    let mut count = 0;

    loop {
        let readline = editor.readline(&prompt);
        match readline {
            Ok(line) => {
                editor.add_history_entry(line.as_str());

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

fn select(items: SkimItemReceiver, field: &str) -> Result<Option<String>> {
    let prompt = format!("{} (select one): ", field);
    let options = SkimOptionsBuilder::default()
        .height(Some("100%"))
        .multi(false)
        .preview(Some(""))
        .preview_window(Some("down:10%"))
        .prompt(Some(&prompt))
        .build()
        .map_err(SomeError::Unknown)?;

    let selected_item = Skim::run_with(&options, Some(items))
        .map(|out| out.selected_items)
        .unwrap_or_else(|| Vec::new())
        .first()
        .map(|item| item.output().to_string());

    Ok(selected_item)
}

fn multi_select(items: SkimItemReceiver, field: &str) -> Result<Vec<String>> {
    let prompt = format!("{} (select multiple): ", field);
    let options = SkimOptionsBuilder::default()
        .height(Some("100%"))
        .multi(true)
        .preview(Some(""))
        .preview_window(Some("down:10%"))
        .prompt(Some(&prompt))
        .build()
        .map_err(SomeError::Unknown)?;

    let selected_items = Skim::run_with(&options, Some(items))
        .map(|out| out.selected_items)
        .unwrap_or_else(|| Vec::new())
        .iter()
        .map(|item| item.output().to_string())
        .collect();

    Ok(selected_items)
}
