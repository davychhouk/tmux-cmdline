use std::{env, fs, io, path::PathBuf};

use ratatui::{
    DefaultTerminal,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    prelude::*,
    widgets::Paragraph,
};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

#[derive(Default)]
struct Editor {
    input: String,
    cursor: usize,
    commands: String,
    aliases: String,
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
    let mut editor = Editor {
        commands: env::var("TCU_COMMANDS").unwrap_or_default(),
        aliases: env::var("TCU_ALIASES").unwrap_or_default(),
        ..Editor::default()
    };
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

fn highlight<'a>(input: &'a str, commands: &str, aliases: &str, accent: Color) -> Vec<Span<'a>> {
    let mut spans = Vec::new();
    let mut start = 0;
    let mut command = true;

    while start < input.len() {
        let rest = &input[start..];
        let first = rest.chars().next().unwrap();
        let mut style = Style::default();
        let end = match first {
            c if c.is_whitespace() => {
                start
                    + rest
                        .find(|c: char| !c.is_whitespace())
                        .unwrap_or(rest.len())
            }
            ';' if rest[1..].chars().next().is_none_or(char::is_whitespace) => {
                command = true;
                style = style.fg(accent);
                start + 1
            }
            '{' | '}' => {
                command = first == '{';
                style = style.fg(accent);
                start + 1
            }
            '\'' | '"' => {
                let (end, closed) = quoted_end(input, start, first);
                if command && closed {
                    let word = &input[start + 1..end - 1];
                    let dynamic = first == '"' && (word.contains('$') || word.contains('\\'));
                    style = style
                        .fg(if dynamic || valid_command(word, commands, aliases) {
                            accent
                        } else {
                            Color::Red
                        })
                        .add_modifier(Modifier::BOLD);
                } else {
                    style = style.fg(Color::Green);
                }
                command = false;
                end
            }
            '#' if rest.starts_with("#{") => {
                command = false;
                style = style.fg(Color::Magenta);
                group_end(input, start + 1, '{', '}')
            }
            '#' if rest.starts_with("#[") => {
                command = false;
                style = style.fg(Color::Magenta);
                group_end(input, start + 1, '[', ']')
            }
            '#' if rest.starts_with("#(") => {
                command = false;
                style = style.fg(Color::Magenta);
                group_end(input, start + 1, '(', ')')
            }
            '#' => {
                style = style.fg(Color::DarkGray);
                input.len()
            }
            '$' => {
                command = false;
                style = style.fg(Color::Magenta);
                variable_end(input, start)
            }
            _ => {
                let end = word_end(input, start);
                if command {
                    command = false;
                    style = style
                        .fg(if valid_command(&input[start..end], commands, aliases) {
                            accent
                        } else {
                            Color::Red
                        })
                        .add_modifier(Modifier::BOLD);
                } else if first == '-' {
                    style = style.fg(Color::Yellow);
                }
                end
            }
        };

        spans.push(Span::styled(&input[start..end], style));
        start = end;
    }

    spans
}

fn valid_command(word: &str, commands: &str, aliases: &str) -> bool {
    if commands.is_empty()
        || commands
            .split_ascii_whitespace()
            .chain(aliases.split_ascii_whitespace())
            .any(|name| name == word)
    {
        return true;
    }

    let mut matches = commands
        .lines()
        .filter_map(|line| line.split_ascii_whitespace().next())
        .filter(|name| name.starts_with(word));
    matches.next().is_some() && matches.next().is_none()
}

fn quoted_end(input: &str, start: usize, quote: char) -> (usize, bool) {
    let mut escaped = false;
    for (offset, c) in input[start + quote.len_utf8()..].char_indices() {
        if quote == '"' && !escaped && c == '\\' {
            escaped = true;
        } else if !escaped && c == quote {
            return (start + quote.len_utf8() + offset + c.len_utf8(), true);
        } else {
            escaped = false;
        }
    }
    (input.len(), false)
}

fn group_end(input: &str, open: usize, opening: char, closing: char) -> usize {
    let mut depth = 0;
    let mut escaped = false;
    for (offset, c) in input[open..].char_indices() {
        if !escaped && c == '\\' {
            escaped = true;
            continue;
        }
        if !escaped && c == opening {
            depth += 1;
        } else if !escaped && c == closing {
            depth -= 1;
            if depth == 0 {
                return open + offset + c.len_utf8();
            }
        }
        escaped = false;
    }
    input.len()
}

fn variable_end(input: &str, start: usize) -> usize {
    let rest = &input[start + 1..];
    if rest.starts_with('{') {
        return group_end(input, start + 1, '{', '}');
    }
    start
        + 1
        + rest
            .find(|c: char| !(c.is_alphanumeric() || c == '_'))
            .unwrap_or(rest.len())
}

fn word_end(input: &str, start: usize) -> usize {
    let mut escaped = false;
    for (offset, c) in input[start..].char_indices() {
        if escaped {
            escaped = false;
        } else if c == '\\' {
            escaped = true;
        // offset > 0: never stop on the first char, so a word always makes
        // forward progress even when it starts with a delimiter.
        } else if offset > 0
            && (c.is_whitespace()
                || "{}'\"#$".contains(c)
                || c == ';'
                    && input[start + offset + 1..]
                        .chars()
                        .next()
                        .is_none_or(char::is_whitespace))
        {
            return start + offset;
        }
    }
    input.len()
}

fn render(frame: &mut Frame, editor: &Editor) {
    let area = frame.area();
    if area.is_empty() {
        return;
    }

    let prompt = env::var("TCU_PROMPT").unwrap_or_else(|_| "❯ ".into());
    let accent = env::var("TCU_ACCENT")
        .ok()
        .and_then(|c| c.parse().ok())
        .unwrap_or(Color::Cyan);

    let cursor = Line::from(prompt.as_str()).width() as u16
        + Line::from(&editor.input[..editor.cursor]).width() as u16;

    // Snap the scroll offset to a grapheme boundary so a double-width
    // glyph never straddles the left edge and leaves a gap. Must walk the
    // same units the renderer does: graphemes measured as strings (ZWJ
    // sequences count once, not per codepoint).
    let desired = cursor.saturating_sub(area.width.saturating_sub(1));
    let mut scroll = 0;
    for grapheme in prompt.graphemes(true).chain(editor.input.graphemes(true)) {
        if scroll >= desired {
            break;
        }
        scroll += grapheme.width() as u16;
    }
    let mut spans = vec![Span::styled(prompt.as_str(), Style::default().fg(accent))];
    spans.extend(highlight(
        &editor.input,
        &editor.commands,
        &editor.aliases,
        accent,
    ));
    let line = Line::from(spans);

    frame.render_widget(Paragraph::new(line).scroll((0, scroll)), area);
    frame.set_cursor_position((area.x + cursor - scroll, area.y));
}

#[cfg(test)]
mod tests {
    use ratatui::backend::TestBackend;

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

    #[test]
    fn scroll_never_splits_a_wide_character() {
        let mut editor = Editor::default();
        for c in "x 🦀🦀🦀".chars() {
            editor.handle(key(KeyCode::Char(c)));
        }

        // Line is "❯ x 🦀🦀🦀" (10 cols); a 6-col area wants scroll 5,
        // which would cut the first crab in half. Snapping moves it to 6.
        let mut terminal = Terminal::new(TestBackend::new(6, 1)).unwrap();
        terminal.draw(|frame| render(frame, &editor)).unwrap();
        terminal.backend().assert_buffer_lines(["🦀🦀  "]);
    }

    #[test]
    fn scroll_counts_multi_codepoint_graphemes_like_the_renderer() {
        // ZWJ sequence: 3 codepoints summing to 4 char-widths, but the
        // renderer measures it as one width-2 grapheme.
        assert_eq!("👩‍🔬".width(), 2);

        let mut editor = Editor::default();
        for c in "x 👩‍🔬a🦀".chars() {
            editor.handle(key(KeyCode::Char(c)));
        }

        // Line is "❯ x 👩‍🔬a🦀" (9 cols); a 3-col area needs scroll 7. Summing
        // per-codepoint widths overshoots to 8, splitting the crab and
        // parking the cursor inside it.
        let mut terminal = Terminal::new(TestBackend::new(3, 1)).unwrap();
        terminal.draw(|frame| render(frame, &editor)).unwrap();
        terminal.backend().assert_buffer_lines(["🦀 "]);
        assert_eq!(
            terminal.get_cursor_position().unwrap(),
            Position { x: 2, y: 0 }
        );
    }

    #[test]
    fn highlights_tmux_lexical_syntax() {
        let spans = highlight(
            "new-w -n 'work' -c $HOME ; display-message foo;bar ; nwe ; \"new-window\" -n foo ; \"nwe\" -n foo ; $TMUX_CMD -t target ; #{command} -t target # note",
            "new-session new\nnew-window neww\ndisplay-message display",
            "sv",
            Color::Cyan,
        );
        let styled: Vec<_> = spans
            .iter()
            .filter(|span| !span.content.trim().is_empty())
            .map(|span| (span.content.as_ref(), span.style.fg))
            .collect();

        assert_eq!(
            styled,
            [
                ("new-w", Some(Color::Cyan)),
                ("-n", Some(Color::Yellow)),
                ("'work'", Some(Color::Green)),
                ("-c", Some(Color::Yellow)),
                ("$HOME", Some(Color::Magenta)),
                (";", Some(Color::Cyan)),
                ("display-message", Some(Color::Cyan)),
                ("foo;bar", None),
                (";", Some(Color::Cyan)),
                ("nwe", Some(Color::Red)),
                (";", Some(Color::Cyan)),
                ("\"new-window\"", Some(Color::Cyan)),
                ("-n", Some(Color::Yellow)),
                ("foo", None),
                (";", Some(Color::Cyan)),
                ("\"nwe\"", Some(Color::Red)),
                ("-n", Some(Color::Yellow)),
                ("foo", None),
                (";", Some(Color::Cyan)),
                ("$TMUX_CMD", Some(Color::Magenta)),
                ("-t", Some(Color::Yellow)),
                ("target", None),
                (";", Some(Color::Cyan)),
                ("#{command}", Some(Color::Magenta)),
                ("-t", Some(Color::Yellow)),
                ("target", None),
                ("# note", Some(Color::DarkGray)),
            ]
        );
    }
}
