use std::io::Write;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread::JoinHandle;

use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;

use crate::{DirObject, Either};
use crate::error_code;

#[derive(Clone)]
pub struct UIState<'a> {
    pub dir_contents: Vec<DirObject<'a>>
}

pub fn start() -> (JoinHandle<Result<(), u8>>, Sender<Either<(), UIState<'static>>>) {
    let (sender, receiver): (Sender<Either<(), UIState>>, Receiver<Either<(), UIState>>) = mpsc::channel();

    let ui_thread_handle: JoinHandle<Result<(), u8>> = std::thread::spawn(|| {
        let screen = &mut AlternateScreen::from(std::io::stdout().into_raw_mode().map_err(|_|error_code::FAILED_TO_CREATE_UI_SCREEN)?);

        // screen draw loop
        for message in receiver {
            match message {
                Either::Left(_) => break,
                Either::Right(state) => {
                    write!(screen, "{}{}state changed", termion::cursor::Goto(1, 1), termion::style::Bold).map_err(|_| error_code::FAILED_TO_WRITE_TO_UI_SCREEN)?;
                    screen.flush().map_err(|_| error_code::FAILED_TO_FLUSH_UI_SCREEN)?;
                }
            }
        }

        screen.flush().map_err(|_| error_code::FAILED_TO_FLUSH_UI_SCREEN) // final flush before handing screen back to shell
    });

    (ui_thread_handle, sender.clone())
}