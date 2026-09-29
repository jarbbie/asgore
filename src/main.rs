use macroquad::prelude::*;

struct Bird {
    pos: Vec2,
    rad: f32,
}

enum State {
    Landing,
    Run,
    GameOver,
    Win,
}


#[macroquad::main("Galaxian")]
async fn main() {
    let mut state = State::Run;

    let mut bird_offset: f32 = 0.;
    let bird_speed: f32 = 4.;
    let mut bird = Bird {
        pos: Vec2::ZERO,
        rad: 30.,
    };

    loop {
        // screen
        let cw: f32 = screen_width() / 2.;
        let ch: f32 = screen_height() / 2.;

        // bird conf
        bird.pos.x = cw - 300.;
        bird.pos.y = ch + bird_offset;

        // bound
        draw_rectangle_lines(
            cw - 500.,
            ch - 500.,
            1000.,
            1000.,
            10.,
            WHITE,
        );

        match &state {
            State::Run => {
                if bird_offset + bird.rad >= 500. || bird_offset - bird.rad <= -500. {
                    state = State::GameOver;
                    continue;
                }

                // bird
                if is_key_down(KeyCode::Space) {
                    bird_offset -= bird_speed;
                } else {
                    bird_offset += bird_speed;
                }

                draw_circle(bird.pos.x, bird.pos.y, bird.rad, WHITE);
            },
            _ => {}
        }


        next_frame().await
    }
}
