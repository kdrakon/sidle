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
use std::time::Duration;

pub fn render(state: &State, screen: &mut impl Write, clear_screen: bool) -> Result<(), ErrorCode> {
    let buffer_init = format!("{}", termion::clear::UntilNewline);
    let mut terminal_line_buffers: Vec<String> = Vec::new();

    let (width, height) = terminal_size().map_err(|_| error_code::COULD_NOT_DETERMINE_TERMINAL_SIZE)?;
    terminal_line_buffers.resize(height as usize, buffer_init.clone());

    if clear_screen {
        write!(screen, "{}", termion::clear::AfterCursor).map_err(|_| error_code::FAILED_TO_WRITE_TO_UI_SCREEN)?;
    }

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
            DirObject::Dir { name, .. } => format!(
                "{}{}{}{}",
                termion::color::Fg(termion::color::LightCyan),
                termion::style::Bold,
                name,
                termion::style::Reset
            ),
            DirObject::File { name, .. } => format!("{}{}", name, termion::style::Reset),
            DirObject::Unknown { name, .. } => format!("{}{}", name, termion::style::Reset),
        };

        let line = if index == dir.content_selection { highlight_line(&line) } else { line };

        if index < terminal_line_buffers.len() {
            terminal_line_buffers[index] = line;
        }
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
    format!("{}{}", termion::style::Invert, line)
}
