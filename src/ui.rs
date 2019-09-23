use core::borrow::Borrow;

use std::io::Write;
use std::ops::Deref;

use termion::terminal_size;

use crate::error_code;
use crate::error_code::ErrorCode;
use crate::Dir;
use crate::{DirObject, State};

pub fn render(
    state: &State,
    terminal_line_buffers: &mut Vec<String>,
    screen: &mut impl Write,
    clear_screen: bool,
) -> Result<(), ErrorCode> {
    let (_, height) = terminal_size().map_err(|_| error_code::COULD_NOT_DETERMINE_TERMINAL_SIZE)?;
    let buffer_init = format!("{}", termion::clear::UntilNewline);
    terminal_line_buffers.resize(height as usize - 1, buffer_init.clone()); // remove 1 for space to print the full path below

    if clear_screen {
        write!(screen, "{}", termion::clear::AfterCursor).map_err(|_| error_code::FAILED_TO_WRITE_TO_UI_SCREEN)?;
    }

    print_dir_contents(screen, terminal_line_buffers.as_mut_slice(), state.dir.borrow().deref())?;
    screen.flush().map_err(|_| error_code::FAILED_TO_FLUSH_UI_SCREEN) // final flush before handing screen back to shell
}

fn print_dir_contents(screen: &mut impl Write, terminal_line_buffers: &mut [String], dir: &Dir) -> Result<(), ErrorCode> {
    let buffer_len = terminal_line_buffers.len();
    let index_offset = (dir.content_selection as isize - buffer_len as isize + 1).max(0) as usize;

    for (index, content) in dir.contents.iter().skip(index_offset).take(buffer_len as usize).enumerate() {
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

        let line = if index == (dir.content_selection - index_offset) { highlight_line(&line) } else { line };

        if index < terminal_line_buffers.len() {
            terminal_line_buffers[index] = line;
        }
    }

    write!(
        screen,
        "{}{}{}{}{}",
        termion::cursor::Goto(1, buffer_len as u16 + 1),
        termion::color::Fg(termion::color::Magenta),
        "► ",
        dir.path.to_str().unwrap_or("-"),
        termion::style::Reset
    )
    .map_err(|_| error_code::FAILED_TO_WRITE_TO_UI_SCREEN)?;

    write!(screen, "{}", termion::cursor::Goto(1, buffer_len as u16))
        .map_err(|_| error_code::FAILED_TO_WRITE_TO_UI_SCREEN)?;
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
