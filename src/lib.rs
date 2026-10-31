use macroquad::{
    audio::{self, PlaySoundParams, Sound},
    prelude::*,
};

/*
* GameConfig ---------------------------------------
*/
struct GameConfig {
    // Arena
    arena_half_width: f32,
    arena_half_height: f32,
    border_thickness: f32,
    wall_padding: f32,
    // Player
    player_radius: f32,
    player_speed: f32,
    player_sprite_size: f32,
    max_player_hp: i32,
    // Transition
    prompt_fade_seconds: f32,
    border_expand_seconds: f32,
    // Enemy
    enemy_shot_interval_seconds: f32,
    bullet_speed: f32,
    bullet_radius: f32,
    opponent_size: Vec2,
    opponent_gap: f32,
    enemy_max_hp: i32,
    // Target
    target_radius: f32,
    target_visible_seconds: f32,
    target_respawn_seconds: f32,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            // Arena
            arena_half_width: 300.0,
            arena_half_height: 200.0,
            border_thickness: 10.0,
            wall_padding: 3.0,
            // Player
            player_radius: 20.0,
            player_speed: 300.0,
            player_sprite_size: 30.0,
            max_player_hp: 5,
            // Transition
            prompt_fade_seconds: 0.7,
            border_expand_seconds: 1.2,
            // Enemy
            enemy_shot_interval_seconds: 0.5,
            bullet_speed: 220.0,
            bullet_radius: 7.0,
            opponent_size: vec2(270.0, 195.0),
            opponent_gap: 50.0,
            enemy_max_hp: 5,
            target_radius: 15.0,
            target_visible_seconds: 2.5,
            target_respawn_seconds: 8.0,
        }
    }
}

impl GameConfig {
    fn opponent_center_offset(&self) -> Vec2 {
        vec2(
            0.0,
            -self.arena_half_height - self.opponent_gap - self.opponent_size.y / 2.0,
        )
    }
}

/*
* Assets ---------------------------------------
*/
struct Assets {
    player: Texture2D,
    opponent: Texture2D,
    theme: Sound,
    savepoint: Sound,
    damage: Sound,
    heal: Sound,
    lose: Sound,
    win: Sound,
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
            theme: audio::load_sound("./assets/audio/asgore_theme.ogg")
                .await
                .expect("failed to load assets/audio/asgore_theme.ogg"),
            savepoint: audio::load_sound("./assets/audio/savepoint.ogg")
                .await
                .expect("failed to load assets/audio/savepoint.ogg"),
            damage: audio::load_sound("./assets/audio/damage_taken.ogg")
                .await
                .expect("failed to load assets/audio/damage_taken.ogg"),
            heal: audio::load_sound("./assets/audio/undertale-heal.ogg")
                .await
                .expect("failed to load assets/audio/undertale-heal.ogg"),
            lose: audio::load_sound("./assets/audio/undertale-death.ogg")
                .await
                .expect("failed to load assets/audio/undertale-death.ogg"),
            win: audio::load_sound("./assets/audio/kids-yay.ogg")
                .await
                .expect("failed to load assets/audio/kids-yay.ogg"),
            gameplay_music: audio::load_sound("./assets/audio/driving-in-my-car.ogg")
                .await
                .expect("failed to load assets/audio/driving-in-my-car.ogg"),
        }
    }
}

/*
* GameState ---------------------------------------
*/
#[derive(Clone, Copy)]
enum GameState {
    Landing,
    PromptFade,
    Starting,
    Run,
    GameOver,
    Win,
}

/*
* Player ---------------------------------------
*/
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
}

/*
* Enemy and Bullet ---------------------------------------
*/
struct Enemy {
    max_hp: i32,
    hp: i32,
    shot_elapsed: f32,
}

impl Enemy {
    fn new(max_hp: i32) -> Self {
        Self {
            max_hp,
            hp: max_hp,
            shot_elapsed: 0.0,
        }
    }

    fn reset(&mut self) {
        self.shot_elapsed = 0.0;
        self.hp = self.max_hp;
    }

    fn update(&mut self, delta: f32, target: Vec2, config: &GameConfig) -> Option<Bullet> {
        self.shot_elapsed += delta;
        if self.shot_elapsed < config.enemy_shot_interval_seconds {
            return None;
        }

        self.shot_elapsed = 0.0;

        // Player and Bullet use position relative to Arena's center
        let spawn_position = config.opponent_center_offset();
        let direction = (target - spawn_position).normalize_or_zero();

        Some(Bullet {
            position: spawn_position,
            velocity: direction * config.bullet_speed,
            radius: config.bullet_radius,
        })
    }
}

struct Bullet {
    position: Vec2,
    velocity: Vec2,
    radius: f32,
}

/*
* Target ---------------------------------------
*/
#[derive(Clone, Copy)]
enum TargetPhase {
    Waiting { remaining: f32 },
    Visible { remaining: f32 },
}

struct TargetArea {
    position: Vec2,
    phase: TargetPhase,
}

impl TargetArea {
    fn new() -> Self {
        Self {
            position: Vec2::ZERO,
            phase: TargetPhase::Waiting { remaining: 0.0 },
        }
    }

    fn reset(&mut self) {
        self.position = Vec2::ZERO;
        // The first target appears as soon as the run begins.
        self.phase = TargetPhase::Waiting { remaining: 0.0 };
    }

    fn is_visible(&self) -> bool {
        matches!(self.phase, TargetPhase::Visible { .. })
    }

    // Returns true once when the player collects the visible target.
    fn update(
        &mut self,
        delta: f32,
        player_position: Vec2,
        player_radius: f32,
        config: &GameConfig,
    ) -> bool {
        match self.phase {
            TargetPhase::Waiting { remaining } => {
                let remaining = remaining - delta;
                if remaining <= 0.0 {
                    self.position = Self::random_position(config);
                    self.phase = TargetPhase::Visible {
                        remaining: config.target_visible_seconds,
                    };
                } else {
                    self.phase = TargetPhase::Waiting { remaining };
                }
                false
            }
            TargetPhase::Visible { remaining } => {
                let combined_radius = config.target_radius + player_radius;
                let collected = self.position.distance_squared(player_position)
                    <= combined_radius * combined_radius;
                let remaining = remaining - delta;

                if collected {
                    self.phase = TargetPhase::Waiting {
                        remaining: config.target_respawn_seconds,
                    };
                    true
                } else if remaining <= 0.0 {
                    self.phase = TargetPhase::Waiting {
                        remaining: config.target_respawn_seconds,
                    };
                    false
                } else {
                    self.phase = TargetPhase::Visible { remaining };
                    false
                }
            }
        }
    }

    fn random_position(config: &GameConfig) -> Vec2 {
        let margin = config.target_radius + config.border_thickness / 2.0 + config.wall_padding;
        let min_x = -config.arena_half_width + margin;
        let max_x = config.arena_half_width - margin;
        let min_y = -config.arena_half_height + margin;
        let max_y = config.arena_half_height - margin;

        vec2(
            macroquad::rand::gen_range(min_x, max_x),
            macroquad::rand::gen_range(min_y, max_y),
        )
    }
}

/*
* Game =================================
*/
pub struct Game {
    config: GameConfig,
    assets: Assets,
    state: GameState,
    player: Player,
    enemy: Enemy,
    bullets: Vec<Bullet>,
    target_area: TargetArea,
    transition_elapsed: f32,
    current_arena_half_width: f32,
}

impl Game {
    pub async fn new() -> Self {
        let config = GameConfig::default();
        let assets = Assets::load().await;
        let enemy = Enemy::new(config.enemy_max_hp);

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
            enemy,
            bullets: Vec::new(),
            target_area: TargetArea::new(),
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
            GameState::Win => self.update_win(),
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
                draw_text("DEFEATED", center.x - 145.0, center.y, 90.0, RED);
                draw_text(
                    "press SPACE to reset",
                    center.x - 142.0,
                    center.y + 40.0,
                    36.0,
                    WHITE,
                );
            }
            GameState::Win => {
                draw_text("YOU WIN!", center.x - 140.0, center.y, 90.0, GREEN);
                draw_text(
                    "Press SPACE to reset",
                    center.x - 142.0,
                    center.y + 40.0,
                    36.0,
                    WHITE,
                );
            }
        }
    }

    // update -------------------------------
    fn update_landing(&mut self) {
        if is_key_pressed(KeyCode::Space) {
            audio::play_sound_once(&self.assets.savepoint);
            self.player.reset();
            self.enemy.reset();
            self.bullets.clear();
            self.target_area.reset();
            self.transition_elapsed = 0.0;
            self.current_arena_half_width = 0.0;
            self.state = GameState::PromptFade;
        }
    }

    fn update_prompt_fade(&mut self, delta: f32) {
        self.transition_elapsed += delta;
        let progress = (self.transition_elapsed / self.config.prompt_fade_seconds).clamp(0.0, 1.0);
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
        self.player.update(&self.config, delta);

        let target_collected = self.target_area.update(
            delta,
            self.player.position,
            self.player.radius,
            &self.config,
        );
        if target_collected {
            audio::play_sound_once(&self.assets.heal);
            self.enemy.hp = (self.enemy.hp - 1).max(0);
        }

        if self.enemy.hp <= 0 {
            audio::stop_sound(&self.assets.gameplay_music);
            audio::play_sound_once(&self.assets.win);
            self.state = GameState::Win;
            return;
        }

        let hit_count = self.update_bullets(delta);
        if hit_count > 0 {
            self.player.hp = (self.player.hp - hit_count).max(0);
            audio::play_sound_once(&self.assets.damage);
        }

        if self.player.hp <= 0 {
            audio::stop_sound(&self.assets.gameplay_music);
            audio::play_sound_once(&self.assets.lose);
            self.state = GameState::GameOver;
            return;
        }

        if let Some(bullet) = self.enemy.update(delta, self.player.position, &self.config) {
            self.bullets.push(bullet);
        }
    }

    fn update_bullets(&mut self, delta: f32) -> i32 {
        let player_position = self.player.position;
        let player_radius = self.player.radius;

        let half_width = self.config.arena_half_width;
        let half_height = self.config.arena_half_height;
        let upper_limit = -half_height - self.config.opponent_gap - self.config.opponent_size.y;

        let mut hit_count = 0;

        self.bullets.retain_mut(|bullet| {
            bullet.position += bullet.velocity * delta;

            let combined_radius = bullet.radius + player_radius;
            if bullet.position.distance_squared(player_position)
                <= combined_radius * combined_radius
            {
                hit_count += 1;
                return false;
            }

            let outside_bounds = bullet.position.x < -half_width - bullet.radius
                || bullet.position.x > half_width + bullet.radius
                || bullet.position.y < upper_limit - bullet.radius
                || bullet.position.y > half_height + bullet.radius;

            !outside_bounds
        });

        hit_count
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

    fn update_win(&mut self) {
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

    // draw -------------------------------
    fn draw_landing(&self, center: Vec2, alpha: f32) {
        draw_text(
            "ASGORE",
            center.x - 105.0,
            center.y,
            90.0,
            Color::new(1.0, 0.0, 0.0, alpha),
        );
        draw_text(
            "press SPACE to challenge",
            center.x - 170.0,
            center.y + 40.0,
            36.0,
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
        let opponent_center = center + self.config.opponent_center_offset();
        let opponent_size = self.config.opponent_size;

        draw_texture_ex(
            &self.assets.opponent,
            opponent_center.x - opponent_size.x / 2.0,
            opponent_center.y - opponent_size.y / 2.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(opponent_size),
                ..Default::default()
            },
        );

        self.draw_arena(center, self.config.arena_half_width);

        if self.target_area.is_visible() {
            let target_screen_position = center + self.target_area.position;
            draw_circle(
                target_screen_position.x,
                target_screen_position.y,
                self.config.target_radius,
                GREEN,
            );
        }

        for bullet in &self.bullets {
            let screen_position = center + bullet.position;
            draw_circle(screen_position.x, screen_position.y, bullet.radius, YELLOW);
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

        let health_text = format!("Player HP: {}/{}", self.player.hp, self.player.max_hp);
        draw_text(
            &health_text,
            center.x - self.config.arena_half_width + 30.0,
            center.y + self.config.arena_half_height + 50.0,
            50.0,
            WHITE,
        );

        let enemy_health_text = format!("Asgore HP: {}/{}", self.enemy.hp, self.enemy.max_hp);
        draw_text(
            &enemy_health_text,
            center.x - 90.0,
            center.y - self.config.arena_half_height - 20.0,
            28.0,
            WHITE,
        );
    }
}
