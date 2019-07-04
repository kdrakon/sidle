use std::cmp::Ordering;
use std::env;
use std::path::PathBuf;
use std::time::Duration;

use clap::App;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use termion::terminal_size;

use crate::dir_object::{DirObject, IntoDirObject};
use crate::error_code::ErrorCode;
use std::sync::{Arc, RwLock};
use std::cell::{RefCell, Ref};

mod error_code;
mod dir_object;
mod ui;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

pub enum Either<A, B> {
    Left(A),
    Right(B),
}

pub struct State {
    pub dir: Dir
}

pub struct Dir {
    pub path: PathBuf,
    pub contents: Vec<DirObject>,
    pub content_selection: usize,
    pub parent: Option<Box<Dir>>
}

fn main() -> Result<(), ErrorCode> {
    let _args: Vec<String> = env::args().collect();

    let matches = App::new("sidle").version(VERSION).get_matches();

    let (ui_thread_handle, ui_sender) = ui::start();
    let current_dir_path = std::env::current_dir().map_err(|_| error_code::COULD_NOT_LIST_DIR)?;
    let dir_contents = read_dir(&current_dir_path)?;

    let mut state: Arc<RwLock<State>> = Arc::new(RwLock::new(State { dir: Dir { path: current_dir_path, contents: dir_contents, content_selection: 0, parent: None }}));
    ui_sender.send(Either::Right(state.clone())).map_err(|_| error_code::COULD_NOT_SEND_TO_UI_THREAD)?;

    for key_event in std::io::stdin().keys() {
        let key = key_event.map_err(|err| error_code::KEY_INPUT_ERROR)?;
        if key == Key::Char('q') {
            match ui_sender.send(Either::Left(())) {
                Ok(_) => break,
                Err(_) => Result::Err(error_code::FAILED_TO_TERMINATE_UI_THREAD)?,
            }
        } else {
//            new_state(&mut state, key);
            ui_sender.send(Either::Right(state.clone())).map_err(|_| error_code::COULD_NOT_SEND_TO_UI_THREAD)?;
        }
    }

    ui_thread_handle.join().map_err(|_| error_code::FAILED_TO_TERMINATE_UI_THREAD)?
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

fn new_state(current_state: &mut State, key: Key) {
    match key {
        _ => {}
    }
}

fn dir_ordering(a: &DirObject, b: &DirObject) -> Ordering {
    match (a, b) {
        (DirObject::Dir { .. }, DirObject::File { .. }) => Ordering::Greater,
        (DirObject::File { .. }, DirObject::Dir { .. }) => Ordering::Less,
        (DirObject::Dir { name: a_name, .. }, DirObject::Dir { name: b_name, .. }) => name_ordering(a_name, b_name),
        (DirObject::File { name: a_name, .. }, DirObject::File { name: b_name, .. }) => name_ordering(a_name, b_name),
    }
}

fn name_ordering(a: &str, b: &str) -> Ordering {
    match (a, b) {
        (a, b) if a.starts_with('.') ^ b.starts_with('.') => {
            if a.starts_with('.') {
                Ordering::Less
            } else {
                Ordering::Greater
            }
        },
        (a, b) => b.cmp(a)
    }
}
