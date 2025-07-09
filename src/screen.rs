use std::{
    sync::{Arc, RwLock},
    time::Duration,
};

use ::termwiz::{
    color::ColorAttribute,
    input::InputEvent,
    surface::{Change, CursorVisibility},
    terminal::{Terminal, buffered::BufferedTerminal},
};

struct ScreenState<T: Terminal> {
    screen_size: [i32; 2],
    buf: BufferedTerminal<T>,
}

pub struct Screen<T: Terminal> {
    state: Arc<RwLock<ScreenState<T>>>,
}

impl<T: Terminal> Clone for Screen<T> {
    fn clone(&self) -> Self {
        Self {
            state: Arc::clone(&self.state),
        }
    }
}

impl<T: Terminal> Screen<T> {
    pub fn new(terminal: T) -> anyhow::Result<Self> {
        let mut buf = BufferedTerminal::new(terminal)?;
        let screen_size = buf.terminal().get_screen_size()?;

        let new = Self {
            state: Arc::new(RwLock::new(ScreenState {
                buf: buf,
                screen_size: [screen_size.cols as _, screen_size.rows as _],
            })),
        };

        Ok(new)
    }

    pub fn screen_size(&self) -> [i32; 2] {
        self.state.read().unwrap().screen_size
    }

    fn _flush(state: &mut ScreenState<T>) -> anyhow::Result<()> {
        state.buf.flush()?;

        Ok(())
    }

    fn _add_changes(state: &mut ScreenState<T>, changes: Vec<Change>) -> anyhow::Result<()> {
        state.buf.add_changes(changes);

        Ok(())
    }

    fn _add_change(state: &mut ScreenState<T>, change: Change) -> anyhow::Result<()> {
        state.buf.add_change(change);

        Ok(())
    }

    pub fn flush(&self) -> anyhow::Result<()> {
        let mut state = self.state.write().unwrap();
        Self::_flush(&mut state)
    }

    pub fn add_change(&self, change: Change) -> anyhow::Result<()> {
        let mut state = self.state.write().unwrap();
        Self::_add_change(&mut state, change)
    }

    pub fn add_changes(&self, changes: Vec<Change>) -> anyhow::Result<()> {
        let mut state = self.state.write().unwrap();
        Self::_add_changes(&mut state, changes)
    }

    pub fn initialize(&self) -> anyhow::Result<()> {
        let mut state = self.state.write().unwrap();

        state.buf.terminal().set_raw_mode()?;

        Self::_add_changes(
            &mut state,
            vec![
                Change::CursorVisibility(CursorVisibility::Hidden),
                Change::Title("pipes.rs".to_string()),
            ],
        )?;

        Ok(())
    }

    pub fn clear(&self, transparent: bool) -> anyhow::Result<()> {
        self.add_change(Change::ClearScreen(if transparent {
            ColorAttribute::Default
        } else {
            ColorAttribute::PaletteIndex(0)
        }))
    }

    pub fn poll_input(&self) -> anyhow::Result<Option<InputEvent>> {
        let mut state = self.state.write().unwrap();

        match state.buf.terminal().poll_input(Some(Duration::ZERO))? {
            Some(InputEvent::Resized { cols, rows }) => {
                state.screen_size = [cols as _, rows as _];

                Ok(Some(InputEvent::Resized { cols, rows }))
            }
            event => Ok(event),
        }
    }
}
