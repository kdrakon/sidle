use core::borrow::Borrow;
use std::cell::{Ref, RefCell};
use std::cmp::Ordering;
use std::io::Write;
use std::ops::Deref;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc, RwLock};
use std::thread::JoinHandle;

use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use termion::terminal_size;

use crate::error_code;
use crate::error_code::ErrorCode;
use crate::Dir;
use crate::{DirObject, Either, State};

//pub fn start() -> (JoinHandle<Result<(), ErrorCode>>, Sender<Either<(), Arc<RwLock<State>>>>) {
//    let (sender, receiver): (Sender<Either<(), Arc<RwLock<State>>>>, Receiver<Either<(), Arc<RwLock<State>>>>) =
//        mpsc::channel();
//
//    let ui_thread_handle: JoinHandle<Result<(), u8>> = std::thread::spawn(|| {
//        let screen = &mut AlternateScreen::from(
//            std::io::stdout().into_raw_mode().map_err(|_| error_code::FAILED_TO_CREATE_UI_SCREEN)?,
//        );
//
//        let buffer_init = format!("{}", termion::clear::UntilNewline);
//        let mut terminal_line_buffers: Vec<String> = Vec::new();
//
//        // screen draw loop
//        for message in receiver {
//            let (width, height) = terminal_size().map_err(|_| error_code::COULD_NOT_DETERMINE_TERMINAL_SIZE)?;
//            terminal_line_buffers.resize(height as usize, buffer_init.clone());
//
//            match message {
//                Either::Left(_) => break,
//                Either::Right(state) => {
//                    let read_state = state.read().map_err(|_| error_code::COULD_NOT_OBTAIN_LOCK_ON_STATE)?;
//                    print_dir_contents(screen, height, terminal_line_buffers.as_mut_slice(), &read_state.dir)?;
//                    screen.flush().map_err(|_| error_code::FAILED_TO_FLUSH_UI_SCREEN)?;
//                }
//            }
//        }
//
//        screen.flush().map_err(|_| error_code::FAILED_TO_FLUSH_UI_SCREEN) // final flush before handing screen back to shell
//    });
//
//    (ui_thread_handle, sender.clone())
//}

pub fn render(state: &State, screen: &mut impl Write) -> Result<(), ErrorCode> {
    let buffer_init = format!("{}", termion::clear::UntilNewline);
    let mut terminal_line_buffers: Vec<String> = Vec::new();

    let (width, height) = terminal_size().map_err(|_| error_code::COULD_NOT_DETERMINE_TERMINAL_SIZE)?;
    terminal_line_buffers.resize(height as usize, buffer_init.clone());

    print_dir_contents(screen, height, terminal_line_buffers.as_mut_slice(), state.dir.borrow().deref())?;
    screen.flush().map_err(|_| error_code::FAILED_TO_FLUSH_UI_SCREEN) // final flush before handing screen back to shell
}

fn print_dir_contents(
    screen: &mut impl Write,
    terminal_height: u16,
    terminal_line_buffers: &mut [String],
    dir: &Dir,
) -> Result<(), ErrorCode> {
    for (index, content) in dir.contents.iter().enumerate() {
        let line = match content {
            DirObject::Dir { name, .. } => {
                let line = format!("{}{}{}", termion::style::Bold, name, termion::style::Reset);
                if index == dir.content_selection {
                    highlight_line(&line)
                } else {
                    line
                }
            }
            DirObject::File { name, .. } => {
                let line = format!("{}{}", name, termion::style::Reset);
                if index == dir.content_selection {
                    highlight_line(&line)
                } else {
                    line
                }
            }
        };
        terminal_line_buffers[index] = line;
    }

    write!(screen, "{}", termion::cursor::Goto(1, terminal_height)).map_err(|_| error_code::FAILED_TO_WRITE_TO_UI_SCREEN)?;
    for line in terminal_line_buffers {
        write!(screen, "{}", line).map_err(|_| error_code::FAILED_TO_WRITE_TO_UI_SCREEN)?;
        write!(screen, "{}{}", termion::cursor::Up(1), termion::cursor::Left(line.len() as u16))
            .map_err(|_| error_code::FAILED_TO_WRITE_TO_UI_SCREEN)?;
    }

    Ok(())
}

fn highlight_line(line: &str) -> String {
    format!("{}{}{}", termion::color::Bg(termion::color::LightWhite), termion::color::Fg(termion::color::Black), line)
}
