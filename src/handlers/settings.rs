use super::KeyHandler;
use crate::application::App;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub fn handle(key: KeyEvent, app: &mut App) -> Option<KeyHandler> {
    match key.code {
        KeyCode::Esc => app.stop(),

        KeyCode::Tab => {
            app.switch_to_test();
            app.reset_test();
            return Some(KeyHandler::StandardTest);
        }

        KeyCode::Char(c) => {
            if let KeyModifiers::CONTROL = key.modifiers {
                if c == 'c' {
                    app.stop();
                    return None;
                }
            }

            match c {
                'h' => {}
                'j' => {}
                'k' => {}
                'l' => {}
                'q' => app.stop(),
                _ => {}
            }
        }

        KeyCode::Left => {}
        KeyCode::Down => {}
        KeyCode::Up => {}
        KeyCode::Right => {}

        _ => (),
    }
    None
}
