use macroquad::prelude::*;

// handles if the game is going or over
enum GameState {
    Playing,
    Gameover { game_over_time: f64},
    Start { game_start_time: f64},
}

//what to do if the enemy is alive or dead
enum EnemyState {
    InFormation,
    Dead,
}

//enemy struct
struct EnemyShip {
    pos: Vec2,
    speed: f32,
    size: f32,
    collided: bool,
    state: EnemyState,
    color: Color,
}

//setting up enemy
impl EnemyShip {
    fn new(x: f32, y: f32, speed:f32, size: f32) -> Self {
        let colorNum = rand::gen_range(0.0, 3.0);
        let mut color = if colorNum < 1.0 {
        GREEN
        } else if 1.0 <= colorNum  && colorNum < 2.0 {
         YELLOW
        } else {
         BLUE
        };
        Self {
            pos: Vec2::new(x,y),
            speed,
            size,
            collided: false,
            state: EnemyState::InFormation,
            color,
        }
    }

    //enemy gird shifts left and right
    fn moveInFormation(&mut self, leftmost: f32, rightmost: f32, screen_width: f32) {
        if rightmost > screen_width - 10.0 || leftmost < 10.0 {
            self.speed = -self.speed;
        }
        self.pos.x += self.speed;
    }

    //enemy draw function
    fn draw(&self) {
        
        draw_rectangle(
            self.pos.x, 
            self.pos.y,
            self.size,
            self.size,
            self.color,

        )
    }

    //handles if enemy gets hit
    fn hit(&mut self) {
        self.collided = true;
        self.state = EnemyState::Dead;
    }

    //enemy shoots
    fn shoot(&self, speed: f32) -> Missile {
        Missile::new(
            self.pos + Vec2::new(self.size / 2.0, self.size), 
            Vec2::new(0.0, speed), 
            get_time(), 
        )
    }
}

//player struct
struct PlayerShip {
    pos: Vec2,
    speed: f32,
    size: f32,
    collided: bool,
}   

impl PlayerShip {
    fn new(x: f32, y: f32, speed: f32, size: f32) -> Self {
        Self {
            pos: Vec2::new(x,y),
            speed,
            size,
            collided: false,
        }
    }

    //moves player ship left
    fn move_left(&mut self) {
        if self.pos.x - self.size > 0.0  {
            self.pos.x -= self.speed;
        }
    }

    //moves player ship right
    fn move_right(&mut self) {
        if self.pos.x + self.size < screen_width()  {
            self.pos.x += self.speed;
        }
    }

    fn draw(&self) {
        draw_triangle(
            Vec2::new(self.pos.x, self.pos.y - self.size),
            Vec2::new(self.pos.x - self.size, self.pos.y + self.size),
            Vec2::new(self.pos.x + self.size, self.pos.y + self.size),
            WHITE
        );
    }
}

//missile stuct
struct Missile {
    pos: Vec2,
    vel: Vec2,
    shot_at: f64,
    collided: bool,
}

impl Missile {
    fn new(pos: Vec2, vel: Vec2, shot_at: f64) -> Self {
        Missile { pos, vel, shot_at, collided: false }
    }

    fn update(&mut self) {
        self.pos += self.vel;
    }

    fn draw(&self) {
        draw_rectangle(self.pos.x, self.pos.y, 8.0, 17.0, RED);
    }
}

//wave struct
struct Wave {
    enemies: Vec<EnemyShip>,
    wave_number: f32,
}

impl Wave {
    //determines how many enemies to put in grid depending on wave number
    fn new(wave_number: f32, num_enemies: i32, enemy_speed: f32) -> Self {
        let mut enemies = Vec::new();
        
        for i in 0..num_enemies {
            let x = 50.0 + (i % 10) as f32 * 50.0;
            let y = 200.0 + (i / 10) as f32 * 40.0;
            enemies.push(EnemyShip::new(x, y, enemy_speed, 25.0))
        }

        Self {
            enemies,
            wave_number,
        }
        
    }

    //helps with the moving of the enemy grid, keeps it within the bounds of the screen 
    fn update(&mut self, player_pos: Vec2) {
        let mut leftmost = f32::MAX;
        let mut rightmost = f32::MIN;

        // Determine the bounds of the wave
        for enemy in &self.enemies {
            if enemy.pos.x < leftmost {
                leftmost = enemy.pos.x;
            }
            if enemy.pos.x + enemy.size > rightmost {
                rightmost = enemy.pos.x + enemy.size;
            }
        }

        // Update enemies
        for enemy in &mut self.enemies {
            enemy.moveInFormation(leftmost, rightmost, screen_width());
        }
    }

    fn draw(&self) {
        for enemy in &self.enemies {
            enemy.draw()
        }
    }
}

//check to see if player missile collides with enemy
fn check_enemy_collide(missile: &Missile, enemy: &EnemyShip) -> bool {
    let missile_rect = Rect::new(missile.pos.x, missile.pos.y, 5.0, 10.0);
    let enemy_rect = Rect::new(enemy.pos.x, enemy.pos.y, enemy.size, enemy.size);

    missile_rect.overlaps(&enemy_rect)
}

//checks to see if enemy missile collides with player
fn check_player_collide(missile: &Missile, player: &PlayerShip) -> bool {
    let missile_rect = Rect::new(missile.pos.x, missile.pos.y, 5.0, 10.0);
    let player_rect = Rect::new(
        player.pos.x - player.size, 
        player.pos.y - player.size, 
        player.size * 2.0, 
        player.size * 2.0,
    );

    missile_rect.overlaps(&player_rect)
}

#[macroquad::main("Rustaga")]
async fn main() {
    //initializes screen size
    request_new_screen_size(680.0,1080.0);

    //initializes player ship and missiles collection
    let mut player_ship = PlayerShip::new((screen_width() / 2.0) - 50.0, 960.0, 5.0, 20.0);
    let mut player_missiles: Vec<Missile> = Vec::new();
    
    //initializes waves and enemies
    let mut wave_number = 1.0;
    let mut num_enemies = 20;
    let mut fire_rate = 0.0005;
    let mut wave = Wave::new(wave_number, num_enemies, 0.5);
    let mut enemy_missiles: Vec<Missile> = Vec::new();
    
    //makes so that cant spam fire
    let mut last_shot = get_time(); 
    
    //scoring
    let mut num_shots = 0.0;
    let mut enemies_killed = 0.0;
    let mut lives_left = 3.0;
    let mut score: f32 = 0000.0;

    let game_start_time = get_time();
    let mut state = GameState::Start {
        game_start_time,
    };


    //main loop
    loop {

        match state {
            GameState::Start { game_start_time } => {
                clear_background(BLANK);

                draw_text("START", (screen_width() / 2.0) - 65.0, 250.0, 50.0, RED );

                //draw top bar that shows score
                draw_rectangle(0.0, 0.0, screen_width(), 100.0, GRAY);
                
                //draw score
                draw_text("SCORE: ", 25.0, 60.0, 30.0, WHITE);
                let scorestr: String = score.to_string();
                let strscore: &str = &scorestr;
                draw_text(strscore, 105.0, 60.0, 30.0, WHITE);

                //draw logo
                draw_text("RUSTAGA", (screen_width() / 2.0) - 90.0, 60.0, 50.0, RED);

                //draw stage
                draw_text("STAGE: ", screen_width() - 125.0, 60.0, 30.0, WHITE);
                let stagestr: String = wave_number.to_string();
                let strstage: &str = &stagestr;
                draw_text(strstage, screen_width() - 45.0, 60.0, 30.0, WHITE);

                //draws bottom bar that shows how many ships you have left
                draw_rectangle(0.0, 980.0, screen_width(), 100.0, GRAY);

                //draws ships left
              
                let extra_ship1 = PlayerShip::new(25.0, 1020.0, 5.0, 20.0);
                extra_ship1.draw();
                let extra_ship2 = PlayerShip::new(70.0, 1020.0, 5.0, 20.0);
                extra_ship2.draw();
                let extra_ship3 = PlayerShip::new(115.0, 1020.0, 5.0, 20.0);
                extra_ship3.draw();
                let extra_ship3 = PlayerShip::new(155.0, 1020.0, 5.0, 20.0);
                extra_ship3.draw();
              
                //pauses on start screen for 5 seconds
                if get_time() - game_start_time >= 5.0 {
                    state = GameState::Playing;
                    continue;
                }
                
            }

            GameState::Playing => {
                clear_background(BLANK);

                //more shot timing
                let frame_t = get_time();


                //checks if missles hit the enemy ship
                for missile in &mut player_missiles {
                    missile.update();
                    for enemy in &mut wave.enemies {
                        if !missile.collided && !enemy.collided && check_enemy_collide(missile, enemy) {
                            missile.collided = true;
                            enemy.collided = true;
                            score += 100.0;
                            enemies_killed += 1.0;
                        }
                    }
                }

                //removes missile and enemy if hit
                player_missiles.retain(|missile| !missile.collided && missile.pos.y > 100.0);
                wave.enemies.retain(|enemy_ship| !enemy_ship.collided);

                //checks if enemy missile hits player
                for missile in &mut enemy_missiles {
                    missile.update();
                    if !missile.collided && !player_ship.collided && check_player_collide(missile, &player_ship) {
                        missile.collided = true;
                        player_ship.collided = true;
                        lives_left -= 1.0;

                        if lives_left > 0.0 {
                            player_ship.pos = Vec2::new((screen_width() / 2.0) - 50.0, 960.0);
                            player_ship.collided = false;
                        }
                    }
                }

                //if the player gets hit reset 
                if player_ship.collided {
                    player_ship.pos = Vec2::new((screen_width() / 2.0) - 50.0, 960.0);
                    player_ship.collided = false;
                }

                //checks if all enemies are dead to move to next stage
                if wave.enemies.is_empty() {
                    wave_number += 1.0;
                    let mut new_enemy_speed = 0.5 + wave_number * 0.5; 
                    if new_enemy_speed > 5.0 {
                        new_enemy_speed = 5.0;
                    }
                    num_enemies += 5;
                    if num_enemies > 50 {
                        num_enemies = 50;
                    }
                    fire_rate += 0.0005;
                    wave = Wave::new(wave_number, num_enemies, new_enemy_speed);
                    score += 1000.0;
                }

                //draws enemy ships
                wave.update(player_ship.pos);
                wave.draw();

                //enemies randomly shoot
                for enemy in &wave.enemies {
                    if rand::gen_range(0.0, 1.0) < fire_rate {
                        enemy_missiles.push(enemy.shoot(5.0));
                    }
                }
            
                // Update and draw enemy missiles
                for missile in &mut enemy_missiles {
                    missile.update();
                    missile.draw();
                }
                //deletes missiles 
                enemy_missiles.retain(|missile| missile.pos.y < screen_height());

                //draws missles coming from the ship
                for missile in &mut player_missiles {
                    missile.update();
                    missile.draw();
                }
                //deletes missiles
                player_missiles.retain(|missile| missile.pos.y > 100.0);

                //draw player ship
                player_ship.draw();


                //draw top bar that shows score
                draw_rectangle(0.0, 0.0, screen_width(), 100.0, GRAY);
                
                //draw score
                draw_text("SCORE: ", 25.0, 50.0, 30.0, WHITE);
                let scorestr: String = score.to_string();
                let strscore: &str = &scorestr;
                draw_text(strscore, 105.0, 50.0, 30.0, WHITE);

                //draw stage
                draw_text("STAGE: ", screen_width() - 175.0, 50.0, 30.0, WHITE);
                let stagestr: String = wave_number.to_string();
                let strstage: &str = &stagestr;
                draw_text(strstage, screen_width() - 95.0, 50.0, 30.0, WHITE);

                //draws bottom bar that shows how many ships you have left
                draw_rectangle(0.0, 980.0, screen_width(), 100.0, GRAY);

                //draws ships left
                if lives_left == 3.0 {
                    let extra_ship1 = PlayerShip::new(25.0, 1020.0, 5.0, 20.0);
                    extra_ship1.draw();
                    let extra_ship2 = PlayerShip::new(70.0, 1020.0, 5.0, 20.0);
                    extra_ship2.draw();
                    let extra_ship3 = PlayerShip::new(115.0, 1020.0, 5.0, 20.0);
                    extra_ship3.draw();
                }  
                else if lives_left == 2.0 {
                    let extra_ship1 = PlayerShip::new(25.0, 1020.0, 5.0, 20.0);
                    extra_ship1.draw();
                    let extra_ship2 = PlayerShip::new(70.0, 1020.0, 5.0, 20.0);
                    extra_ship2.draw();
                }
                else if lives_left == 1.0 {
                    let extra_ship1 = PlayerShip::new(25.0, 1020.0, 5.0, 20.0);
                    extra_ship1.draw();
                }
                else if lives_left < 0.0 {
                    state = GameState::Gameover {
                        game_over_time: get_time(),
                    };
                }

                //handles keypresses
                if is_key_down(KeyCode::D) {
                    player_ship.move_right();
                }
                if is_key_down(KeyCode::A) {
                    player_ship.move_left();
                }
                if is_key_down(KeyCode::Space) && frame_t - last_shot > 0.30 {
                    player_missiles.push(Missile::new(
                        Vec2::new(player_ship.pos.x, player_ship.pos.y - 20.0),
                        Vec2::new(0.0, -10.0),
                        frame_t,
                    ));
                    last_shot = frame_t;
                    num_shots += 1.0;
                }
            }
            
            GameState::Gameover { game_over_time } => {
                clear_background(BLACK);

                draw_text("GAME OVER", (screen_width() / 2.0) - 90.0, 250.0, 50.0, RED );
                draw_text("FINAL SCORE: ", (screen_width() / 2.0) - 100.0, 300.0, 30.0, WHITE);
                let scorestr: String = score.to_string();
                let strscore: &str = &scorestr;
                draw_text(strscore, (screen_width() / 2.0) + 60.0, 300.0, 30.0, WHITE);
                draw_text("FINAL STAGE: ", (screen_width() / 2.0) - 100.0, 350.0, 30.0, WHITE);
                let stagestr: String = wave_number.to_string();
                let strstage: &str = &stagestr;
                draw_text(strstage, (screen_width() / 2.0) + 60.0 , 350.0, 30.0, WHITE);
                draw_text("NUMBER OF SHOTS: ", (screen_width() / 2.0) - 100.0, 400.0, 30.0, WHITE);
                let shotstr: String = num_shots.to_string();
                let strshots: &str = &shotstr;
                draw_text(strshots, (screen_width() / 2.0) + 125.0 , 400.0, 30.0, WHITE);
                draw_text("ACCURACY: ", (screen_width() / 2.0) - 100.0, 450.0, 30.0, WHITE);
                let accuracy = (enemies_killed / num_shots) * 100.0;
                let acrstr = format!("{:.2}", accuracy);
                draw_text(&acrstr, (screen_width() / 2.0) + 20.0, 450.0, 30.0, WHITE);
                draw_text("%", (screen_width() / 2.0) + 85.0, 450.0, 30.0, WHITE);


                
                //pause on game over screen for 5 seconds
                if get_time() - game_over_time >= 5.0 {
                    break;
                }
            }
        }
        next_frame().await;
    }
}

