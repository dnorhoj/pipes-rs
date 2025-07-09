use std::ops::Range;

use ::anyhow::anyhow;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    UP,
    RIGHT,
    DOWN,
    LEFT,
}

impl Direction {
    pub fn from_index(idx: i32) -> Self {
        if idx < 0 || idx > 3 {
            println!("Invalid index: {}", idx);
            panic!();
        }

        match idx {
            0 => Direction::UP,
            1 => Direction::RIGHT,
            2 => Direction::DOWN,
            3 => Direction::LEFT,
            _ => unreachable!(),
        }
    }

    pub fn get_index(&self) -> i32 {
        match self {
            Direction::UP => 0,
            Direction::RIGHT => 1,
            Direction::DOWN => 2,
            Direction::LEFT => 3,
        }
    }

    pub fn left(&self) -> Self {
        Self::from_index((self.get_index() - 1).rem_euclid(4))
    }

    pub fn right(&self) -> Self {
        Self::from_index((self.get_index() + 1).rem_euclid(4))
    }
}

pub struct PipeSet {
    pipes: String,
}

pub const PIPESETS: [&'static str; 6] = [
    "┃━┏┛┓┗", // original
    "│─┌┘┐└", // light
    "║═╔╝╗╚", // double lines
    "██████", // blocky set
    "╽╼╭╯╮╰", // dashed/light style
    "┃━╭╯╮╰", // hybrid round corners
];

impl PipeSet {
    pub fn new(pipes: String) -> anyhow::Result<Self> {
        if pipes.chars().count() != 6 {
            return Err(anyhow!("Invalid pipeset: {:?}", pipes.len()));
        }

        Ok(Self { pipes })
    }

    pub fn vertical(&self) -> char {
        self.pipes.chars().nth(0).unwrap()
    }

    pub fn horizontal(&self) -> char {
        self.pipes.chars().nth(1).unwrap()
    }

    pub fn top_left(&self) -> char {
        self.pipes.chars().nth(2).unwrap()
    }

    pub fn bottom_right(&self) -> char {
        self.pipes.chars().nth(3).unwrap()
    }

    pub fn top_right(&self) -> char {
        self.pipes.chars().nth(4).unwrap()
    }

    pub fn bottom_left(&self) -> char {
        self.pipes.chars().nth(5).unwrap()
    }

    pub fn get_pipeset(set: usize) -> anyhow::Result<Self> {
        Ok(Self::new(
            (*PIPESETS
                .get(set)
                .ok_or(anyhow!("No pipeset with id {set}"))?)
            .into(),
        )?)
    }

    pub fn pipeset_idx_range() -> Range<usize> {
        0..PIPESETS.len()
    }
}

impl Default for PipeSet {
    fn default() -> Self {
        Self::get_pipeset(0).unwrap()
    }
}
