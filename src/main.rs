use ggez::{Context, ContextBuilder, GameResult};
use ggez::conf::{FullscreenType, WindowMode, WindowSetup};
use ggez::graphics::{self, Color, Mesh, Text};
use ggez::event::{self, EventHandler};
use ggez::timer;

const TARGET_FPS: u32 = 60;
const DIMENSIONS: glam::Vec2 = glam::const_vec2!([76.0, 45.0]);
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
    top_left: glam::Vec2,
    score: u32,
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
            (ratio_y, glam::Vec2::new((width - ratio_y * DIMENSIONS.x) / 2.0, 0.0))
        } else {
            (ratio_x, glam::Vec2::new(0.0, 0.0))
        };
        println!("{}, {}", top_left.x, top_left.y);
        Game {
            dim,
            top_left,
            score: 0,
        }
    }
}

impl EventHandler for Game {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        while timer::check_update_time(ctx, TARGET_FPS) {

        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, Color::from_rgb_u32(0x232528));

        let grid = &Mesh::new_rectangle(
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
            grid,
            graphics::DrawParam::default(),
        )?;

        graphics::queue_text(
            ctx,
            &Text::new(self.score.to_string()).set_font(
                graphics::Font::default(),
                graphics::PxScale::from(self.dim * 2.0),
            ),
            glam::Vec2::new(
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
