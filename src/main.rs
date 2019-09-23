use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use clap::{App, Arg};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;

use crate::dir_object::{DirObject, IntoDirObject};
use crate::error_code::ErrorCode;

mod dir_object;
mod error_code;
mod ui;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

pub enum Either<A, B> {
    Left(A),
    Right(B),
}

#[derive(Clone)]
pub struct State {
    pub dir: Dir,
    pub parents: Vec<Dir>,
    pub prev_selections: Vec<(PathBuf, usize)>,
    pub file_selection: FileSelection,
}

#[derive(Clone)]
pub struct Dir {
    pub path: PathBuf,
    pub contents: Vec<DirObject>,
    pub content_selection: usize,
}

#[derive(Clone)]
pub struct FileSelection {
    pub files_selectable: bool,
    pub file_selected: Option<PathBuf>,
}

fn main() -> Result<(), ErrorCode> {
    let _args: Vec<String> = env::args().collect();

    let matches = App::new("sidle")
        .version(VERSION)
        .arg(Arg::with_name("path").required(false).takes_value(true))
        .arg(
            Arg::with_name("output")
                .required(false)
                .short("o")
                .takes_value(true)
                .help("Where to write the final path chosen. Defaults to the file 'sidle_path' in a temp directory"),
        )
        .arg(
            Arg::with_name("files_selectable")
                .required(false)
                .long("files-selectable")
                .takes_value(false)
                .help("Allows files, in addition to directories, to be selected as output"),
        )
        .get_matches();

    let output_path = match matches.value_of("output") {
        Some(output) => PathBuf::from(output),
        None => {
            let mut temp_path = std::env::temp_dir();
            temp_path.push("sidle_path");
            temp_path
        }
    };

    let current_dir_path = match matches.value_of("path") {
        None => std::env::current_dir().map_err(|_| error_code::COULD_NOT_LIST_DIR)?,
        Some(path) => PathBuf::from(path),
    };
    let dir_contents = read_dir(&current_dir_path)?;

    let file_selection = FileSelection { files_selectable: matches.is_present("files_selectable"), file_selected: None };

    let mut state = State {
        dir: Dir { path: current_dir_path, contents: dir_contents, content_selection: 0 },
        parents: vec![],
        prev_selections: vec![],
        file_selection,
    };

    // termion alternate screen scope
    {
        let mut terminal_line_buffers: Vec<String> = Vec::new();
        let mut screen =
            AlternateScreen::from(std::io::stdout().into_raw_mode().map_err(|_| error_code::FAILED_TO_CREATE_UI_SCREEN)?);

        let _hide_cursor_scope = termion::cursor::HideCursor::from(std::io::stdout());

        ui::render(&state, &mut terminal_line_buffers, &mut screen, false)?;

        for key_event in std::io::stdin().keys() {
            let key = key_event.map_err(|_err| error_code::KEY_INPUT_ERROR)?;
            state = new_state(state, key)?;
            if key == Key::Char('q') {
                Err(error_code::ABORT)?
            } else if key == Key::Char('\n') || key == Key::Char('.') {
                if let Some(file_path) = state.file_selection.file_selected {
                    write_path(&output_path, file_path.to_str().expect("Error converting directory path to string"))?
                } else {
                    write_path(&output_path, state.dir.path.to_str().expect("Error converting directory path to string"))?
                };
                break;
            } else {
                ui::render(&state, &mut terminal_line_buffers, &mut screen, key == Key::Left || key == Key::Right)?;
            }
        }
    }

    Ok(())
}

fn read_dir(path: &PathBuf) -> Result<Vec<DirObject>, ErrorCode> {
    let mut vec: Vec<DirObject> = vec![];
    for dir_result in std::fs::read_dir(path).map_err(|_| error_code::COULD_NOT_LIST_DIR)? {
        let dir_entry = dir_result.map_err(|_| error_code::COULD_NOT_LIST_DIR)?;
        let dir_object = dir_entry.new_dir_object()?;
        vec.push(dir_object);
    }
    vec.sort_by(dir_object::dir_ordering);
    Ok(vec)
}

fn write_path(path: &PathBuf, content: &str) -> Result<(), ErrorCode> {
    let mut file = File::create(path).map_err(|_| error_code::ERROR_WRITING_TO_OUTPUT)?;
    write!(file, "{}\n", content).map_err(|_| error_code::ERROR_WRITING_TO_OUTPUT)
}

fn new_state(mut current_state: State, key: Key) -> Result<State, ErrorCode> {
    match key {
        Key::Up => {
            current_state.dir.content_selection =
                std::cmp::min(current_state.dir.contents.len() - 1, current_state.dir.content_selection + 1);
            Ok(current_state)
        }
        Key::Down => {
            if current_state.dir.content_selection >= 1 {
                current_state.dir.content_selection -= 1
            }
            Ok(current_state)
        }
        Key::Right | Key::Char('\n') => {
            let dir_object = current_state.dir.contents.get(current_state.dir.content_selection);

            match dir_object {
                None | Some(DirObject::Unknown { .. }) => Ok(current_state),
                Some(DirObject::File { name: _, path }) => {
                    if current_state.file_selection.files_selectable {
                        current_state.file_selection.file_selected = Some(path.clone())
                    }
                    Ok(current_state)
                }
                Some(DirObject::Dir { name: dir_name, .. }) => {
                    let parent_dir = current_state.dir.clone();
                    let prev_selection = current_state.prev_selections.pop();
                    current_state.file_selection.file_selected = None;
                    current_state.dir.path.push(dir_name);
                    current_state.dir.contents = read_dir(&current_state.dir.path)?;
                    current_state.dir.content_selection = match &prev_selection {
                        None => 0,
                        Some((last_path, selection)) => {
                            if last_path == &current_state.dir.path {
                                *selection
                            } else {
                                0
                            }
                        }
                    };
                    current_state.parents.push(parent_dir);
                    Ok(current_state)
                }
            }
        }
        Key::Char('.') => {
            current_state.file_selection.file_selected = None;
            Ok(current_state)
        }
        Key::Left => {
            let mut parents = current_state.parents;
            let mut prev_selections = current_state.prev_selections;
            prev_selections.push((current_state.dir.path.clone(), current_state.dir.content_selection));

            let parent = match parents.pop() {
                Some(parent) => parent,
                None => {
                    let existing_path = current_state.dir.path.clone();
                    let dir_selected = current_state
                        .dir
                        .path
                        .file_name()
                        .and_then(|p| p.to_str())
                        .map(|p| DirObject::Dir { name: String::from(p), path: existing_path });
                    current_state.dir.path.pop();
                    let contents = read_dir(&current_state.dir.path)?;
                    let content_selection = {
                        if let Some(dir_selected) = dir_selected {
                            contents.binary_search_by(|dir_object| dir_object::dir_ordering(dir_object, &dir_selected)).ok()
                        } else {
                            None
                        }
                    };
                    Dir { path: current_state.dir.path, contents, content_selection: content_selection.unwrap_or(0) }
                }
            };
            Ok(State { dir: parent, parents, prev_selections, file_selection: current_state.file_selection })
        }
        _ => Ok(current_state),
    }
}
