use ::rand::Rng;
use ::termwiz::{color::PaletteIndex, terminal::Terminal};

use crate::random::Random;
use crate::screen::Screen;
use crate::utils::{Direction, PipeSet};

pub struct Pipe<R: Rng, T: Terminal> {
    random: Random<R>,
    position: [i64; 2],
    direction: Direction,
    color: PaletteIndex,
    screen: Screen<T>,
    pipeset: PipeSet,
    straight_bias: usize,
    keep_color: bool,
}

impl<R: Rng, T: Terminal> Pipe<R, T> {
    pub fn new(
        screen: Screen<T>,
        random: Random<R>,
        pipeset: PipeSet,
        straight_bias: usize,
        keep_color: bool,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            position: random.get_random_position(&screen.screen_size()),
            direction: random.get_random_direction(),
            color: random.get_random_color(),
            random,
            screen,
            pipeset,
            straight_bias,
            keep_color,
        })
    }

    fn change_color(&mut self) -> () {
        if !self.keep_color {
            self.color = self.random.get_random_color();
        }
    }

    pub fn get_color(&self) -> PaletteIndex {
        self.color
    }

    fn move_forward(&mut self) -> () {
        let size = self.screen.screen_size();

        match self.direction {
            Direction::UP => self.position[1] = self.position[1] - 1,
            Direction::RIGHT => self.position[0] = self.position[0] + 1,
            Direction::DOWN => self.position[1] = self.position[1] + 1,
            Direction::LEFT => self.position[0] = self.position[0] - 1,
        }

        if self.position[0] >= (size[0] as _)
            || self.position[0] < 0
            || self.position[1] >= (size[1] as _)
            || self.position[1] < 0
        {
            self.change_color();

            self.position = [
                self.position[0].rem_euclid(size[0] as _),
                self.position[1].rem_euclid(size[1] as _),
            ]
        }
    }

    fn get_corner(&self, before: Direction, after: Direction) -> char {
        match (before, after) {
            (Direction::UP, Direction::RIGHT) | (Direction::LEFT, Direction::DOWN) => {
                self.pipeset.top_left()
            }
            (Direction::RIGHT, Direction::UP) | (Direction::DOWN, Direction::LEFT) => {
                self.pipeset.bottom_right()
            }
            (Direction::UP, Direction::LEFT) | (Direction::RIGHT, Direction::DOWN) => {
                self.pipeset.top_right()
            }
            (Direction::LEFT, Direction::UP) | (Direction::DOWN, Direction::RIGHT) => {
                self.pipeset.bottom_left()
            }
            _ => panic!("Invalid wtf"),
        }
    }

    pub fn get_move(&mut self) -> ([i64; 2], char) {
        self.move_forward();

        let turning = self.random.random_ratio(1, self.straight_bias as _);

        (
            self.position,
            if turning {
                let dir = self.random.random_range(0..=6);

                let before = self.direction;

                self.direction = if dir > 2 {
                    // Turns left slightly more often (57%) to simulate natural wandering behavior
                    self.direction.left()
                } else {
                    self.direction.right()
                };

                self.get_corner(before, self.direction)
            } else {
                match self.direction {
                    Direction::UP | Direction::DOWN => self.pipeset.vertical(),
                    Direction::RIGHT | Direction::LEFT => self.pipeset.horizontal(),
                }
            },
        )
    }
}
