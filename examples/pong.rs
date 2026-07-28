use engine::core::{Application, Game};
use engine::graphics::{Color, Frame, Transform};
use engine::hecs::World;
use engine::input::Input;
use glam::Vec2;
use winit::keyboard::KeyCode;

const WINDOW_WIDTH: f32 = 800.0;
const WINDOW_HEIGHT: f32 = 600.0;

const PADDLE_WIDTH: f32 = 12.0;
const PADDLE_HEIGHT: f32 = 80.0;
const PADDLE_MARGIN: f32 = 30.0;
const PADDLE_SPEED: f32 = 400.0;

const BALL_SIZE: f32 = 10.0;
const BALL_SPEED: f32 = 350.0;
const BALL_SPEEDUP: f32 = 1.05;
const BALL_MAX_SPEED: f32 = 700.0;

const WINNING_SCORE: u32 = 7;

// Components

#[derive(Clone, Copy, PartialEq)]
enum Side {
    Left,
    Right,
}

struct Paddle {
    side: Side,
}

struct Ball;

struct Velocity(Vec2);

// Systems

fn serve_velocity(total_points: u32) -> Velocity {
    // Serve toward the player who was just scored on.
    let direction = if total_points % 2 == 0 { 1.0 } else { -1.0 };
    Velocity(Vec2::new(direction * BALL_SPEED, BALL_SPEED * 0.3))
}

fn input_system(world: &mut World, input: &Input, dt: f32) {
    for (_, (paddle, transform)) in world.query_mut::<(&Paddle, &mut Transform)>() {
        let (up, down) = match paddle.side {
            Side::Left => (KeyCode::KeyW, KeyCode::KeyS),
            Side::Right => (KeyCode::ArrowUp, KeyCode::ArrowDown),
        };
        if input.keyboard.is_pressed(&up) {
            transform.position.y -= PADDLE_SPEED * dt;
        }
        if input.keyboard.is_pressed(&down) {
            transform.position.y += PADDLE_SPEED * dt;
        }
        transform.position.y = transform
            .position
            .y
            .clamp(PADDLE_HEIGHT / 2.0, WINDOW_HEIGHT - PADDLE_HEIGHT / 2.0);
    }
}

fn movement_system(world: &mut World, dt: f32) {
    for (_, (velocity, transform)) in world.query_mut::<(&Velocity, &mut Transform)>() {
        transform.position += velocity.0 * dt;
    }
}

fn overlaps(pos_a: Vec2, size_a: Vec2, pos_b: Vec2, size_b: Vec2) -> bool {
    let delta = (pos_a - pos_b).abs();
    let min = (size_a + size_b) / 2.0;
    delta.x < min.x && delta.y < min.y
}

fn bounce_system(world: &mut World) {
    let paddles: Vec<(Side, Vec2)> = world
        .query::<(&Paddle, &Transform)>()
        .iter()
        .map(|(_, (paddle, transform))| (paddle.side, transform.position))
        .collect();

    for (_, (_, transform, velocity)) in world.query_mut::<(&Ball, &mut Transform, &mut Velocity)>()
    {
        // Top / bottom walls.
        if transform.position.y - BALL_SIZE / 2.0 <= 0.0 && velocity.0.y < 0.0 {
            velocity.0.y = -velocity.0.y;
        }
        if transform.position.y + BALL_SIZE / 2.0 >= WINDOW_HEIGHT && velocity.0.y > 0.0 {
            velocity.0.y = -velocity.0.y;
        }

        // Paddle bounces. The steeper the hit offset, the steeper the return angle.
        for (side, paddle_pos) in &paddles {
            let heading_left = velocity.0.x < 0.0;
            if (*side == Side::Left) != heading_left {
                continue;
            }
            if !overlaps(
                transform.position,
                Vec2::splat(BALL_SIZE),
                *paddle_pos,
                Vec2::new(PADDLE_WIDTH, PADDLE_HEIGHT),
            ) {
                continue;
            }

            let offset = (transform.position.y - paddle_pos.y) / (PADDLE_HEIGHT / 2.0);
            let speed = (velocity.0.length() * BALL_SPEEDUP).min(BALL_MAX_SPEED);
            let direction = if *side == Side::Left { 1.0 } else { -1.0 };
            velocity.0 = Vec2::new(direction, offset).normalize() * speed;
        }
    }
}

fn score_system(world: &mut World, left_score: &mut u32, right_score: &mut u32) {
    for (_, (_, transform, velocity)) in world.query_mut::<(&Ball, &mut Transform, &mut Velocity)>()
    {
        let scored = if transform.position.x < 0.0 {
            *right_score += 1;
            true
        } else if transform.position.x > WINDOW_WIDTH {
            *left_score += 1;
            true
        } else {
            false
        };

        if scored {
            transform.position = Vec2::new(WINDOW_WIDTH / 2.0, WINDOW_HEIGHT / 2.0);
            *velocity = serve_velocity(*left_score + *right_score);
        }
    }

    if *left_score >= WINNING_SCORE || *right_score >= WINNING_SCORE {
        *left_score = 0;
        *right_score = 0;
    }
}

fn render_system(world: &World, frame: &mut Frame) {
    for (_, (_, transform)) in world.query::<(&Paddle, &Transform)>().iter() {
        frame.draw_rectangle(PADDLE_WIDTH, PADDLE_HEIGHT, Color::WHITE, *transform);
    }
    for (_, (_, transform)) in world.query::<(&Ball, &Transform)>().iter() {
        frame.draw_quad(BALL_SIZE, Color::RED, *transform);
    }
}

// Game

struct Pong {
    world: World,
    left_score: u32,
    right_score: u32,
}

impl Pong {
    fn new() -> Self {
        let mut world = World::new();
        world.spawn((
            Paddle { side: Side::Left },
            Transform::new(Vec2::new(PADDLE_MARGIN, WINDOW_HEIGHT / 2.0)),
        ));
        world.spawn((
            Paddle { side: Side::Right },
            Transform::new(Vec2::new(WINDOW_WIDTH - PADDLE_MARGIN, WINDOW_HEIGHT / 2.0)),
        ));
        world.spawn((
            Ball,
            Transform::new(Vec2::new(WINDOW_WIDTH / 2.0, WINDOW_HEIGHT / 2.0)),
            serve_velocity(0),
        ));

        Self {
            world,
            left_score: 0,
            right_score: 0,
        }
    }
}

impl Game for Pong {
    fn update(&mut self, input: &Input, dt: f64) {
        let dt = dt as f32;
        input_system(&mut self.world, input, dt);
        movement_system(&mut self.world, dt);
        bounce_system(&mut self.world);
        score_system(&mut self.world, &mut self.left_score, &mut self.right_score);
    }

    fn render(&self, frame: &mut Frame) {
        // Score pips.
        for i in 0..self.left_score {
            let pos = Vec2::new(WINDOW_WIDTH / 2.0 - 20.0 - i as f32 * 16.0, 20.0);
            frame.draw_quad(8.0, Color::WHITE, Transform::new(pos));
        }
        for i in 0..self.right_score {
            let pos = Vec2::new(WINDOW_WIDTH / 2.0 + 20.0 + i as f32 * 16.0, 20.0);
            frame.draw_quad(8.0, Color::WHITE, Transform::new(pos));
        }

        render_system(&self.world, frame);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let app = Application::new("Pong", 800, 600, Pong::new());
    app.run()
}
