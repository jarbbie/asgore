use asgore::Game;
use macroquad::prelude::{get_frame_time, next_frame};

#[macroquad::main("Asgore")]
async fn main() {
    let mut game = Game::new().await;

    loop {
        game.update(get_frame_time());
        game.draw();
        next_frame().await;
    }
}
