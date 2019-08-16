use std::borrow::BorrowMut;
use std::cell::{Ref, RefCell};
use std::cmp::Ordering;
use std::env;
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use clap::App;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use termion::terminal_size;

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
}

#[derive(Clone)]
pub struct Dir {
    pub path: PathBuf,
    pub contents: Vec<DirObject>,
    pub content_selection: usize,
}

fn main() -> Result<(), ErrorCode> {
    let _args: Vec<String> = env::args().collect();

    let matches = App::new("sidle").version(VERSION).get_matches();

    let mut screen =
        AlternateScreen::from(std::io::stdout().into_raw_mode().map_err(|_| error_code::FAILED_TO_CREATE_UI_SCREEN)?);

    let current_dir_path = std::env::current_dir().map_err(|_| error_code::COULD_NOT_LIST_DIR)?;
    let dir_contents = read_dir(&current_dir_path)?;

    let mut state =
        State { dir: Dir { path: current_dir_path, contents: dir_contents, content_selection: 0 }, parents: vec![] };

    ui::render(&state, &mut screen)?;

    for key_event in std::io::stdin().keys() {
        let key = key_event.map_err(|err| error_code::KEY_INPUT_ERROR)?;
        if key == Key::Char('q') {
            break;
        } else {
            state = new_state(state, key)?;
            ui::render(&state, &mut screen)?;
        }
    }

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
                    DirObject::File { .. } => None,
                    DirObject::Dir { name, .. } => Some(name.clone()),
                });

            match dir_name {
                None => (),
                Some(dir_name) => {
                    let parent_dir = current_state.dir.clone();
                    current_state.dir.path.push(dir_name);
                    current_state.dir.contents = read_dir(&current_state.dir.path)?;
                    current_state.parents.push(parent_dir);
                }
            }

            Ok(current_state)
        }
        Key::Left => {
            let mut parents = current_state.parents;
            let parent = match parents.pop() {
                None => unimplemented!(),
                Some(tail) => tail,
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
    vec.sort_by(dir_ordering);
    Ok(vec)
}

fn dir_ordering(a: &DirObject, b: &DirObject) -> Ordering {
    match (a, b) {
        (DirObject::Dir { .. }, DirObject::File { .. }) => Ordering::Less,
        (DirObject::File { .. }, DirObject::Dir { .. }) => Ordering::Greater,
        (DirObject::Dir { name: a_name, .. }, DirObject::Dir { name: b_name, .. }) => name_ordering(a_name, b_name),
        (DirObject::File { name: a_name, .. }, DirObject::File { name: b_name, .. }) => name_ordering(a_name, b_name),
    }
}

fn name_ordering(a: &str, b: &str) -> Ordering {
    match (a, b) {
        (a, b) if a.starts_with('.') ^ b.starts_with('.') => {
            if a.starts_with('.') {
                Ordering::Greater
            } else {
                Ordering::Less
            }
        }
        (a, b) => a.cmp(b),
    }
}
