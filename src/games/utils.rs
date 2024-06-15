#[derive(Copy, Clone)]
pub struct Parabola {
    max: usize,
    duration: usize,
    time: usize,

    // Derivated value
    value: usize,
}

impl Parabola {
    pub fn new(max: usize, duration: usize) -> Parabola {
        Parabola {
            max,
            duration,
            time: 0,
            value: 0,
        }
    }

    pub fn step(&mut self) {
        self.time += 1;
        self.value = self.calc_value();
    }

    pub fn value(&self) -> usize {
        self.value
    }

    pub fn finished(&self) -> bool {
        self.time >= self.duration
    }

    fn calc_value(&self) -> usize {
        let a = 4 * self.max;
        let b = 4 * self.max * self.duration;

        if self.time >= self.duration {
            return 0;
        }

        (b * self.time - a * self.time * self.time) / (self.duration * self.duration)
    }
}
