use std::io::Write;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread::JoinHandle;

use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;

use crate::{DirObject, Either};
use crate::DirObject::*;
use crate::error_code;

#[derive(Clone)]
pub struct UIState {
    pub dir_contents: Vec<DirObject>
}

pub fn start() -> (JoinHandle<Result<(), u8>>, Sender<Either<(), UIState>>) {
    let (sender, receiver): (Sender<Either<(), UIState>>, Receiver<Either<(), UIState>>) = mpsc::channel();

    let ui_thread_handle: JoinHandle<Result<(), u8>> = std::thread::spawn(|| {
        let screen = &mut AlternateScreen::from(std::io::stdout().into_raw_mode().map_err(|_| error_code::FAILED_TO_CREATE_UI_SCREEN)?);

        // screen draw loop
        for message in receiver {
            write!(screen, "{}{}", termion::clear::All, termion::cursor::Goto(1, 1)).map_err(|_| error_code::FAILED_TO_WRITE_TO_UI_SCREEN)?;
            match message {
                Either::Left(_) => break,
                Either::Right(state) => {
                    for content in state.dir_contents {
                        match content {
                            Dir(name) => write!(screen, "{}{}", termion::cursor::Down(1), name),
                            File(name) => write!(screen, "{}{}", termion::cursor::Down(1), name),
                            HiddenFile(name) => write!(screen, "{}{}", termion::cursor::Down(1), name),
                        };
                    }
                    screen.flush().map_err(|_| error_code::FAILED_TO_FLUSH_UI_SCREEN)?;
                }
            }
        }

        screen.flush().map_err(|_| error_code::FAILED_TO_FLUSH_UI_SCREEN) // final flush before handing screen back to shell
    });

    (ui_thread_handle, sender.clone())
}