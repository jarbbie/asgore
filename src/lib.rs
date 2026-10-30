use macroquad::{
    audio::{self, PlaySoundParams, Sound},
    prelude::*,
};

struct GameConfig {
    arena_half_width: f32,
    arena_half_height: f32,
    border_thickness: f32,
    wall_padding: f32,
    player_radius: f32,
    player_speed: f32,
    player_sprite_size: f32,
    max_player_hp: i32,
    prompt_fade_seconds: f32,
    border_expand_seconds: f32,
    hazard_phase_seconds: f32,
    hazard_half_size: f32,
    opponent_size: Vec2,
    opponent_gap: f32,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            arena_half_width: 300.0,
            arena_half_height: 200.0,
            border_thickness: 10.0,
            wall_padding: 3.0,
            player_radius: 20.0,
            // Preserves the old 8 px/frame movement at 60 FPS.
            player_speed: 480.0,
            player_sprite_size: 30.0,
            max_player_hp: 5,
            prompt_fade_seconds: 0.7,
            border_expand_seconds: 1.2,
            // The old timer switched phases every 100 frames at 60 FPS.
            hazard_phase_seconds: 100.0 / 60.0,
            hazard_half_size: 100.0,
            opponent_size: vec2(270.0, 195.0),
            opponent_gap: 50.0,
        }
    }
}

struct Assets {
    player: Texture2D,
    opponent: Texture2D,
    theme: Sound,
    savepoint: Sound,
    damage: Sound,
    gameplay_music: Sound,
}

impl Assets {
    async fn load() -> Self {
        Self {
            player: load_texture("./assets/image/player.png")
                .await
                .expect("failed to load assets/image/player.png"),
            opponent: load_texture("./assets/image/asgore.png")
                .await
                .expect("failed to load assets/image/asgore.png"),
            theme: audio::load_sound("./assets/audio/asgore_theme.wav")
                .await
                .expect("failed to load assets/audio/asgore_theme.wav"),
            savepoint: audio::load_sound("./assets/audio/savepoint.wav")
                .await
                .expect("failed to load assets/audio/savepoint.wav"),
            damage: audio::load_sound("./assets/audio/damage_taken.wav")
                .await
                .expect("failed to load assets/audio/damage_taken.wav"),
            gameplay_music: audio::load_sound("./assets/audio/driving-in-my-car.wav")
                .await
                .expect("failed to load assets/audio/driving-in-my-car.wav"),
        }
    }
}

#[derive(Clone, Copy)]
enum GameState {
    Landing,
    PromptFade,
    Starting,
    Run,
    GameOver,
}

struct Player {
    position: Vec2,
    radius: f32,
    speed: f32,
    hp: i32,
    max_hp: i32,
}

impl Player {
    fn new(config: &GameConfig) -> Self {
        Self {
            position: Vec2::ZERO,
            radius: config.player_radius,
            speed: config.player_speed,
            hp: config.max_player_hp,
            max_hp: config.max_player_hp,
        }
    }

    fn reset(&mut self) {
        self.position = Vec2::ZERO;
        self.hp = self.max_hp;
    }

    fn update(&mut self, config: &GameConfig, delta: f32) {
        let mut direction = Vec2::ZERO;

        if is_key_down(KeyCode::Up) {
            direction.y -= 1.0;
        }
        if is_key_down(KeyCode::Down) {
            direction.y += 1.0;
        }
        if is_key_down(KeyCode::Left) {
            direction.x -= 1.0;
        }
        if is_key_down(KeyCode::Right) {
            direction.x += 1.0;
        }

        self.position += direction * self.speed * delta;

        let safe_half_width = config.arena_half_width
            - self.radius
            - config.border_thickness / 2.0
            - config.wall_padding;
        let safe_half_height = config.arena_half_height
            - self.radius
            - config.border_thickness / 2.0
            - config.wall_padding;

        self.position.x = self.position.x.clamp(-safe_half_width, safe_half_width);
        self.position.y = self.position.y.clamp(-safe_half_height, safe_half_height);
    }

    fn overlaps_hazard(&self, config: &GameConfig) -> bool {
        let closest_x = self
            .position
            .x
            .clamp(-config.hazard_half_size, config.hazard_half_size);
        let closest_y = self
            .position
            .y
            .clamp(-config.hazard_half_size, config.hazard_half_size);
        let dx = self.position.x - closest_x;
        let dy = self.position.y - closest_y;

        dx * dx + dy * dy <= self.radius * self.radius
    }
}

struct Hazard {
    active: bool,
    phase_elapsed: f32,
}

impl Hazard {
    fn new() -> Self {
        Self {
            active: false,
            phase_elapsed: 0.0,
        }
    }

    fn reset(&mut self) {
        self.active = false;
        self.phase_elapsed = 0.0;
    }

    // Returns true when the player takes damage at the end of an active phase.
    fn update(&mut self, delta: f32, player: &Player, config: &GameConfig) -> bool {
        self.phase_elapsed += delta;
        if self.phase_elapsed < config.hazard_phase_seconds {
            return false;
        }

        self.phase_elapsed = 0.0;
        let takes_damage = self.active && player.overlaps_hazard(config);
        self.active = !self.active;
        takes_damage
    }
}

pub struct Game {
    config: GameConfig,
    assets: Assets,
    state: GameState,
    player: Player,
    hazard: Hazard,
    transition_elapsed: f32,
    current_arena_half_width: f32,
}

impl Game {
    pub async fn new() -> Self {
        let config = GameConfig::default();
        let assets = Assets::load().await;

        audio::play_sound(
            &assets.theme,
            PlaySoundParams {
                looped: true,
                volume: 1.0,
            },
        );

        Self {
            player: Player::new(&config),
            config,
            assets,
            state: GameState::Landing,
            hazard: Hazard::new(),
            transition_elapsed: 0.0,
            current_arena_half_width: 0.0,
        }
    }

    pub fn update(&mut self, delta: f32) {
        match self.state {
            GameState::Landing => self.update_landing(),
            GameState::PromptFade => self.update_prompt_fade(delta),
            GameState::Starting => self.update_starting(delta),
            GameState::Run => self.update_run(delta),
            GameState::GameOver => self.update_game_over(),
        }
    }

    pub fn draw(&self) {
        let center = vec2(screen_width() / 2.0, screen_height() / 2.0);

        match self.state {
            GameState::Landing => self.draw_landing(center, 1.0),
            GameState::PromptFade => {
                let progress =
                    (self.transition_elapsed / self.config.prompt_fade_seconds).clamp(0.0, 1.0);
                self.draw_landing(center, 1.0 - progress);
            }
            GameState::Starting => self.draw_arena(center, self.current_arena_half_width),
            GameState::Run => self.draw_run(center),
            GameState::GameOver => {
                draw_text(
                    "GameOver: press SPACE to replay",
                    center.x - 340.0,
                    center.y,
                    50.0,
                    WHITE,
                );
            }
        }
    }

    fn update_landing(&mut self) {
        if is_key_pressed(KeyCode::Space) {
            audio::play_sound_once(&self.assets.savepoint);
            self.player.reset();
            self.hazard.reset();
            self.transition_elapsed = 0.0;
            self.current_arena_half_width = 0.0;
            self.state = GameState::PromptFade;
        }
    }

    fn update_prompt_fade(&mut self, delta: f32) {
        self.transition_elapsed += delta;
        let progress =
            (self.transition_elapsed / self.config.prompt_fade_seconds).clamp(0.0, 1.0);
        audio::set_sound_volume(&self.assets.theme, 1.0 - progress);

        if progress >= 1.0 {
            audio::stop_sound(&self.assets.theme);
            self.transition_elapsed = 0.0;
            self.state = GameState::Starting;
        }
    }

    fn update_starting(&mut self, delta: f32) {
        self.transition_elapsed += delta;
        let progress =
            (self.transition_elapsed / self.config.border_expand_seconds).clamp(0.0, 1.0);
        self.current_arena_half_width = self.config.arena_half_width * progress;

        if progress >= 1.0 {
            self.current_arena_half_width = self.config.arena_half_width;
            self.transition_elapsed = 0.0;
            audio::play_sound(
                &self.assets.gameplay_music,
                PlaySoundParams {
                    looped: true,
                    volume: 0.5,
                },
            );
            self.state = GameState::Run;
        }
    }

    fn update_run(&mut self, delta: f32) {
        if self.hazard.update(delta, &self.player, &self.config) {
            self.player.hp -= 1;
            audio::play_sound_once(&self.assets.damage);
        }

        self.player.update(&self.config, delta);

        if self.player.hp <= 0 {
            audio::stop_sound(&self.assets.gameplay_music);
            self.state = GameState::GameOver;
        }
    }

    fn update_game_over(&mut self) {
        if is_key_pressed(KeyCode::Space) {
            audio::play_sound(
                &self.assets.theme,
                PlaySoundParams {
                    looped: true,
                    volume: 1.0,
                },
            );
            self.state = GameState::Landing;
        }
    }

    fn draw_landing(&self, center: Vec2, alpha: f32) {
        draw_text(
            "press SPACE to start",
            center.x - 220.0,
            center.y,
            50.0,
            Color::new(1.0, 1.0, 1.0, alpha),
        );
    }

    fn draw_arena(&self, center: Vec2, half_width: f32) {
        draw_rectangle_lines(
            center.x - half_width,
            center.y - self.config.arena_half_height,
            2.0 * half_width,
            2.0 * self.config.arena_half_height,
            self.config.border_thickness,
            WHITE,
        );
    }

    fn draw_run(&self, center: Vec2) {
        let opponent_top = center.y
            - self.config.arena_half_height
            - self.config.opponent_size.y
            - self.config.opponent_gap;
        draw_texture_ex(
            &self.assets.opponent,
            center.x - self.config.opponent_size.x / 2.0,
            opponent_top,
            WHITE,
            DrawTextureParams {
                dest_size: Some(self.config.opponent_size),
                ..Default::default()
            },
        );

        self.draw_arena(center, self.config.arena_half_width);

        if self.hazard.active {
            let size = self.config.hazard_half_size * 2.0;
            draw_rectangle(
                center.x - self.config.hazard_half_size,
                center.y - self.config.hazard_half_size,
                size,
                size,
                YELLOW,
            );
        }

        let player_screen_position = center + self.player.position;
        let sprite_half_size = self.config.player_sprite_size / 2.0;
        draw_texture_ex(
            &self.assets.player,
            player_screen_position.x - sprite_half_size,
            player_screen_position.y - sprite_half_size,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(
                    self.config.player_sprite_size,
                    self.config.player_sprite_size,
                )),
                ..Default::default()
            },
        );

        let health_text = format!("HP: {} / {}", self.player.hp, self.player.max_hp);
        draw_text(
            &health_text,
            center.x - self.config.arena_half_width + 30.0,
            center.y + self.config.arena_half_height + 50.0,
            50.0,
            WHITE,
        );
    }
}
