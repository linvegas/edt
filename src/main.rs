use std::io::{stdout, Write, Stdout};

use crossterm::{
    terminal, cursor,
    style::{self, Stylize, Color},
    event::{read, Event, KeyCode},
    ExecutableCommand, QueueableCommand
};

enum Mode {
    Normal,
    Insert
}

enum Action {
    MoveLeft,
    MoveUp,
    MoveRight,
    MoveDown,
    DeleteChar,
    InsertNewLine,
    ChangeMode(Mode),
    InsertChar(char),
    Quit,
}

struct Buffer {
    lines: Vec<String>,
    c_row: usize,
    c_col: usize,
    c_col_prev: usize,
}

impl Buffer {
    fn new() -> Self {
        Self {
            lines: vec![
                String::from("#include <stdio.h>"),
                String::from(""),
                String::from("int main() {"),
                String::from("    printf(\"Le Rust\\n\");"),
                String::from("    return 0;"),
                String::from("}"),
            ],
            c_row: 0, c_col: 0, c_col_prev: 0,
        }
    }

    fn insert_new_line(&mut self) {
        let prev_line_indent = self.lines[self.c_row]
            .chars()
            .take_while(|c| c.is_whitespace() && *c != '\n')
            .map(|c| c.len_utf8())
            .sum();
        self.lines.insert(self.c_row + 1, String::from(" ".repeat(prev_line_indent)));
        self.c_row += 1;
        self.c_col = prev_line_indent;
    }

    fn insert_char(&mut self, c: char) {
        self.lines[self.c_row].insert(self.c_col, c);
        self.c_col += 1;
    }

    fn delete_char(&mut self) {
        self.lines[self.c_row].remove(self.c_col - 1);
        self.c_col -= 1;
    }
}

fn handle_event(mode: &Mode, stdout: &mut Stdout, cols: &mut u16, rows: &mut u16, event: Event) -> Option<Action> {
    match event {
        Event::Resize(width, height) => {
            *cols = width; *rows = height;
        },
        _ => {},
    };
    match mode {
        Mode::Normal => handle_normal_mode(event),
        Mode::Insert => handle_insert_mode(event, stdout),
    }
}

fn handle_normal_mode(event: Event) -> Option<Action> {
    match event {
        Event::Key(event) => match event.code {
            KeyCode::Char('q') => Some(Action::Quit),

            KeyCode::Char('i') => Some(Action::ChangeMode(Mode::Insert)),
            KeyCode::Char('o') => Some(Action::InsertNewLine),

            KeyCode::Char('h') | KeyCode::Left  => Some(Action::MoveLeft),
            KeyCode::Char('l') | KeyCode::Right => Some(Action::MoveRight),
            KeyCode::Char('k') | KeyCode::Up    => Some(Action::MoveUp),
            KeyCode::Char('j') | KeyCode::Down  => Some(Action::MoveDown),
            _ => None,
        },
        _ => None
    }
}

fn handle_insert_mode(event: Event, _stdout: &mut Stdout) -> Option<Action> {
    match event {
        Event::Key(event) => match event.code {
            KeyCode::Esc => Some(Action::ChangeMode(Mode::Normal)),
            KeyCode::Backspace => Some(Action::DeleteChar),
            KeyCode::Enter => Some(Action::InsertNewLine),
            KeyCode::Char(c) => Some(Action::InsertChar(c)),
            _ => None,
        },
        _ => None
    }
}

fn draw_statusline(stdout: &mut Stdout, rows: u16, cols: u16, mode: &mut Mode) -> Result<(), std::io::Error> {
    let current_mode = match mode {
        Mode::Normal => "NOR",
        Mode::Insert => "INS",
    };

    stdout.queue(cursor::MoveTo(0, rows))?;
    stdout.queue(style::Print(" ".repeat(cols as usize).with(Color::White).on(Color::DarkGrey)))?;
    stdout.queue(cursor::MoveTo(1, rows))?;
    stdout.queue(style::Print(current_mode.with(Color::White).on(Color::DarkGrey).bold()))?;
    // stdout.flush()?;

    Ok(())
}

fn main() -> std::io::Result<()> {
    let mut stdout = stdout();

    stdout.execute(terminal::EnterAlternateScreen)?;
    terminal::enable_raw_mode()?;

    stdout.execute(terminal::Clear(terminal::ClearType::All))?;

    let (mut cols, mut rows) = terminal::size()?;

    let mut mode = Mode::Normal;

    let mut buf = Buffer::new();

    buf.c_row = 0;
    buf.c_col = 0;

    loop {
        stdout.execute(terminal::Clear(terminal::ClearType::All))?;
        draw_statusline(&mut stdout, rows, cols, &mut mode)?;

        for (i, line) in buf.lines.iter().enumerate() {
            stdout.queue(cursor::MoveTo(0, i as u16))?;
            stdout.queue(style::Print(line))?;
        }

        stdout.queue(cursor::MoveTo(buf.c_col as u16, buf.c_row as u16))?;
        stdout.flush()?;

        if let Some(action) = handle_event(&mode, &mut stdout, &mut cols, &mut rows, read()?) {
            match action {
                Action::Quit => break,
                Action::MoveLeft => {
                    buf.c_col = buf.c_col.saturating_sub(1);
                    buf.c_col_prev = buf.c_col;
                },
                Action::MoveRight => {
                    if buf.c_col < buf.lines[buf.c_row].len() {
                        buf.c_col += 1;
                        buf.c_col_prev = buf.c_col;
                    }
                    // buf.c_col = buf.c_col.saturating_sub(1),
                },
                Action::MoveUp => {
                    if buf.c_row > 0 {
                        buf.c_row -= 1;
                    }
                    if buf.lines[buf.c_row].len() == 0 {
                        buf.c_col = 0;
                    } else {
                        buf.c_col = buf.c_col_prev;
                    }
                    if buf.c_col > buf.lines[buf.c_row].len() {
                        buf.c_col = buf.lines[buf.c_row].len();
                    }
                },
                Action::MoveDown => {
                    buf.c_row += 1;
                    if buf.lines[buf.c_row].len() == 0 {
                        buf.c_col = 0;
                    } else {
                        buf.c_col = buf.c_col_prev;
                    }
                    if buf.c_col > buf.lines[buf.c_row].len() {
                        buf.c_col = buf.lines[buf.c_row].len();
                    }
                },
                Action::InsertNewLine => {
                    match mode {
                        Mode::Normal => mode = Mode::Insert,
                        _ => {}
                    }
                    buf.insert_new_line();
                }
                Action::ChangeMode(m) => mode = m,
                Action::InsertChar(c) => buf.insert_char(c),
                Action::DeleteChar => buf.delete_char(),
            }
        }
    }

    stdout.execute(terminal::LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;

    Ok(())
}
