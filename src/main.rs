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
    ChangeMode(Mode),
    InsertChar(char),
    Quit,
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

            KeyCode::Char('h') | KeyCode::Left  => Some(Action::MoveLeft),
            KeyCode::Char('l') | KeyCode::Right => Some(Action::MoveUp),
            KeyCode::Char('k') | KeyCode::Up    => Some(Action::MoveRight),
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
    stdout.flush()?;

    Ok(())
}

fn main() -> std::io::Result<()> {
    let mut stdout = stdout();

    stdout.execute(terminal::EnterAlternateScreen)?;
    terminal::enable_raw_mode()?;

    stdout.execute(terminal::Clear(terminal::ClearType::All))?;

    let mut cursor_x = 0;
    let mut cursor_y = 0;

    let (mut cols, mut rows) = terminal::size()?;

    let mut mode = Mode::Normal;

    loop {
        draw_statusline(&mut stdout, rows, cols, &mut mode)?;

        stdout.queue(cursor::MoveTo(cursor_x, cursor_y))?;
        stdout.flush()?;

        if let Some(action) = handle_event(&mode, &mut stdout, &mut cols, &mut rows, read()?) {
            match action {
                Action::Quit => break,
                Action::MoveLeft => cursor_x = cursor_x.saturating_sub(1),
                Action::MoveUp => cursor_x += 1,
                Action::MoveRight => cursor_y = cursor_y.saturating_sub(1),
                Action::MoveDown => cursor_y += 1,
                Action::ChangeMode(m) => mode = m,
                Action::InsertChar(c) => {
                    print!("{}", c);
                    cursor_x += 1;
                },
            }
        }
    }

    stdout.execute(terminal::LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;

    Ok(())
}
