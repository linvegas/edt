use std::env;
use std::fs::File;
use std::io::{
    self, stdout,
    Write, Stdout, BufReader, BufRead,
};

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
    name: String,
}

impl Buffer {
    fn from_file(filepath: &str) -> Self {
        let file = File::open(filepath).unwrap();
        let file = BufReader::new(file);
        let content = file.lines().map(|line| line.unwrap()).collect();
        Self {
            lines: content, name: filepath.to_string(),
        }
    }

    // fn insert_new_line(&mut self) {
    //     let prev_line_indent = self.lines[self.c_row]
    //         .chars()
    //         .take_while(|c| c.is_whitespace() && *c != '\n')
    //         .map(|c| c.len_utf8())
    //         .sum();
    //     self.lines.insert(self.c_row + 1, String::from(" ".repeat(prev_line_indent)));
    //     self.c_row += 1;
    //     self.c_col = prev_line_indent;
    // }

    // fn insert_char(c_row: &Editor.c_row, c_col: &Editor.c_col, c: char) {
    //     lines[c_row].insert(c_col, c);
    //     self.c_col += 1;
    // }

    // fn delete_char(&mut self) {
    //     self.lines[self.c_row].remove(self.c_col - 1);
    //     self.c_col -= 1;
    // }
}

struct Editor {
    mode: Mode,
    stdout: Stdout,
    c_row: usize,
    c_col: usize,
    c_col_prev: usize,
    size_col: u16,
    size_row: u16,
    scroll: usize,
    buffer: Buffer,
}

impl Editor {
    fn new(buffer: Buffer) -> Self {
        let mut stdout = stdout();
        let (size_col, size_row) = terminal::size().unwrap();

        stdout
            .execute(terminal::EnterAlternateScreen).unwrap()
            .execute(terminal::Clear(terminal::ClearType::All)).unwrap();

        terminal::enable_raw_mode().unwrap();

        Self {
            mode: Mode::Normal,
            stdout, size_col, size_row,
            c_row: 0, c_col: 0, c_col_prev: 0,
            buffer, scroll: 0,
        }
    }

    fn render(&mut self) {
        self.stdout.execute(terminal::Clear(terminal::ClearType::All)).unwrap();

        self.render_statuslines();
        self.render_buffer();

        self.stdout.queue(cursor::MoveTo(self.c_col as u16, self.c_row as u16)).unwrap();

        self.stdout.flush().unwrap();
    }

    fn render_buffer(&mut self) {
        for i in 0..self.size_row - 1 {
            let line =  match self.buffer.lines.get(i as usize + self.scroll) {
                None => String::new(),
                Some(s) => s.to_string(),
            };
            self.stdout
                .queue(cursor::MoveTo(0, i as u16)).unwrap()
                .queue(style::Print(line)).unwrap();
        }
        // for (i, line) in self.buffer.lines.iter().enumerate() {
        //     if i < (self.size_row - 1).into() {
        //         self.stdout.queue(cursor::MoveTo(0, i as u16)).unwrap();
        //         self.stdout.queue(style::Print(line)).unwrap();
        //     }
        // }
    }

    fn render_statuslines(&mut self) {
        let current_mode = match self.mode {
            Mode::Normal => "NOR",
            Mode::Insert => "INS",
        };

        self.stdout
            .queue(cursor::MoveTo(0, self.size_row)).unwrap()
            .queue(style::Print(" ".repeat(self.size_col as usize).on(Color::DarkMagenta))).unwrap();

        self.stdout
            .queue(cursor::MoveTo(1, self.size_row)).unwrap()
            .queue(style::Print(current_mode.with(Color::Black).on(Color::DarkMagenta).bold())).unwrap();

        self.stdout
            .queue(cursor::MoveTo((current_mode.len() + 3).try_into().unwrap(), self.size_row)).unwrap()
            .queue(style::Print(self.buffer.name.to_string().with(Color::Black).on(Color::DarkMagenta).bold())).unwrap();
    }

    // fn v_height(&mut self) -> u16 {
    //     self.size_row - 2
    // }

    fn insert_char(&mut self, c: char) {
        self.buffer.lines[self.c_row].insert(self.c_col, c);
        self.c_col += 1;
    }

    fn delete_char(&mut self) {
        self.buffer.lines[self.c_row].remove(self.c_col - 1);
        self.c_col -= 1;
    }

    fn insert_new_line(&mut self) {
        let prev_line_indent = self.buffer.lines[self.c_row]
            .chars()
            .take_while(|c| c.is_whitespace() && *c != '\n')
            .map(|c| c.len_utf8())
            .sum();
        self.buffer.lines.insert(self.c_row + 1, String::from(" ".repeat(prev_line_indent)));
        self.c_row += 1;
        self.c_col = prev_line_indent;
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

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    let filepath = &args[1];

    let buf = Buffer::from_file(filepath);

    let mut edt = Editor::new(buf);

    loop {
        edt.render();

        // TODO: Move this code to Editor
        if let Some(action) = handle_event(&edt.mode, &mut edt.stdout, &mut edt.size_col, &mut edt.size_row, read()?) {
            match action {
                Action::Quit => break,
                Action::MoveLeft => {
                    edt.c_col = edt.c_col.saturating_sub(1);
                    edt.c_col_prev = edt.c_col;
                },
                Action::MoveRight => {
                    if usize::from(edt.c_col) < edt.buffer.lines[edt.c_row as usize].len() {
                        edt.c_col += 1;
                        edt.c_col_prev = edt.c_col;
                    }
                    // buf.c_col = buf.c_col.saturating_sub(1),
                },
                Action::MoveUp => {
                    if edt.c_row > 0 {
                        edt.c_row -= 1;
                    } else {
                        edt.scroll -= 1
                    }
                    if edt.buffer.lines[edt.c_row].len() == 0 {
                        edt.c_col = 0;
                    } else {
                        edt.c_col = edt.c_col_prev;
                    }
                    if edt.c_col > edt.buffer.lines[edt.c_row as usize].len() {
                        edt.c_col = edt.buffer.lines[edt.c_row as usize].len();
                    }
                },
                Action::MoveDown => {
                    if edt.c_row < (edt.size_row - 2).into() {
                        edt.c_row += 1;
                    } else {
                        edt.scroll += 1
                    }
                    if edt.buffer.lines[edt.c_row as usize].len() == 0 {
                        edt.c_col = 0;
                    } else {
                        edt.c_col = edt.c_col_prev;
                    }
                    if usize::from(edt.c_col) > edt.buffer.lines[edt.c_row as usize].len() {
                        edt.c_col = edt.buffer.lines[edt.c_row as usize].len();
                    }
                },
                Action::InsertNewLine => {
                    match edt.mode {
                        Mode::Normal => edt.mode = Mode::Insert,
                        _ => {}
                    }
                    edt.insert_new_line();
                }
                Action::ChangeMode(m) => edt.mode = m,
                Action::InsertChar(c) => edt.insert_char(c),
                Action::DeleteChar => edt.delete_char(),
            }
        }
    }

    // TODO: Implement le Drop
    edt.stdout.execute(terminal::LeaveAlternateScreen).unwrap();
    terminal::disable_raw_mode()?;

    Ok(())
}
