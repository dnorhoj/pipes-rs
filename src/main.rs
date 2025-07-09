use std::{
    thread::sleep,
    time::{Duration, Instant},
};

use ::anyhow::anyhow;
use ::clap::Parser;
use ::rand::Rng;
use ::termwiz::{
    caps::Capabilities,
    cell::AttributeChange,
    color::ColorAttribute,
    input::{InputEvent, KeyCode},
    surface::{Change, Position},
    terminal::{Terminal, new_terminal},
};

use crate::pipe::Pipe;
use crate::random::Random;
use crate::screen::Screen;
use crate::utils::PipeSet;

mod pipe;
mod random;
mod screen;
mod utils;

struct PipesRs<R: Rng, T: Terminal> {
    random: Random<R>,
    screen: Screen<T>,
    pipes: Vec<Pipe<R, T>>,
    frames_since_clear: usize,
    running: bool,
    debug: bool,
    args: Args,
}

impl<R: Rng, T: Terminal> PipesRs<R, T> {
    pub fn new(random: Random<R>, screen: Screen<T>, args: Args) -> anyhow::Result<Self> {
        let mut new = Self {
            random,
            screen,
            pipes: Vec::new(),
            frames_since_clear: 0,
            running: false,
            debug: args.debug,
            args,
        };

        new.screen.initialize()?;
        new.screen.clear(new.args.transparent)?;

        for _ in 0..new.args.pipes {
            new.add_pipe()?;
        }

        Ok(new)
    }

    pub fn add_pipe(&mut self) -> anyhow::Result<()> {
        let pipeset = if self.args.random_pipeset {
            self.random.random_pipeset()?
        } else {
            PipeSet::get_pipeset(self.args.pipeset)?
        };

        self.pipes.push(Pipe::new(
            Screen::clone(&self.screen),
            Random::clone(&self.random),
            pipeset,
            self.args.straight_bias,
            self.args.keep_colors,
        )?);

        Ok(())
    }

    pub fn remove_pipe(&mut self) -> anyhow::Result<()> {
        self.pipes.pop();
        self.clear()?;
        self.frames_since_clear = 0;

        Ok(())
    }

    fn update_pipes(&mut self) -> anyhow::Result<()> {
        for pipe in self.pipes.iter_mut() {
            let (position, character) = pipe.get_move();

            if !self.args.no_colors {
                self.screen
                    .add_change(Change::Attribute(AttributeChange::Foreground(
                        ColorAttribute::PaletteIndex(pipe.get_color()),
                    )))?;
            }

            self.screen.add_changes(vec![
                Change::CursorPosition {
                    x: Position::Absolute(position[0] as _),
                    y: Position::Absolute(position[1] as _),
                },
                Change::Text(character.to_string()),
            ])?;
        }

        Ok(())
    }

    fn handle_input(&mut self) -> anyhow::Result<()> {
        while let Some(input) = self.screen.poll_input()? {
            match input {
                InputEvent::Key(key_event) => match key_event.key {
                    KeyCode::Char('q') => {
                        self.stop();
                    }
                    KeyCode::Char('c') => {
                        self.clear()?;
                        self.frames_since_clear = 0;
                    }
                    KeyCode::Char('d') => {
                        self.debug = !self.debug;
                        self.clear()?;
                    }
                    KeyCode::Char('+') => {
                        self.add_pipe()?;
                    }
                    KeyCode::Char('-') => {
                        self.remove_pipe()?;
                    }
                    _ => {}
                },
                InputEvent::Resized { cols: _, rows: _ } => {
                    self.clear()?;
                    self.frames_since_clear = 0;
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn draw_box(&self, position: [usize; 2], lines: Vec<String>) -> anyhow::Result<()> {
        let max_line_len = lines.iter().map(|i| i.len()).max().unwrap();

        let mut changes = Vec::new();

        changes.push(Change::Attribute(AttributeChange::Foreground(
            ColorAttribute::PaletteIndex(0xf),
        )));

        changes.push(Change::CursorPosition {
            x: Position::Absolute(position[0]),
            y: Position::Absolute(position[1]),
        });

        let border = "━".repeat(max_line_len + 2);
        changes.push(Change::Text(format!("┏{}┓", border)));

        for (i, line) in lines.iter().enumerate() {
            changes.push(Change::CursorPosition {
                x: Position::Absolute(position[0]),
                y: Position::Absolute(position[1] + i + 1),
            });

            changes.push(Change::Text(format!(
                "┃ {}{} ┃",
                line,
                " ".repeat(max_line_len - line.len())
            )));
        }

        changes.push(Change::CursorPosition {
            x: Position::Absolute(position[0]),
            y: Position::Absolute(position[1] + 1 + lines.len()),
        });

        changes.push(Change::Text(format!("┗{}┛", border)));

        self.screen.add_changes(changes)?;

        Ok(())
    }

    fn update_debug(&self, elapsed: Duration, max_ms_per_frame: f64) -> anyhow::Result<()> {
        self.draw_box(
            [2, 1],
            vec![
                format!("pipes.rs - Version {}", env!("CARGO_PKG_VERSION")),
                format!("Pipe count: {}", self.pipes.len()),
                format!(
                    "Frame: {}/{}",
                    self.frames_since_clear, self.args.frame_clear
                ),
                format!(
                    "MSPT: {:.2}/{:.2}",
                    elapsed.as_secs_f64() * 1000.,
                    max_ms_per_frame
                ),
            ],
        )
    }

    pub fn stop(&mut self) -> () {
        self.running = false;
    }

    pub fn cleanup(&mut self) -> anyhow::Result<()> {
        self.screen.add_change(Change::CursorPosition {
            x: Position::Absolute(0),
            y: Position::Absolute(0),
        })?;

        self.screen.flush()
    }

    fn clear(&self) -> anyhow::Result<()> {
        self.screen.clear(self.args.transparent)
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        if self.running {
            return Err(anyhow!("Already running!"));
        }

        self.running = true;
        let s_per_frame = 1. / self.args.fps as f64;
        let ms_per_frame = s_per_frame * 1000.;

        while self.running {
            let frame_start = Instant::now();

            if self.frames_since_clear == self.args.frame_clear {
                self.frames_since_clear = 0;
                self.clear()?;
            }

            self.frames_since_clear += 1;

            self.update_pipes()?;
            self.handle_input()?;

            let elapsed = frame_start.elapsed();

            let sleep_duration = if elapsed.as_secs_f64() >= s_per_frame {
                Duration::ZERO
            } else {
                Duration::from_secs_f64(s_per_frame) - elapsed
            };

            if self.debug {
                self.update_debug(elapsed, ms_per_frame)?;
            }

            self.screen.flush()?;

            sleep(sleep_duration);
        }

        self.cleanup()?;

        Ok(())
    }
}

#[derive(Parser, Debug)]
pub struct Args {
    /// Show debug information
    #[arg(short, long)]
    debug: bool,

    /// Amount of pipes to show
    #[arg(short, long, default_value_t = 1)]
    pipes: usize,

    /// Which pipeset to use (between 0-6)
    #[arg(short = 's', long, default_value_t = 0)]
    pipeset: usize,

    /// Frames per second
    #[arg(short, long, default_value_t = 60)]
    fps: usize,

    /// Amount of frames to show before clearing screen
    #[arg(short = 'c', long, default_value_t = 1000)]
    frame_clear: usize,

    /// Straight bias (higher values result in less turns)
    #[arg(short = 'b', long, default_value_t = 13)]
    straight_bias: usize,

    /// Use random pipeset for each pipe
    #[arg(long)]
    random_pipeset: bool,

    /// Disable colors :(
    #[arg(long)]
    no_colors: bool,

    /// Keep colors when pipes hit edge
    #[arg(long)]
    keep_colors: bool,

    /// Use transparent background (no background color)
    #[arg(long)]
    transparent: bool,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let mut pipes_rs = PipesRs::new(
        Random::new(),
        Screen::new(new_terminal(Capabilities::new_from_env()?)?)?,
        args,
    )?;

    pipes_rs.run()?;

    Ok(())
}
