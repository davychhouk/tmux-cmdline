use std::{env, fs, io, path::PathBuf};

use ratatui::{
    DefaultTerminal,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    prelude::*,
    widgets::Paragraph,
};

#[derive(Default)]
struct Editor {
    input: String,
    cursor: usize,
}

enum Action {
    Continue,
    Submit,
    Cancel,
}

impl Editor {
    fn handle(&mut self, key: KeyEvent) -> Action {
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char('a') => self.cursor = 0,
                KeyCode::Char('e') => self.cursor = self.input.len(),
                KeyCode::Char('u') => {
                    self.input.drain(..self.cursor);
                    self.cursor = 0;
                }
                KeyCode::Char('k') => self.input.truncate(self.cursor),
                KeyCode::Char('w') => self.delete_word(),
                KeyCode::Char('c') => return Action::Cancel,
                _ => return Action::Continue,
            }
            return Action::Continue;
        }

        match key.code {
            KeyCode::Char(c) => {
                self.input.insert(self.cursor, c);
                self.cursor += c.len_utf8();
            }
            KeyCode::Left => self.cursor = self.previous_boundary(),
            KeyCode::Right => self.cursor = self.next_boundary(),
            KeyCode::Home => self.cursor = 0,
            KeyCode::End => self.cursor = self.input.len(),
            KeyCode::Backspace if self.cursor > 0 => {
                let previous = self.previous_boundary();
                self.input.drain(previous..self.cursor);
                self.cursor = previous;
            }
            KeyCode::Delete if self.cursor < self.input.len() => {
                self.input.drain(self.cursor..self.next_boundary());
            }
            KeyCode::Enter => return Action::Submit,
            KeyCode::Esc => return Action::Cancel,
            _ => {}
        }

        Action::Continue
    }

    fn previous_boundary(&self) -> usize {
        self.input[..self.cursor]
            .char_indices()
            .next_back()
            .map_or(0, |(index, _)| index)
    }

    fn next_boundary(&self) -> usize {
        self.input[self.cursor..]
            .chars()
            .next()
            .map_or(self.cursor, |c| self.cursor + c.len_utf8())
    }

    fn delete_word(&mut self) {
        let trimmed = self.input[..self.cursor].trim_end_matches(char::is_whitespace);
        let start = trimmed
            .char_indices()
            .rev()
            .find(|(_, c)| c.is_whitespace())
            .map_or(0, |(index, c)| index + c.len_utf8());
        self.input.drain(start..self.cursor);
        self.cursor = start;
    }
}

fn main() -> io::Result<()> {
    let output = env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "missing output file"))?;
    let mut editor = Editor::default();
    let submitted = ratatui::run(|terminal| run(terminal, &mut editor))?;

    if submitted && !editor.input.trim().is_empty() {
        editor.input.push('\n');
        fs::write(output, editor.input)?;
    }

    Ok(())
}

fn run(terminal: &mut DefaultTerminal, editor: &mut Editor) -> io::Result<bool> {
    loop {
        terminal.draw(|frame| render(frame, editor))?;
        if let Event::Key(key) = event::read()?
            && matches!(key.kind, KeyEventKind::Press | KeyEventKind::Repeat)
        {
            match editor.handle(key) {
                Action::Continue => {}
                Action::Submit => return Ok(true),
                Action::Cancel => return Ok(false),
            }
        }
    }
}

fn render(frame: &mut Frame, editor: &Editor) {
    let area = frame.area();
    if area.is_empty() {
        return;
    }

    let cursor = 2 + Line::from(&editor.input[..editor.cursor]).width() as u16;
    let scroll = cursor.saturating_sub(area.width.saturating_sub(1));
    let line = Line::from(vec![
        Span::styled("> ", Style::default().fg(Color::Cyan)),
        Span::raw(&editor.input),
    ]);

    frame.render_widget(Paragraph::new(line).scroll((0, scroll)), area);
    frame.set_cursor_position((area.x + cursor - scroll, area.y));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn edits_unicode_on_character_boundaries() {
        let mut editor = Editor::default();
        for c in "new 🦀 window".chars() {
            editor.handle(key(KeyCode::Char(c)));
        }
        editor.handle(key(KeyCode::Home));
        editor.handle(key(KeyCode::Right));
        editor.handle(key(KeyCode::Delete));
        editor.handle(key(KeyCode::End));
        editor.handle(KeyEvent::new(KeyCode::Char('w'), KeyModifiers::CONTROL));

        assert_eq!(editor.input, "nw 🦀 ");
        assert_eq!(editor.cursor, editor.input.len());
    }
}
