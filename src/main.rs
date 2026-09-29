use macroquad::prelude::*;

struct Player {
    pos: Vec2,
    rad: f32,
    speed: f32,
    health: i32,
}

enum State {
    Landing,
    Run,
    GameOver,
    Win,
}

#[macroquad::main("Undertale")]
async fn main() {
    let mut state = State::Run;

    // player conf
    let mut p_x: f32 = 0.;
    let mut p_y: f32 = 0.;
    let p_speed: f32 = 4.;
    let mut p = Player {
        pos: Vec2::ZERO,
        rad: 20.,
        speed: 10.,
        health: 5,
    };

    // bound conf
    let b_x = 400.;
    let b_y = 200.;

    // timer
    let mut interval = 100;
    let mut timer = interval;
    let mut flag = false;

    loop {
        // screen
        let cw: f32 = screen_width() / 2.;
        let ch: f32 = screen_height() / 2.;

        // bird.pos conf
        p.pos.x = cw + p_x;
        p.pos.y = ch + p_y;

        // bound
        draw_rectangle_lines(
            cw - b_x,
            ch - b_y,
            2. * b_x,
            2. * b_y,
            10.,
            WHITE,
        );

        match &state {
            State::Run => {
                // if p_offset + p.rad >= 500. || p_offset - p.rad <= -500. {
                // }

                // timer
                if flag {
                    draw_rectangle(cw - 100., ch - 100., 200., 200., RED);
                }

                if timer > 0 {
                    timer -= 1;
                } else {
                    flag = !flag;
                    timer = interval;
                }


                /*
*                * Player
*                */
                // player movement
                if is_key_down(KeyCode::Up) {
                    p_y -= p.speed;
                }
                if is_key_down(KeyCode::Down) {
                    p_y += p.speed;
                }
                if is_key_down(KeyCode::Left) {
                    p_x -= p.speed;
                }
                if is_key_down(KeyCode::Right) {
                    p_x += p.speed;
                }

                // player bound
                if p_x + p.rad >= b_x {
                    p_x -= p_x + p.rad - b_x;
                }
                if p_x - p.rad <= -b_x {
                    p_x += -b_x - (p_x - p.rad);
                }
                if p_y + p.rad >= b_y {
                    p_y -= p_y + p.rad - b_y;
                }
                if p_y - p.rad <= -b_y {
                    p_y += -b_y - (p_y - p.rad);
                }

                draw_circle(p.pos.x, p.pos.y, p.rad, WHITE);
            },
            _ => {}
        }


        next_frame().await
    }
}
