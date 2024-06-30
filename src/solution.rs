use crate::input::{Key, Keys};
use crate::log::log;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

const SOLUTION_SIZE: usize = 1 << 10;

struct SolutionGenerator {
    keys: Keys,
    phase: GeneratorPhase,
    phase_time_left: usize,
    random_generator: SmallRng,
}

#[derive(Copy, Clone, Debug)]
enum GeneratorPhase {
    LowFreq,
    HighFreq,
}

impl GeneratorPhase {
    fn sample(rng: &mut SmallRng) -> GeneratorPhase {
        match rng.gen_range(0..100) {
            0..=29 => GeneratorPhase::LowFreq,
            30..=99 => GeneratorPhase::HighFreq,
            // SAFETY: gen_range(0..100) will always be in the 0..100 range
            _ => unreachable!(),
        }
    }
}

impl SolutionGenerator {
    fn new() -> Self {
        Self {
            keys: Keys::new(),
            phase: GeneratorPhase::LowFreq,
            phase_time_left: 0,
            random_generator: SmallRng::from_seed(*b"This is a funny random seed !!!!"),
        }
    }

    fn next(&mut self) -> Keys {
        // Update Generator Phase
        if self.phase_time_left == 0 {
            self.phase = GeneratorPhase::sample(&mut self.random_generator);
            self.phase_time_left = self.random_generator.gen_range(50..100);
        } else {
            self.phase_time_left -= 1;
        }

        self.keys.update();

        match self.phase {
            GeneratorPhase::LowFreq => {
                log!("LowF");
                if self.random_generator.gen_range(0..20) == 0 {
                    if self.keys.any_pressed() {
                        self.keys = Keys::new(); // Release all
                    } else {
                        let key = Key::from_u8(self.random_generator.gen_range(0..5));
                        self.keys.press(key);
                    }
                }
            }
            GeneratorPhase::HighFreq => {
                log!("HighF");
                // Release keys
                for _ in 0..4 {
                    if self.random_generator.gen_range(0..2) < 0 {
                        let key = Key::from_u8(self.random_generator.gen_range(0..5));
                        self.keys.release(key);
                    }
                }

                // Press keys
                for _ in 0..4 {
                    if self.random_generator.gen_range(0..5) < 0 {
                        let key = Key::from_u8(self.random_generator.gen_range(0..5));
                        self.keys.press(key);
                    }
                }
            }
        }

        self.keys
    }

    fn random_key(&mut self) -> Key {
        match self.random_generator.gen_range(0..100) {
            0..=14 => Key::Up,
            15..=29 => Key::Down,
            30..=44 => Key::Left,
            45..=59 => Key::Right,
            60..=99 => Key::Space,
            // SAFETY: gen_range(0..100) will always be in the 0..100 range
            _ => unreachable!(),
        }
    }
}

pub struct Solution {
    first_index: usize,
    generator: SolutionGenerator,
    keys: [Keys; SOLUTION_SIZE],
}

impl Solution {
    pub fn new() -> Solution {
        let mut generator = SolutionGenerator::new();

        Solution {
            first_index: 0,
            keys: std::array::from_fn(|_| generator.next()),
            generator,
        }
    }

    pub fn keys(&self, time: usize) -> Keys {
        if time >= SOLUTION_SIZE {
            panic!("Index out of bounds");
        }

        let index = self.first_index.wrapping_add(time) & (SOLUTION_SIZE - 1);
        self.keys[index]
    }

    pub fn update(&mut self) {
        self.keys[self.first_index] = self.generator.next();
        self.first_index = self.first_index.wrapping_add(1) & (SOLUTION_SIZE - 1);
    }
}
