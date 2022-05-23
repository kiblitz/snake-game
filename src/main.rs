use ggez::{Context, ContextBuilder, GameResult};
use ggez::conf::{FullscreenType, WindowMode, WindowSetup};
use ggez::graphics::{self, Color, Mesh, Text};
use ggez::input::keyboard;
use ggez::event::{self, EventHandler, KeyCode};
use ggez::timer;

use glam::{IVec2, Vec2};

use rand::Rng;

use std::collections::{LinkedList, HashMap, HashSet};
use std::vec::Vec;

const TARGET_FPS: u32 = 60;
const STARTING_FRAME_DELAY: u8 = 12;
const FRAME_DELAY_DECAY: f32 = 0.9;
const FRAME_DELAY_INC: f32 = 1.4;

const BB_GEN_FRAMES: u32 = 640;

const DIMENSIONS: IVec2 = glam::const_ivec2!([76, 45]);
const SCORE_STRIP: i32 = 4;

const APPLE_CIRCLE_TOLERANCE: f32 = 5.0;
const BB_CIRCLE_TOLERANCE: f32 = 1.0;

const OFF_LIMITS_RANGE: i32 = 3;

fn main() {
    let (tmp_ctx, _) = ContextBuilder::new("", "")
        .window_mode(WindowMode::default()
            .fullscreen_type(FullscreenType::Desktop)
        )
        .build()
        .expect("failed to create context");
    let (width, height) = graphics::size(&tmp_ctx);

    let (mut ctx, event_loop) =
        ContextBuilder::new("Snake_Game", "kiblitz")
            .window_setup(WindowSetup::default()
                .title("Snake_Game")
            )
            .window_mode(WindowMode::default()
                .dimensions(width, height)
                .fullscreen_type(FullscreenType::True)
                .maximized(true)
                .borderless(true)
            )
            .build()
            .expect("failed to create context");
    let my_game = Game::new(&mut ctx);
    event::run(ctx, event_loop, my_game);
}

struct Game {
    geo_config: GeoConfig,
    score: u32,
    live: bool,
    snake: Snake,
    apple: IVec2,
    blueberry: Option<IVec2>,
    buffered_direction: Option<Direction>,
    direction: Option<Direction>,
    frame_data: FrameData,
    open_squares: Vec<IVec2>,
    rng: rand::rngs::ThreadRng,
}

struct GeoConfig {
    dim: f32,
    top_left: Vec2,
}

struct Snake {
    body: LinkedList<IVec2>,
    set: HashSet<IVec2>,
    occupied: HashMap<IVec2, u8>,
}

struct FrameData {
    frame: u8,
    frame_delay: f32,
    bb_frame: u32,
}

#[derive(Copy, Clone, PartialEq)]
enum Direction {
    UP, DOWN, LEFT, RIGHT,
}

impl Snake {
    fn new(start_pos: IVec2) -> Self {
        let mut snake = Self {
            body: LinkedList::new(),
            set: HashSet::new(),
            occupied: HashMap::new(),
        };
        snake.grow(start_pos);
        snake
    }

    fn head(&self) -> IVec2 {
        *self.body.back().unwrap()
    }

    fn iter(&self) -> std::collections::linked_list::Iter<'_, IVec2> {
        self.body.iter()
    }

    fn grow(&mut self, pos: IVec2) -> bool {
        if self.set.contains(&pos) && *self.body.front().unwrap() != pos {
            return false;
        }
        self.body.push_back(pos);
        self.set.insert(pos);
        for x in -OFF_LIMITS_RANGE..=OFF_LIMITS_RANGE {
            for y in -OFF_LIMITS_RANGE..=OFF_LIMITS_RANGE {
                let delta = glam::const_ivec2!([x, y]);
                let new_pos = pos + delta;
                let new_count = match self.occupied.get(&new_pos) {
                    Some(count) => count + 1,
                    _ => 1,
                };
                self.occupied.insert(new_pos, new_count);
            }
        }
        true
    }

    fn shrink(&mut self) {
        let elem = self.body.pop_front().unwrap();
        self.set.remove(&elem);
        for x in -OFF_LIMITS_RANGE..=OFF_LIMITS_RANGE {
            for y in -OFF_LIMITS_RANGE..=OFF_LIMITS_RANGE {
                let delta = glam::const_ivec2!([x, y]);
                let new_pos = elem + delta;
                let count = *self.occupied.get(&new_pos).unwrap();
                if count == 1 {
                    self.occupied.remove(&new_pos);
                } else {
                    self.occupied.insert(new_pos, count - 1);
                }
            }
        }
    }

    fn is_off_limits(&self, pos: IVec2) -> bool {
        self.occupied.contains_key(&pos)
    }
}

impl FrameData {
    fn new() -> Self {
        Self {
            frame: 0,
            frame_delay: STARTING_FRAME_DELAY as f32,
            bb_frame: 0,
        }
    }

    fn next_frame(&mut self) {
        self.frame += 1;
    }

    fn time_to_update(&mut self) -> bool {
        if self.frame >= self.frame_delay.ceil() as u8 {
            self.frame = 0;
            true
        } else {
            false
        }
    }

    fn next_bb_frame(&mut self) {
        self.bb_frame += 1;
    }

    fn time_to_gen_bb(&mut self) -> bool {
        if self.bb_frame >= BB_GEN_FRAMES {
            self.bb_frame = 0;
            true
        } else {
            false
        }
    }
}

fn invalid_coord() -> IVec2 {
    glam::const_ivec2!([-1, -1])
}

impl Game {
    pub fn new(ctx: &mut Context) -> Game {
        let (width, height) = graphics::size(&ctx);
        let total_dim_y = DIMENSIONS.y as f32 + SCORE_STRIP as f32;
        let (ratio_x, ratio_y) = (
            width as f32 / DIMENSIONS.x as f32,
            height as f32 / total_dim_y,
        );
        let (dim, top_left) = if ratio_y < ratio_x {
            (
                ratio_y,
                Vec2::new((width - ratio_y * DIMENSIONS.x as f32) / 2.0, 0.0),
            )
        } else {
            (ratio_x, Vec2::new(0.0, 0.0))
        };
        let (mut x, mut y) = (0, 0);
        let mut open_squares = Vec::new();
        open_squares.resize_with(
            DIMENSIONS.x as usize * DIMENSIONS.y as usize,
            || {
                let sq = IVec2::new(x, y);
                x += 1;
                if x >= DIMENSIONS.x {
                    x = 0;
                    y += 1;
                }
                sq
            }
        );
        let mut game = Game {
            geo_config: GeoConfig {
                dim,
                top_left,
            },
            score: 0,
            live: true,
            snake: Snake::new(IVec2::new(
                DIMENSIONS.x as i32 / 2,
                DIMENSIONS.y as i32 / 2,
            )),
            apple: invalid_coord(),
            blueberry: None,
            buffered_direction: None,
            direction: None,
            frame_data: FrameData::new(),
            open_squares,
            rng: rand::thread_rng(),
        };
        game.apple = game.gen_open_square();
        game
    }

    fn gen_open_square(&mut self) -> IVec2 {
        let index = self.rng.gen_range(0..self.open_squares.len());
        let sq = self.open_squares[index];
        if self.snake.is_off_limits(sq) ||
            self.apple == sq ||
            (self.blueberry != None && self.blueberry.unwrap() == sq)
        {
            self.gen_open_square()
        } else {
            sq
        }
    }
}

fn right(ctx: &mut Context) -> bool {
    keyboard::is_key_pressed(ctx, KeyCode::Right)
        || keyboard::is_key_pressed(ctx, KeyCode::D)
}
fn left(ctx: &mut Context) -> bool {
    keyboard::is_key_pressed(ctx, KeyCode::Left)
        || keyboard::is_key_pressed(ctx, KeyCode::A)
}
fn up(ctx: &mut Context) -> bool {
    keyboard::is_key_pressed(ctx, KeyCode::Up)
        || keyboard::is_key_pressed(ctx, KeyCode::W)
}
fn down(ctx: &mut Context) -> bool {
    keyboard::is_key_pressed(ctx, KeyCode::Down)
        || keyboard::is_key_pressed(ctx, KeyCode::S)
}

impl EventHandler for Game {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        if !self.live {
            if keyboard::is_key_pressed(ctx, KeyCode::Space) {
                *self = Game::new(ctx);
            }
            return Ok(());
        }

        // Set snake direction
        let vert_states = [None, Some(Direction::UP), Some(Direction::DOWN)];
        let hor_states = [None, Some(Direction::LEFT), Some(Direction::RIGHT)];
        if left(ctx) && !right(ctx) && vert_states.contains(&self.direction) {
            self.buffered_direction = Some(Direction::LEFT);
        } else if right(ctx) && !left(ctx) &&
            vert_states.contains(&self.direction) {
            self.buffered_direction = Some(Direction::RIGHT);
        } else if up(ctx) && !down(ctx) &&
            hor_states.contains(&self.direction) {
            self.buffered_direction = Some(Direction::UP);
        } else if down(ctx) && !up(ctx) &&
            hor_states.contains(&self.direction) {
            self.buffered_direction = Some(Direction::DOWN);
        }
        if self.buffered_direction == None {
            return Ok(());
        }

        // Update game state
        while timer::check_update_time(ctx, TARGET_FPS) {
            self.frame_data.next_frame();
            if !self.frame_data.time_to_update() {
                return Ok(());
            }

            // Update direction
            self.direction = self.buffered_direction;
            let (dx, dy) = match self.direction {
                Some(Direction::UP) => (0, -1),
                Some(Direction::DOWN) => (0, 1),
                Some(Direction::LEFT) => (-1, 0),
                Some(Direction::RIGHT) => (1, 0),
                _ => panic!("unexpected snake direction"),
            };

            // Move snake
            let head = self.snake.head();
            let (new_head_x, new_head_y) = (head.x + dx, head.y + dy);
            if new_head_x < 0 || new_head_x >= DIMENSIONS.x ||
                new_head_y < 0 || new_head_y >= DIMENSIONS.y {
                self.live = false;
                return Ok(());
            }
            let new_head = IVec2::new(new_head_x, new_head_y);
            if !self.snake.grow(new_head) {
                self.live = false;
                return Ok(());
            }

            // Apple collection
            if new_head == self.apple {
                self.apple = self.gen_open_square();
                self.score += 1;
                self.frame_data.frame_delay *= FRAME_DELAY_DECAY;
            } else {
                self.snake.shrink();
            }

            // Blueberry collection
            if let Some(blueberry) = self.blueberry {
                if new_head == blueberry {
                    self.blueberry = None;
                    self.score += 1;
                    self.frame_data.frame_delay += FRAME_DELAY_INC;
                }
            } else {
                self.frame_data.next_bb_frame();
                if self.frame_data.time_to_gen_bb() {
                    self.blueberry = Some(self.gen_open_square());
                }
            }
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, Color::from_rgb_u32(0x232528));
        let dim = self.geo_config.dim;
        let top_left = self.geo_config.top_left;
        let radius = dim / 2.0;
        let scale_vec = glam::const_vec2!([dim, dim]);
        let center_dim = glam::const_vec2!([radius, radius]);

        // Draw play area
        let area = &Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::Fill(graphics::FillOptions::default()),
            graphics::Rect::new(
                top_left.x,
                top_left.y,
                dim * DIMENSIONS.x as f32,
                dim * DIMENSIONS.y as f32,
            ),
            Color::BLACK,
        ).unwrap();
        graphics::draw(
            ctx,
            area,
            graphics::DrawParam::default(),
        )?;

        // Draw the snake
        for pos in self.snake.iter() {
            let px_pos =
                (*pos).as_vec2() * scale_vec + top_left;
            let body_graphic = &Mesh::new_polygon(
                ctx,
                graphics::DrawMode::Fill(graphics::FillOptions::default()),
                &[
                    px_pos + Vec2::new(radius, 0.0),
                    px_pos + Vec2::new(dim, radius),
                    px_pos + Vec2::new(radius, dim),
                    px_pos + Vec2::new(0.0, radius),
                ],
                Color::GREEN,
            ).unwrap();
            graphics::draw(
                ctx,
                body_graphic,
                graphics::DrawParam::default(),
            )?;
        }

        // Draw foods
        let apple_graphic = &Mesh::new_circle(
            ctx,
            graphics::DrawMode::Fill(graphics::FillOptions::default()),
            self.apple.as_vec2() * scale_vec + center_dim + top_left,
            radius,
            APPLE_CIRCLE_TOLERANCE,
            Color::RED,
        ).unwrap();
        graphics::draw(
            ctx,
            apple_graphic,
            graphics::DrawParam::default(),
        )?;

        if let Some(blueberry) = self.blueberry {
            let blueberry_graphic = &Mesh::new_circle(
                ctx,
                graphics::DrawMode::Fill(graphics::FillOptions::default()),
                blueberry.as_vec2() * scale_vec + center_dim + top_left,
                radius,
                BB_CIRCLE_TOLERANCE,
                Color::from_rgb_u32(0x4287f5),
            ).unwrap();
            graphics::draw(
                ctx,
                blueberry_graphic,
                graphics::DrawParam::default(),
            )?;
        }

        // Draw score
        let text_size = dim * 2.0;
        graphics::queue_text(
            ctx,
            &Text::new(self.score.to_string()).set_font(
                graphics::Font::default(),
                graphics::PxScale::from(text_size),
            ),
            Vec2::new(
                top_left.x + text_size / 2.0,
                top_left.y + dim * DIMENSIONS.y as f32 + text_size / 2.0,
            ),
            Some(Color::WHITE),
        );

        // Game over screen
        let big_text_size = dim * 4.0;
        if !self.live {
            graphics::queue_text(
                ctx,
                &Text::new("GAME OVER").set_font(
                    graphics::Font::default(),
                    graphics::PxScale::from(big_text_size),
                ),
                Vec2::new(
                    top_left.x + dim * DIMENSIONS.x as f32 / 2.0
                        - big_text_size * 2.25,
                    top_left.y + dim * DIMENSIONS.y as f32 / 2.0 - dim * 2.0
                        - big_text_size / 2.0,
                ),
                Some(Color::WHITE),
            );
            graphics::queue_text(
                ctx,
                &Text::new("space to continue").set_font(
                    graphics::Font::default(),
                    graphics::PxScale::from(text_size),
                ),
                Vec2::new(
                    top_left.x + dim * DIMENSIONS.x as f32 / 2.0
                        - text_size * 4.25,
                    top_left.y + dim * DIMENSIONS.y as f32 / 2.0
                        - text_size / 2.0,
                ),
                Some(Color::WHITE),
            );
        }

        graphics::draw_queued_text(
            ctx,
            graphics::DrawParam::default(),
            None,
            graphics::FilterMode::Linear,
        )?;
        graphics::present(ctx)
    }
}
