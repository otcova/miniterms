use super::utils::Parabola;
use super::GameContext;
use crate::image::{Image, ImageAnimation, Origin, Sprite};
use crate::input::{Key, Keys};
use crate::log;
use crate::math::Pos;
use crate::pixel_canvas::PixelCanvas;
use crate::solution::Solution;
use rand::rngs::SmallRng;
use rand::{Rng, RngCore, SeedableRng};
use ratatui::style::Color;
use std::collections::VecDeque;

#[derive(Copy, Clone)]
enum EnemyModel {
    Cactus { model: u8 },
    Bird,
}

#[derive(Copy, Clone)]
struct Enemy {
    position: Pos<i32>,
    velocity: u8,
    model: EnemyModel,
}

#[derive(Copy, Clone)]
struct TRex {
    jump: Option<Parabola>,
    crouching: bool,
}

pub struct TRexGame {
    trex: TRex,
    trex_solution: TRex,
    enemies: VecDeque<Enemy>,
    enemy_cooldown: u16,
    random: SmallRng,
    frame_count: usize,
}

impl TRexGame {
    pub fn new() -> Self {
        let initial_trex = TRex {
            crouching: false,
            jump: None,
        };
        TRexGame {
            trex: initial_trex,
            trex_solution: initial_trex,
            frame_count: 0,
            enemies: VecDeque::new(),
            enemy_cooldown: 10,
            random: SmallRng::from_seed(*b"Seed chosen by a fair dice roll."),
        }
    }

    pub fn update(&mut self, game: &mut GameContext) {
        self.trex.update(game.keys);
        self.trex_solution.update(game.solution.keys(0));

        self.despawn_enemies();
        self.spawn_enemies(game);
        self.update_enemies();

        if self.collide(&self.trex, 0) {
            log!("Game Over");
        }

        self.frame_count += 1;
    }

    fn collide(&self, trex: &TRex, time: usize) -> bool {
        let frame_count = self.frame_count + time;
        let trex = trex.sprite(frame_count);

        self.enemies.iter().copied().any(|mut enemy| {
            enemy.position.x += enemy.velocity as i32 * time as i32;
            enemy.sprite(frame_count).collide(&trex)
        })
    }

    fn spawn_cactus(&mut self, game: &mut GameContext) {
        // TODO: Check game solution

        self.enemies.push_back(Enemy {
            position: Pos::new(game.size.width as i32, 0),
            velocity: 3,
            model: EnemyModel::Cactus { model: 0 },
        });
    }

    fn spawn_bird(&mut self, game: &mut GameContext) {
        // TODO: Check game solution

        let x = game.size.width as i32;
        let y = self.random.gen_range(1..=20);

        self.enemies.push_back(Enemy {
            position: Pos { x, y },
            velocity: self.random.gen_range(4..=7),
            model: EnemyModel::Bird,
        });
    }

    fn spawn_enemy(&mut self, game: &mut GameContext) {
        let spawn_cactus = self.random.next_u32() & 3 != 0 || self.frame_count < 100;

        if spawn_cactus {
            self.spawn_cactus(game);
        } else {
            self.spawn_bird(game);
        }
    }

    fn spawn_enemies(&mut self, game: &mut GameContext) {
        if self.enemy_cooldown == 0 {
            self.enemy_cooldown = self.random.gen_range(10..50);

            self.spawn_enemy(game);
        }

        self.enemy_cooldown -= 1;
    }

    fn update_enemies(&mut self) {
        for enemy in &mut self.enemies {
            enemy.position.x -= enemy.velocity as i32;
        }
    }

    fn despawn_enemies(&mut self) {
        const DESPAWN_Y_BARRIER: i32 = -32;

        if let Some(enemy) = self.enemies.front() {
            if enemy.position.x < DESPAWN_Y_BARRIER {
                self.enemies.pop_front();
            }
        }
    }

    #[allow(unused)]
    fn trex_solution(&self, solution: &Solution, time: usize) -> TRex {
        let mut trex = self.trex_solution;
        for t in 1..=time {
            let keys = solution.keys(t);
            trex.update(keys);
        }
        trex
    }
}

impl TRex {
    fn pos(&self) -> (i32, i32) {
        const TREX_Y: i32 = 4;
        (TREX_Y, self.jump.as_ref().map_or(0, |p| p.value() as i32))
    }

    fn update(&mut self, keys: Keys) {
        self.crouching = keys.pressing(Key::Down);
        self.handle_jump(keys);
    }

    fn handle_jump(&mut self, keys: Keys) {
        // Update Jump
        if let Some(parabola) = &mut self.jump {
            parabola.step();

            if parabola.finished() {
                self.jump = None;
            }
        }

        // Start Jump
        if self.jump.is_none() && keys.pressing(Key::Space) {
            let jump_height = if self.crouching { 6 } else { 25 };
            let jump_duration = if self.crouching { 8 } else { 22 };
            self.jump = Some(Parabola::new(jump_height, jump_duration));
        }
    }
}

//////////////////////////////////////////////////
//////////////// Draw Logic //////////////////////
//////////////////////////////////////////////////

impl TRex {
    pub fn sprite(&self, frame_count: usize) -> Sprite {
        let skin_frame_divisor = if self.jump.is_some() { 2 } else { 4 };
        let skin = if self.crouching {
            TREX_CROUCHING
        } else {
            TREX_RUNNING
        };

        let x = if self.jump.is_some() {
            0
        } else {
            (frame_count / skin_frame_divisor) as i32 & 1
        };

        Sprite {
            image: skin.image(frame_count / skin_frame_divisor),
            position: Pos::new(x, self.pos().1),
            origin: Pos::new(Origin::Min, Origin::Max),
        }
    }
}

impl TRexGame {
    pub fn draw(&self, canvas: &mut PixelCanvas) {
        canvas.draw(self.trex.sprite(self.frame_count));
        canvas.draw(self.trex_solution.sprite(self.frame_count));

        // Draw enemies
        for enemy in &self.enemies {
            canvas.draw(enemy.sprite(self.frame_count));
        }
    }
}

impl Enemy {
    fn skin(&self, frame_count: usize) -> Image {
        match self.model {
            EnemyModel::Cactus { model } => CACTUSES[model as usize],
            EnemyModel::Bird => BIRD.image(self.velocity as usize * frame_count / 16),
        }
    }

    fn sprite(&self, frame_count: usize) -> Sprite {
        Sprite {
            image: self.skin(frame_count),
            position: self.position,
            origin: Pos::new(Origin::Min, Origin::Max),
        }
    }
}

pub const TREX_RUNNING: ImageAnimation = ImageAnimation(&[
    Image {
        pixels: &[
            0b_0_1_1_1_1_1_1_0_0_0_0_0_0_0, //
            0b_1_1_1_1_0_0_1_1_0_0_0_0_0_0, //
            0b_1_1_1_1_0_0_1_1_0_0_0_0_0_0, //
            0b_1_1_1_1_1_1_1_1_0_0_0_0_0_0, //
            0b_0_0_0_0_1_1_1_1_0_0_0_0_0_0, //
            0b_0_0_1_1_1_1_1_1_0_0_0_0_0_0, //
            0b_0_0_0_0_0_1_1_1_1_0_0_0_0_1, //
            0b_0_0_0_1_1_1_1_1_1_1_0_0_1_1, //
            0b_0_0_0_1_0_1_1_1_1_1_1_1_1_1, //
            0b_0_0_0_0_0_1_1_1_1_1_1_1_1_1, //
            0b_0_0_0_0_0_1_1_1_1_1_1_1_1_0, //
            0b_0_0_0_0_0_0_1_1_1_1_1_1_0_0, //
            0b_0_0_0_0_0_0_0_1_1_1_1_0_0_0, //
            0b_0_0_0_0_0_0_1_1_0_1_1_0_0_0, //
            0b_0_0_0_0_0_0_0_0_0_0_1_0_0_0, //
            0b_0_0_0_0_0_0_0_0_0_1_1_0_0_0, //
        ],
        width: 14,
        color: Color::Red,
    },
    Image {
        pixels: &[
            0b_0_1_1_1_1_1_1_0_0_0_0_0_0_0, //
            0b_1_1_1_1_0_0_1_1_0_0_0_0_0_0, //
            0b_1_1_1_1_0_0_1_1_0_0_0_0_0_0, //
            0b_1_1_1_1_1_1_1_1_0_0_0_0_0_0, //
            0b_0_0_0_0_1_1_1_1_0_0_0_0_0_0, //
            0b_0_0_1_1_1_1_1_1_0_0_0_0_0_0, //
            0b_0_0_0_0_0_1_1_1_1_0_0_0_0_1, //
            0b_0_0_0_1_1_1_1_1_1_1_0_0_1_1, //
            0b_0_0_0_1_0_1_1_1_1_1_1_1_1_1, //
            0b_0_0_0_0_0_1_1_1_1_1_1_1_1_1, //
            0b_0_0_0_0_0_1_1_1_1_1_1_1_1_0, //
            0b_0_0_0_0_0_0_1_1_1_1_1_1_0_0, //
            0b_0_0_0_0_0_0_0_1_1_1_1_0_0_0, //
            0b_0_0_0_0_0_0_0_1_0_0_1_0_0_0, //
            0b_0_0_0_0_0_0_0_1_0_1_1_0_0_0, //
            0b_0_0_0_0_0_0_1_1_0_0_0_0_0_0, //
        ],
        width: 14,
        color: Color::Red,
    },
]);

pub const TREX_CROUCHING: ImageAnimation = ImageAnimation(&[
    Image {
        pixels: &[
            0b_0_1_1_1_1_1_1_0_0_0_0_0_0_0_0_0_0_0, //
            0b_1_1_1_1_0_0_1_1_0_1_1_1_1_0_0_0_0_1, //
            0b_1_1_1_1_1_1_1_1_1_1_1_1_1_1_0_0_1_1, //
            0b_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1, //
            0b_0_0_0_0_1_1_1_1_1_1_1_1_1_1_1_1_1_1, //
            0b_0_0_1_1_1_1_1_0_0_1_1_1_1_1_1_1_1_0, //
            0b_0_0_0_0_0_0_0_0_1_1_1_1_1_1_1_1_0_0, //
            0b_0_0_0_0_0_0_0_0_1_0_1_1_0_1_1_0_0_0, //
            0b_0_0_0_0_0_0_0_0_0_0_0_1_0_0_1_1_0_0, //
            0b_0_0_0_0_0_0_0_0_0_0_1_1_0_0_0_1_0_0, //
            0b_0_0_0_0_0_0_0_0_0_0_0_0_0_0_1_1_0_0, //
        ],
        width: 18,
        color: Color::Red,
    },
    Image {
        pixels: &[
            0b_0_1_1_1_1_1_1_0_0_0_0_0_0_0_0_0_0_0, //
            0b_1_1_1_1_0_0_1_1_0_1_1_1_1_0_0_0_0_1, //
            0b_1_1_1_1_1_1_1_1_1_1_1_1_1_1_0_0_1_1, //
            0b_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1, //
            0b_0_0_0_0_1_1_1_1_1_1_1_1_1_1_1_1_1_1, //
            0b_0_0_1_1_1_1_1_0_0_1_1_1_1_1_1_1_1_0, //
            0b_0_0_0_0_0_0_0_0_1_1_1_1_1_1_1_1_0_0, //
            0b_0_0_0_0_0_0_0_0_1_0_1_1_0_0_1_0_0_0, //
            0b_0_0_0_0_0_0_0_0_0_0_0_1_1_0_0_1_0_0, //
            0b_0_0_0_0_0_0_0_0_0_0_0_0_1_0_1_1_0_0, //
            0b_0_0_0_0_0_0_0_0_0_0_0_1_1_0_0_0_0_0, //
        ],
        width: 18,
        color: Color::Red,
    },
]);

pub const BIRD: ImageAnimation = ImageAnimation(&[
    Image {
        pixels: &[
            0b_0_0_0_0_0_0_0_0_0_1_1_1_0_0_0_0_0, //
            0b_0_0_0_0_0_0_0_0_0_1_1_1_1_0_0_0_0, //
            0b_0_0_0_0_0_0_1_1_0_1_1_1_1_1_0_0_0, //
            0b_0_0_0_0_0_1_1_1_0_1_1_0_0_1_0_0_0, //
            0b_0_0_1_1_1_1_1_1_1_1_1_0_0_1_1_1_1, //
            0b_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_0, //
            0b_0_0_0_0_1_1_1_1_1_1_1_1_1_1_0_0_0, //
            0b_0_0_1_1_1_1_1_1_1_1_1_1_1_1_0_0_0, //
            0b_0_0_0_0_0_1_1_1_1_1_1_1_1_0_0_0_0, //
        ],
        width: 17,
        color: Color::LightBlue,
    },
    Image {
        pixels: &[
            0b_0_0_0_0_0_0_0_0_0_1_1_1_0_0_0_0_0, //
            0b_0_0_0_0_0_0_0_0_0_1_1_1_1_0_0_0_0, //
            0b_0_0_0_0_0_0_0_0_0_1_1_1_1_1_0_0_0, //
            0b_0_0_0_0_0_0_0_0_0_1_1_0_0_1_0_0_0, //
            0b_0_0_1_1_1_1_1_1_1_1_1_0_0_1_1_1_1, //
            0b_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_0, //
            0b_0_0_0_0_1_1_1_1_1_1_1_1_1_1_0_0_0, //
            0b_0_0_1_1_1_1_1_1_1_1_1_1_1_1_0_0_0, //
            0b_0_0_0_0_0_1_1_1_1_1_1_1_1_0_0_0_0, //
            0b_0_0_0_0_0_0_0_0_0_0_0_0_0_0_0_0_0, //
        ],
        width: 17,
        color: Color::LightBlue,
    },
    Image {
        pixels: &[
            0b_0_0_0_0_0_0_0_0_0_1_1_1_0_0_0_0_0, //
            0b_0_0_0_0_0_0_0_0_0_1_1_1_1_0_0_0_0, //
            0b_0_0_0_0_0_0_0_0_0_1_1_1_1_1_0_0_0, //
            0b_0_0_0_0_0_0_0_0_0_1_1_0_0_1_0_0_0, //
            0b_0_0_1_1_1_1_1_1_1_1_1_0_0_1_1_1_1, //
            0b_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_0, //
            0b_0_0_0_0_1_1_1_1_1_1_1_1_1_1_0_0_0, //
            0b_0_0_1_1_1_1_1_1_1_1_1_1_1_1_0_0_0, //
            0b_0_0_0_0_0_1_1_1_1_1_1_1_1_0_0_0_0, //
            0b_0_0_0_0_0_1_1_1_0_0_0_0_0_0_0_0_0, //
            0b_0_0_0_0_0_0_1_1_0_0_0_0_0_0_0_0_0, //
        ],
        width: 17,
        color: Color::LightBlue,
    },
    Image {
        pixels: &[
            0b_0_0_0_0_0_0_0_0_0_1_1_1_0_0_0_0_0, //
            0b_0_0_0_0_0_0_0_0_0_1_1_1_1_0_0_0_0, //
            0b_0_0_0_0_0_0_0_0_0_1_1_1_1_1_0_0_0, //
            0b_0_0_0_0_0_0_0_0_0_1_1_0_0_1_0_0_0, //
            0b_0_0_1_1_1_1_1_1_1_1_1_0_0_1_1_1_1, //
            0b_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_1_0, //
            0b_0_0_0_0_1_1_1_1_1_1_1_1_1_1_0_0_0, //
            0b_0_0_1_1_1_1_1_1_1_1_1_1_1_1_0_0_0, //
            0b_0_0_0_0_0_1_1_1_1_1_1_1_1_0_0_0_0, //
            0b_0_0_0_0_0_0_0_0_0_0_0_0_0_0_0_0_0, //
        ],
        width: 17,
        color: Color::LightBlue,
    },
]);

#[allow(unused)]
pub const TREX: Image = Image {
    pixels: &[
        0b_0_1_1_1_1_1_1_0_0_0_0_0_0_0, //
        0b_1_1_1_1_1_0_1_1_0_0_0_0_0_0, //
        0b_1_1_1_1_1_1_1_1_0_0_0_0_0_0, //
        0b_1_1_1_1_1_1_1_1_0_0_0_0_0_0, //
        0b_0_0_0_0_1_1_1_1_0_0_0_0_0_0, //
        0b_0_0_1_1_1_1_1_1_0_0_0_0_0_0, //
        0b_0_0_0_0_0_1_1_1_1_0_0_0_0_1, //
        0b_0_0_0_1_1_1_1_1_1_1_0_0_1_1, //
        0b_0_0_0_1_0_1_1_1_1_1_1_1_1_1, //
        0b_0_0_0_0_0_1_1_1_1_1_1_1_1_1, //
        0b_0_0_0_0_0_1_1_1_1_1_1_1_1_0, //
        0b_0_0_0_0_0_0_1_1_1_1_1_1_0_0, //
        0b_0_0_0_0_0_0_0_1_1_1_1_0_0_0, //
        0b_0_0_0_0_0_0_0_1_0_1_1_0_0_0, //
        0b_0_0_0_0_0_0_0_1_0_0_1_0_0_0, //
        0b_0_0_0_0_0_0_1_1_0_1_1_0_0_0, //
    ],
    width: 16,
    color: Color::Red,
};

pub const CACTUSES: [Image; 3] = [
    Image {
        pixels: &[
            0b_0_0_0_0_0_1_0_0_0_0_0, //
            0b_0_0_0_0_1_1_1_0_0_0_0, //
            0b_0_1_0_0_1_1_1_0_0_0_0, //
            0b_1_1_0_0_1_1_1_0_0_1_0, //
            0b_1_1_0_0_1_1_1_0_0_1_1, //
            0b_1_1_0_0_1_1_1_0_0_1_1, //
            0b_1_1_0_0_1_1_1_0_0_1_1, //
            0b_1_1_0_0_1_1_1_0_0_1_1, //
            0b_1_1_1_1_1_1_1_1_1_1_1, //
            0b_0_1_1_1_1_1_1_1_1_1_0, //
            0b_0_0_1_1_1_1_1_1_1_0_0, //
            0b_0_0_0_0_1_1_1_0_0_0_0, //
            0b_0_0_0_0_1_1_1_0_0_0_0, //
            0b_0_0_0_0_1_1_1_0_0_0_0, //
            0b_0_0_0_0_1_1_1_0_0_0_0, //
        ],
        width: 11,
        color: Color::Green,
    },
    Image {
        pixels: &[
            0b_0_0_0_0_0_1_0_0_0_0_0, //
            0b_0_0_0_0_1_1_1_0_0_0_0, //
            0b_0_1_0_0_1_1_1_0_0_0_0, //
            0b_1_1_0_0_1_1_1_0_0_1_0, //
            0b_1_1_0_0_1_1_1_0_0_1_1, //
            0b_1_1_0_0_1_1_1_0_0_1_1, //
            0b_1_1_0_0_1_1_1_0_0_1_1, //
            0b_1_1_0_0_1_1_1_0_0_1_1, //
            0b_1_1_1_1_1_1_1_1_1_1_1, //
            0b_0_1_1_1_1_1_1_1_1_1_0, //
            0b_0_0_1_1_1_1_1_1_1_0_0, //
            0b_0_0_0_0_1_1_1_0_0_0_0, //
            0b_0_0_0_0_1_1_1_0_0_0_0, //
            0b_0_0_0_0_1_1_1_0_0_0_0, //
            0b_0_0_0_0_1_1_1_0_0_0_0, //
        ],
        width: 11,
        color: Color::Green,
    },
    Image {
        pixels: &[
            0b_0_0_0_0_0_1_0_0_0_0_0, //
            0b_0_0_0_0_1_1_1_0_0_0_0, //
            0b_0_1_0_0_1_1_1_0_0_0_0, //
            0b_1_1_0_0_1_1_1_0_0_1_0, //
            0b_1_1_0_0_1_1_1_0_0_1_1, //
            0b_1_1_0_0_1_1_1_0_0_1_1, //
            0b_1_1_0_0_1_1_1_0_0_1_1, //
            0b_1_1_0_0_1_1_1_0_0_1_1, //
            0b_1_1_1_1_1_1_1_1_1_1_1, //
            0b_0_1_1_1_1_1_1_1_1_1_0, //
            0b_0_0_1_1_1_1_1_1_1_0_0, //
            0b_0_0_0_0_1_1_1_0_0_0_0, //
            0b_0_0_0_0_1_1_1_0_0_0_0, //
            0b_0_0_0_0_1_1_1_0_0_0_0, //
            0b_0_0_0_0_1_1_1_0_0_0_0, //
        ],
        width: 11,
        color: Color::Green,
    },
];
