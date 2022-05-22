use ggez::{Context, ContextBuilder, GameResult};
use ggez::conf::{FullscreenType, WindowMode, WindowSetup};
use ggez::graphics::{self, Color, Mesh, Text};
use ggez::input::keyboard;
use ggez::event::{self, EventHandler, KeyCode};
use ggez::timer;

use glam::{IVec2, Vec2};

use std::collections::LinkedList;

const TARGET_FPS: u32 = 60;
const STARTING_FRAME_DELAY: u8 = 15;
const DIMENSIONS: Vec2 = glam::const_vec2!([76.0, 45.0]);
const SCORE_STRIP: f32 = 4.0;

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
    dim: f32,
    top_left: Vec2,
    score: u32,
    snake: LinkedList<IVec2>,
    direction: Option<Direction>,
    frame: u8,
    frame_delay: u8,
}

#[derive(Copy, Clone, PartialEq)]
enum Direction {
    UP, DOWN, LEFT, RIGHT,
}

impl Game {
    pub fn new(ctx: &mut Context) -> Game {
        let (width, height) = graphics::size(&ctx);
        let total_dim_y = DIMENSIONS.y + SCORE_STRIP;
        let (ratio_x, ratio_y) = (
            width as f32 / DIMENSIONS.x,
            height as f32 / total_dim_y,
        );
        let (dim, top_left) = if ratio_y < ratio_x {
            (ratio_y, Vec2::new((width - ratio_y * DIMENSIONS.x) / 2.0, 0.0))
        } else {
            (ratio_x, Vec2::new(0.0, 0.0))
        };
        Game {
            dim,
            top_left,
            score: 0,
            snake: LinkedList::from([IVec2::new(
                (DIMENSIONS.x as u32 / 2) as i32,
                (DIMENSIONS.y  as u32 / 2) as i32,
            )]),
            direction: None,
            frame: 0,
            frame_delay: STARTING_FRAME_DELAY,
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
        // Set snake direction
        let vert_states = [None, Some(Direction::UP), Some(Direction::DOWN)];
        let hor_states = [None, Some(Direction::LEFT), Some(Direction::RIGHT)];
        if left(ctx) && !right(ctx) && vert_states.contains(&self.direction) {
            self.direction = Some(Direction::LEFT);
        } else if right(ctx) && !left(ctx) &&
            vert_states.contains(&self.direction) {
            self.direction = Some(Direction::RIGHT);
        } else if up(ctx) && !down(ctx) &&
            hor_states.contains(&self.direction) {
            self.direction = Some(Direction::UP);
        } else if down(ctx) && !up(ctx) &&
            hor_states.contains(&self.direction) {
            self.direction = Some(Direction::DOWN);
        }
        if self.direction == None {
            return Ok(());
        }

        while timer::check_update_time(ctx, TARGET_FPS) {
            self.frame += 1;
            if self.frame < self.frame_delay {
                return Ok(());
            }

            self.frame = 0;
            let (dx, dy) = match self.direction {
                Some(Direction::UP) => (0, -1),
                Some(Direction::DOWN) => (0, 1),
                Some(Direction::LEFT) => (-1, 0),
                Some(Direction::RIGHT) => (1, 0),
                _ => panic!("unexpected snake direction"),
            };
            let head = self.snake.back().unwrap();
            let (head_x, head_y) = (head.x, head.y);
            self.snake.push_back(IVec2::new(head_x + dx, head_y + dy));
            self.snake.pop_front();
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, Color::from_rgb_u32(0x232528));

        // Draw play area
        let area = &Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::Fill(graphics::FillOptions::default()),
            graphics::Rect::new(
                self.top_left.x,
                self.top_left.y,
                self.dim * DIMENSIONS.x,
                self.dim * DIMENSIONS.y,
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
                (*pos).as_vec2() * Vec2::new(self.dim, self.dim) + self.top_left;
            let body = &Mesh::new_polygon(
                ctx,
                graphics::DrawMode::Fill(graphics::FillOptions::default()),
                &[
                    px_pos + Vec2::new(self.dim / 2.0, 0.0),
                    px_pos + Vec2::new(self.dim, self.dim / 2.0),
                    px_pos + Vec2::new(self.dim / 2.0, self.dim),
                    px_pos + Vec2::new(0.0, self.dim / 2.0),
                ],
                Color::GREEN,
            ).unwrap();
            graphics::draw(
                ctx,
                body,
                graphics::DrawParam::default(),
            )?;
        }

        // Draw score
        graphics::queue_text(
            ctx,
            &Text::new(self.score.to_string()).set_font(
                graphics::Font::default(),
                graphics::PxScale::from(self.dim * 2.0),
            ),
            Vec2::new(
                self.top_left.x + self.dim,
                self.top_left.y + self.dim * DIMENSIONS.y + self.dim,
            ),
            Some(Color::WHITE),
        );
        graphics::draw_queued_text(
            ctx,
            graphics::DrawParam::default(),
            None,
            graphics::FilterMode::Linear,
        )?;

        graphics::present(ctx)
    }
}
