use macroquad::{
    prelude::*,
    audio,
    texture,
};

struct Player {
    pos: Vec2,
    rad: f32,
    speed: f32,
    hp: i32,
    max_hp: i32,
}

enum State {
    Landing,
    PromptFade,
    Starting,
    Run,
    GameOver,
    Win,
}

#[macroquad::main("Asgore")]
async fn main() {
    let mut state = State::Landing;

    // player conf
    let mut p_x: f32 = 0.;
    let mut p_y: f32 = 0.;
    let p_speed: f32 = 2.8;
    let mut p = Player {
        pos: Vec2::ZERO,
        rad: 20.,
        speed: 8.,
        hp: 5,
        max_hp: 5,
    };
    let p_texture: Texture2D = load_texture("./assets/image/player.png").await.unwrap();

    // bound conf
    let b_x = 300.;
    let b_y = 200.;
    let mut current_b_x = b_x;

    // timer
    let mut interval = 100;
    let mut timer = interval;
    let mut flag = false;
    let mut transition_elapsed = 0.0;

    // scence
    let opp_texture: Texture2D = load_texture("./assets/image/asgore.png").await.unwrap();
    let opp_mb = 50.;

    // animation
    let prompt_fade_seconds = 0.7;
    let border_expand_seconds = 1.2;
    
    // audio
    let aud_asgore_theme = audio::load_sound("./assets/audio/asgore_theme.wav").await.unwrap();
    let aud_savepoint = audio::load_sound("./assets/audio/savepoint.wav").await.unwrap();
    let aud_damage_taken = audio::load_sound("./assets/audio/damage_taken.wav").await.unwrap();
    let aud_asgore_driving = audio::load_sound("./assets/audio/driving-in-my-car.wav").await.unwrap();

    // initial
    audio::play_sound(&aud_asgore_theme,
        audio::PlaySoundParams {
            looped: true,
            volume: 1.0,
        },
    );

    loop {
        // screen
        let cw: f32 = screen_width() / 2.;
        let ch: f32 = screen_height() / 2.;

        // p.pos conf
        p.pos.x = cw + p_x;
        p.pos.y = ch + p_y;

        // frame time
        let delta = get_frame_time();

        match &state {
            State::Landing => {
                draw_text("press SPACE to start", cw - 220.0, ch, 50.0, WHITE);

                if is_key_pressed(KeyCode::Space) {
                    audio::play_sound_once(&aud_savepoint);
                    p.hp = p.max_hp;
                    transition_elapsed = 0.0;
                    state = State::PromptFade;
                }
            },
            State::PromptFade => {
                transition_elapsed += delta;
                let progress = (transition_elapsed / prompt_fade_seconds).clamp(0.0, 1.0);
                let alpha = 1.0 - progress;

                draw_text("press SPACE to start", cw - 220.0, ch, 50.0, Color::new(1.0, 1.0, 1.0, alpha));

                if progress >= 1.0 {
                    audio::stop_sound(&aud_asgore_theme);
                    transition_elapsed = 0.0;
                    current_b_x = 0.0;
                    state = State::Starting;
                }
            },
            State::Starting => {
                transition_elapsed += delta;
                let progress = (transition_elapsed / border_expand_seconds).clamp(0.0, 1.0);

                current_b_x = b_x * progress;

                draw_rectangle_lines(
                    cw - current_b_x,
                    ch - b_y,
                    2.0 * current_b_x,
                    2.0 * b_y,
                    10.0,
                    WHITE,
                );

                if progress >= 1.0 {
                    current_b_x = b_x;

                    audio::play_sound(&aud_asgore_driving,
                        audio::PlaySoundParams {
                            looped: true,
                            volume: 0.5,
                        },
                    );

                    state = State::Run;
                }
            },
            State::Run => {
                // opp
                draw_texture_ex(
                    &opp_texture,
                    cw - (135.),
                    ch - b_y - 195. - opp_mb,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(270., 195.)),
                        ..Default::default()
                    },
                );

                // bound
                draw_rectangle_lines(
                    cw - b_x,
                    ch - b_y,
                    2. * b_x,
                    2. * b_y,
                    10.,
                    WHITE,
                );

                // damage box
                if flag {
                    draw_rectangle(cw - 100., ch - 100., 200., 200., YELLOW);
                }

                if timer > 0 {
                    timer -= 1;
                }
                else {
                    if flag {
                        let closest_x = p_x.clamp(-100., 100.);
                        let closest_y = p_y.clamp(-100., 100.);
                        let dx = p_x - closest_x;
                        let dy = p_y - closest_y;

                        // Circle-rectangle collision: damage once when the red phase ends.
                        if dx * dx + dy * dy <= p.rad * p.rad {
                            audio::play_sound_once(&aud_damage_taken);
                            p.hp -= 1;
                        }
                    }
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

                let health_bar = format!("HP: {} / {}", p.hp.to_string(), p.max_hp.to_string());
                draw_text(health_bar, cw - b_x + 30., ch + b_y + 50., 50., WHITE);
                draw_texture_ex(
                    &p_texture,
                    p.pos.x - p.rad * 0.75,
                    p.pos.y - p.rad * 0.75,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(p.rad * 1.5, p.rad * 1.5)),
                        ..Default::default()
                    },
                );

                if p.hp == 0 {
                    audio::stop_sound(&aud_asgore_driving);
                    state = State::GameOver;
                }
            },
            State::GameOver => {
                draw_text("GameOver: press SPACE to replay", cw - 340.0, ch, 50.0, WHITE);
                if is_key_pressed(KeyCode::Space) {
                    state = State::Landing;
                }
            },
            _ => {}
        }

        next_frame().await
    }
}
