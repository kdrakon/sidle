use std::cell::{Ref, RefCell};
use std::cmp::Ordering;
use std::io::Write;
use std::sync::{Arc, mpsc, RwLock};
use std::sync::mpsc::{Receiver, Sender};
use std::thread::JoinHandle;

use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use termion::terminal_size;

use crate::{DirObject, Either, State};
use crate::DirObject::*;
use crate::error_code;
use crate::error_code::ErrorCode;
use core::borrow::Borrow;

pub fn start() -> (JoinHandle<Result<(), ErrorCode>>, Sender<Either<(), Arc<RwLock<State>>>>) {
    let (sender, receiver): (Sender<Either<(), Arc<RwLock<State>>>>, Receiver<Either<(), Arc<RwLock<State>>>>) = mpsc::channel();

    let ui_thread_handle: JoinHandle<Result<(), u8>> = std::thread::spawn(|| {
        let screen = &mut AlternateScreen::from(
            std::io::stdout().into_raw_mode().map_err(|_| error_code::FAILED_TO_CREATE_UI_SCREEN)?,
        );

        let buffer_init = format!("{}", termion::clear::UntilNewline);
        let mut terminal_line_buffers: Vec<String> = Vec::new();

        // screen draw loop
        for message in receiver {
            let (width, height) = terminal_size().map_err(|_| error_code::COULD_NOT_DETERMINE_TERMINAL_SIZE)?;
            terminal_line_buffers.resize(height as usize, buffer_init.clone());

            match message {
                Either::Left(_) => break,
                Either::Right(state) => {
                    for (index, content) in state.read().unwrap().dir.contents.iter().enumerate() {
                        let line = match content {
                            Dir { name, .. } => format!("{}", name),
                            File { name, .. } => format!("{}", name),
                        };
                        terminal_line_buffers[index] = line;
                    }
                    write!(screen, "{}", termion::cursor::Goto(1, 1)).map_err(|_| error_code::FAILED_TO_WRITE_TO_UI_SCREEN)?;
                    for line in terminal_line_buffers.as_slice() {
                        write!(screen, "{}", line).map_err(|_| error_code::FAILED_TO_WRITE_TO_UI_SCREEN)?;
                        write!(screen, "{}{}", termion::cursor::Down(1), termion::cursor::Left(line.len() as u16))
                            .map_err(|_| error_code::FAILED_TO_WRITE_TO_UI_SCREEN)?;
                    }
                    screen.flush().map_err(|_| error_code::FAILED_TO_FLUSH_UI_SCREEN)?;
                }
            }
        }

        screen.flush().map_err(|_| error_code::FAILED_TO_FLUSH_UI_SCREEN) // final flush before handing screen back to shell
    });

    (ui_thread_handle, sender.clone())
}


