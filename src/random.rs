use std::sync::{Arc, Mutex};

use ::rand::Rng;
use ::rand::distr::uniform::{SampleRange, SampleUniform};
use ::rand::rngs::ThreadRng;
use ::termwiz::color::PaletteIndex;

use crate::utils::{Direction, PipeSet};

pub struct Random<R: Rng> {
    rng: Arc<Mutex<R>>,
}

impl<R: Rng> Clone for Random<R> {
    fn clone(&self) -> Self {
        Self {
            rng: Arc::clone(&self.rng),
        }
    }
}

impl Random<ThreadRng> {
    pub fn new() -> Self {
        Self {
            rng: Arc::new(Mutex::new(rand::rng())),
        }
    }
}

impl<R: Rng> Random<R> {
    pub fn get_random_position(&self, terminal_size: &[i32; 2]) -> [i64; 2] {
        let mut rng = self.rng.lock().unwrap();

        [
            rng.random_range(0..=terminal_size[0]) as _,
            rng.random_range(0..=terminal_size[1]) as _,
        ]
    }

    pub fn get_random_direction(&self) -> Direction {
        let mut rng = self.rng.lock().unwrap();

        Direction::from_index(rng.random_range(0..4))
    }

    pub fn get_random_color(&self) -> PaletteIndex {
        let mut rng = self.rng.lock().unwrap();

        rng.random()
    }

    pub fn random_ratio(&self, numerator: u32, denominator: u32) -> bool {
        let mut rng = self.rng.lock().unwrap();

        rng.random_ratio(numerator, denominator)
    }

    pub fn random_range<T, Ra>(&self, range: Ra) -> T
    where
        T: SampleUniform,
        Ra: SampleRange<T>,
    {
        let mut rng = self.rng.lock().unwrap();

        rng.random_range(range)
    }

    pub fn random_pipeset(&self) -> anyhow::Result<PipeSet> {
        let mut rng = self.rng.lock().unwrap();

        PipeSet::get_pipeset(rng.random_range(PipeSet::pipeset_idx_range()))
    }
}
