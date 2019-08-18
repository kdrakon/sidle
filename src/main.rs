use std::borrow::BorrowMut;
use std::cell::{Ref, RefCell};
use std::cmp::Ordering;
use std::env;
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use clap::{App, Arg};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use termion::terminal_size;

use crate::dir_object::{DirObject, IntoDirObject};
use crate::error_code::ErrorCode;
use std::io::stdout;
use std::io::Write;

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
}

#[derive(Clone)]
pub struct Dir {
    pub path: PathBuf,
    pub contents: Vec<DirObject>,
    pub content_selection: usize,
}

fn main() -> Result<(), ErrorCode> {
    let _args: Vec<String> = env::args().collect();

    let matches = App::new("sidle").version(VERSION).arg(Arg::with_name("path").required(false)).get_matches();

        let current_dir_path = match matches.value_of("path") {
            None => std::env::current_dir().map_err(|_| error_code::COULD_NOT_LIST_DIR)?,
            Some(path) => PathBuf::from(path),
        };
        let dir_contents = read_dir(&current_dir_path)?;

        let mut state =
            State { dir: Dir { path: current_dir_path, contents: dir_contents, content_selection: 0 }, parents: vec![] };

    // termion alternate screen scope
    {
        let mut screen =
            AlternateScreen::from(std::io::stdout().into_raw_mode().map_err(|_| error_code::FAILED_TO_CREATE_UI_SCREEN)?);

        ui::render(&state, &mut screen)?;

        for key_event in std::io::stdin().keys() {
            let key = key_event.map_err(|err| error_code::KEY_INPUT_ERROR)?;
            if key == Key::Char('q') || key == Key::Char('\n') {
                break;
            } else {
                state = new_state(state, key)?;
                ui::render(&state, &mut screen)?;
            }
        }
    }

    write!(stdout(), "{}\n", state.dir.path.to_str().unwrap());

    Ok(())
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
        Key::Right => {
            let dir_name =
                current_state.dir.contents.get(current_state.dir.content_selection).and_then(|selection| match selection {
                    DirObject::File { .. } | DirObject::Unknown { .. } => None,
                    DirObject::Dir { name, .. } => Some(name.clone()),
                });

            match dir_name {
                None => Ok(current_state),
                Some(dir_name) => {
                    let parent_dir = current_state.dir.clone();
                    current_state.dir.path.push(dir_name);
                    current_state.dir.contents = read_dir(&current_state.dir.path)?;
                    current_state.dir.content_selection = 0;
                    current_state.parents.push(parent_dir);
                    Ok(current_state)
                }
            }
        }
        Key::Left => {
            let mut parents = current_state.parents;
            let parent = match parents.pop() {
                Some(parent) => parent,
                None => {
                    current_state.dir.path.pop();
                    let contents = read_dir(&current_state.dir.path)?;
                    Dir { path: current_state.dir.path, contents, content_selection: 0 }
                }
            };
            Ok(State { dir: parent, parents })
        }
        _ => Ok(current_state),
    }
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
