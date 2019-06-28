use core::borrow::Borrow;
use std::env;
use std::fs::{DirEntry, read};
use std::io::{Error, stdin, Write};
use std::path::PathBuf;
use std::time::Duration;

use clap::App;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use termion::terminal_size;

use crate::ui::UIState;

mod ui;
mod error_code;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

pub enum Either<A, B> {
    Left(A),
    Right(B),
}

#[derive(Clone)]
pub enum DirObject {
    Dir(String),
    File(String),
    HiddenFile(String),
}

fn main() -> Result<(), u8> {
    let _args: Vec<String> = env::args().collect();

    let matches = App::new("sidle")
        .version(VERSION)
        .get_matches();

    let (ui_thread_handle, ui_sender) = ui::start();
    let current_dir = std::env::current_dir().map_err(|_| error_code::COULD_NOT_LIST_DIR)?;
    let dir_contents = read_dir_into_vec(current_dir)?;

    let mut ui_state = UIState {
        dir_contents
    };

    for key_event in std::io::stdin().keys() {
        let key = key_event.map_err(|err| error_code::KEY_INPUT_ERROR)?;
        if key == Key::Char('q') {
            match ui_sender.send(Either::Left(())) {
                Ok(_) => break,
                Err(_) => Result::Err(error_code::FAILED_TO_TERMINATE_UI_THREAD)?
            }
        } else {
            new_ui_state(&mut ui_state, key);
            ui_sender.send(Either::Right(ui_state.clone())).map_err(|_| error_code::COULD_NOT_SEND_TO_UI_THREAD)?;
        }
    }

    ui_thread_handle.join().map_err(|_| error_code::FAILED_TO_TERMINATE_UI_THREAD)?
}

fn read_dir_into_vec(path: PathBuf) -> Result<Vec<DirObject>, u8> {
    let mut vec: Vec<DirObject> = vec![];
    for dir_result in std::fs::read_dir(path).map_err(|_| error_code::COULD_NOT_LIST_DIR)? {
        let dir_entry = dir_result.map_err(|_| error_code::COULD_NOT_LIST_DIR)?;
        let name = dir_entry.file_name().to_str().ok_or(error_code::COULD_NOT_READ_METADATA)?.to_string();
        vec.push(DirObject::Dir(name));
    }
    Ok(vec)
}

fn new_ui_state(current_ui_state: &mut UIState, key: Key) {
    match key {
        _ => {}
    }
}
