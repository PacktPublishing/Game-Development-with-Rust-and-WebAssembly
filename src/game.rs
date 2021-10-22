use crate::engine::Audio;
use crate::engine::Sound;
use std::rc::Rc;

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use engine::Image;
use rand::prelude::*;
use web_sys::HtmlImageElement;

use crate::{
    browser,
    engine::{self, Cell, Game, KeyState, Point, Rect, Renderer, Sheet, SpriteSheet},
    segments::*,
};

const FLOOR: i16 = 479;
const HEIGHT: i16 = 600;
const HEIGHT_OFFSET: i16 = HEIGHT - FLOOR;
const STARTING_POINT: i16 = -20;
const IDLE_FRAMES: u8 = 29;
const RUNNING_FRAMES: u8 = 23;
const JUMPING_FRAMES: u8 = 35;
const SLIDING_FRAMES: u8 = 14;
const FALLING_FRAMES: u8 = 29;
const RUNNING_SPEED: i16 = 3;
const IDLE_FRAME_NAME: &str = "Idle";
const RUN_FRAME_NAME: &str = "Run";
const SLIDING_FRAME_NAME: &str = "Slide";
const JUMPING_FRAME_NAME: &str = "Jump";
const FALLING_FRAME_NAME: &str = "Dead";
const JUMP_SPEED: i16 = -25;
const GRAVITY: i16 = 1;
const TERMINAL_VELOCITY: i16 = 20;

pub struct WalkTheDog {
    machine: Option<WalkTheDogStateMachine>,
}

impl WalkTheDog {
    pub fn new() -> Self {
        WalkTheDog { machine: None }
    }
}

enum WalkTheDogStateMachine {
    Ready(WalkTheDogState<Ready>),
    Walking(WalkTheDogState<Walking>),
    GameOver(WalkTheDogState<GameOver>),
}

impl WalkTheDogStateMachine {
    fn update(self, keystate: &KeyState) -> Self {
        match self {
            WalkTheDogStateMachine::Ready(state) => state.update(keystate),
            WalkTheDogStateMachine::Walking(state) => state.update(keystate),
            WalkTheDogStateMachine::GameOver(state) => state.update(keystate),
        }
    }
}

struct WalkTheDogState<T> {
    _state: T,
}

impl WalkTheDogState<Ready> {
    fn update(mut self, keystate: &KeyState) -> WalkTheDogStateMachine {
        if keystate.is_pressed("ArrowRight") {
            WalkTheDogStateMachine::Walking(self.into())
        } else {
            WalkTheDogStateMachine::Ready(self)
        }
        //        self._state.update(keystate)
        /*        if keystate.is_pressed("ArrowRight") {
            state._state.walk.boy.run_right();
        }

        if keystate.is_pressed("Space") {
            state._state.walk.boy.jump();
        }

        if keystate.is_pressed("ArrowDown") {
            state._state.walk.boy.slide();
        }

        state._state.walk.boy.update();

        let walking_speed = state._state.walk.velocity();
        let [first_background, second_background] = &mut state._state.walk.backgrounds;
        first_background.move_horizontally(walking_speed);
        second_background.move_horizontally(walking_speed);

        if first_background.right() < 0 {
            first_background.set_x(second_background.right());
        }
        if second_background.right() < 0 {
            second_background.set_x(first_background.right());
        }

        state
            ._state
            .walk
            .obstacles
            .retain(|obstacle| obstacle.right() > 0);

        for (_, obstacle) in state._state.walk.obstacles.iter_mut().enumerate() {
            obstacle.move_horizontally(walking_speed);
            obstacle.check_intersection(&mut state._state.walk.boy);
        }

        if state._state.walk.timeline < 1000 {
            state._state.walk.generate_next_segment()
        } else {
            state._state.walk.timeline += walking_speed;
        }

        self

        WalkTheDogStateMachine::Ready(self)*/
    }
}

struct Ready {
    walk: Walk,
}

impl Ready {
    fn update(&mut self) {
        self.walk.boy.update();
        /*
                let walking_speed = state._state.walk.velocity();
                let [first_background, second_background] = &mut state._state.walk.backgrounds;
                first_background.move_horizontally(walking_speed);
                second_background.move_horizontally(walking_speed);

                if first_background.right() < 0 {
                    first_background.set_x(second_background.right());
                }
                if second_background.right() < 0 {
                    second_background.set_x(first_background.right());
                }
        */
        /*       state
            ._state
            .walk
            .obstacles
            .retain(|obstacle| obstacle.right() > 0);

        for (_, obstacle) in state._state.walk.obstacles.iter_mut().enumerate() {
            obstacle.move_horizontally(walking_speed);
            obstacle.check_intersection(&mut state._state.walk.boy);
        }

        if state._state.walk.timeline < 1000 {
            state._state.walk.generate_next_segment()
        } else {
            state._state.walk.timeline += walking_speed;
        }

        self*/
    }
}

struct Walking {
    walk: Walk,
}

impl WalkTheDogState<Walking> {
    fn update(self, keystate: &KeyState) -> WalkTheDogStateMachine {
        WalkTheDogStateMachine::Walking(self)
    }
}

struct GameOver {
    walk: Walk,
}

impl WalkTheDogState<GameOver> {
    fn update(self, keystate: &KeyState) -> WalkTheDogStateMachine {
        WalkTheDogStateMachine::GameOver(self)
    }
}

impl From<WalkTheDogState<Ready>> for WalkTheDogState<Walking> {
    fn from(mut state: WalkTheDogState<Ready>) -> Self {
        state._state.walk.boy.run_right();

        WalkTheDogState {
            _state: Walking {
                walk: state._state.walk,
            },
        }
    }
}

pub trait Obstacle {
    fn check_intersection(&self, boy: &mut RedHatBoy);
    fn draw(&self, renderer: &Renderer);
    fn move_horizontally(&mut self, x: i16);
    fn right(&self) -> i16;
}

pub struct Platform {
    sheet: Rc<SpriteSheet>,
    bounding_box: Rect,
    sprites: Vec<String>,
}

impl Platform {
    pub fn new(sheet: Rc<SpriteSheet>, position: Point, sprites: Vec<String>) -> Self {
        let mut cells = sprites.iter().filter_map(|sprite| sheet.cell(sprite));
        let first_cell = cells.next();
        let height = first_cell.map_or(0, |cell| cell.frame.h);

        let width =
            cells.map(|cell| cell.frame.w).sum::<i16>() + first_cell.map_or(0, |cell| cell.frame.w);

        Platform {
            sheet: sheet.clone(),
            bounding_box: Rect {
                position,
                width,
                height,
            },
            sprites,
        }
    }

    fn bounding_box(&self) -> &Rect {
        &self.bounding_box
    }

    fn position(&self) -> &Point {
        &self.bounding_box.position
    }
}

impl Obstacle for Platform {
    fn check_intersection(&self, boy: &mut RedHatBoy) {
        if boy.bounding_box().intersects(&self.bounding_box()) {
            if boy.pos_y() < self.position().y {
                boy.land_on(self.bounding_box().y());
            } else {
                boy.kill();
            }
        }
    }

    fn draw(&self, renderer: &Renderer) {
        let mut x = 0;
        self.sprites.iter().for_each(move |sprite| {
            let cell = self
                .sheet
                .cell(sprite)
                .expect("Cell does not exist on draw! Should be impossible!");

            self.sheet.draw(
                renderer,
                &Rect::new_from_x_y(cell.frame.x, cell.frame.y, cell.frame.w, cell.frame.h),
                &Rect::new_from_x_y(
                    self.position().x + x,
                    self.position().y,
                    cell.frame.w,
                    cell.frame.h,
                ),
            );
            x += cell.frame.w;
        });
    }

    fn move_horizontally(&mut self, x: i16) {
        self.bounding_box.set_x(self.position().x + x);
    }

    fn right(&self) -> i16 {
        self.bounding_box().right()
    }
}

pub struct RedHatBoy {
    state: RedHatBoyStateMachine,
    sprite_sheet: Sheet,
    image: HtmlImageElement,
}

impl RedHatBoy {
    fn new(sheet: Sheet, image: HtmlImageElement, audio: Audio, sound: Sound) -> Self {
        RedHatBoy {
            state: RedHatBoyStateMachine::Idle(RedHatBoyState::new(audio, sound)),
            sprite_sheet: sheet,
            image,
        }
    }

    fn run_right(&mut self) {
        let state = self.state.clone();
        self.state = state.run();
    }

    fn slide(&mut self) {
        let state = self.state.clone();
        self.state = state.slide();
    }

    fn jump(&mut self) {
        let state = self.state.clone();
        self.state = state.jump();
    }

    fn kill(&mut self) {
        let state = self.state.clone();
        self.state = state.kill();
    }

    fn land_on(&mut self, position: i16) {
        let position = position - HEIGHT_OFFSET;
        let state = self.state.clone();
        self.state = state.land_on(position);
    }

    fn update(&mut self) {
        let state = self.state.clone();
        self.state = state.update();
    }

    fn pos_y(&self) -> i16 {
        self.state.game_object().position.y
    }

    fn walking_speed(&self) -> i16 {
        self.state.game_object().velocity.x
    }

    fn frame_name(&self) -> String {
        format!(
            "{} ({}).png",
            self.state.state_name(),
            (self.state.game_object().frame / 3) + 1
        )
    }

    fn current_sprite(&self) -> Option<&Cell> {
        self.sprite_sheet.frames.get(&self.frame_name())
    }

    fn bounding_box(&self) -> Rect {
        let sprite = self.current_sprite().expect("Cell not found");

        Rect::new_from_x_y(
            self.state.game_object().position.x + sprite.sprite_source_size.x,
            self.state.game_object().position.y + sprite.sprite_source_size.y,
            sprite.frame.w,
            sprite.frame.h,
        )
    }

    fn draw(&self, renderer: &Renderer) {
        let sprite = self.current_sprite().expect("Cell not found");

        renderer.draw_image(
            &self.image,
            &Rect::new_from_x_y(
                sprite.frame.x,
                sprite.frame.y,
                sprite.frame.w,
                sprite.frame.h,
            ),
            &self.bounding_box(),
        );
    }
}

#[derive(Clone)]
enum RedHatBoyStateMachine {
    Idle(RedHatBoyState<Idle>),
    Running(RedHatBoyState<Running>),
    Sliding(RedHatBoyState<Sliding>),
    Jumping(RedHatBoyState<Jumping>),
    Falling(RedHatBoyState<Falling>),
    KnockedOut(RedHatBoyState<KnockedOut>),
}

impl RedHatBoyStateMachine {
    fn run(self) -> Self {
        match self {
            RedHatBoyStateMachine::Idle(val) => RedHatBoyStateMachine::Running(val.into()),
            _ => self,
        }
    }

    fn jump(self) -> Self {
        match self {
            RedHatBoyStateMachine::Running(val) => RedHatBoyStateMachine::Jumping(val.into()),
            _ => self,
        }
    }

    fn slide(self) -> Self {
        match self {
            RedHatBoyStateMachine::Running(val) => RedHatBoyStateMachine::Sliding(val.into()),
            _ => self,
        }
    }

    fn kill(self) -> Self {
        match self {
            RedHatBoyStateMachine::Running(val) => RedHatBoyStateMachine::Falling(val.into()),
            RedHatBoyStateMachine::Jumping(val) => RedHatBoyStateMachine::Falling(val.into()),
            RedHatBoyStateMachine::Sliding(val) => RedHatBoyStateMachine::Falling(val.into()),
            _ => self,
        }
    }

    fn land_on(self, position: i16) -> Self {
        match self {
            RedHatBoyStateMachine::Jumping(mut state) => {
                state.game_object = state.game_object.set_on(position);
                RedHatBoyStateMachine::Running(state.into())
            }
            RedHatBoyStateMachine::Idle(mut state) => {
                state.game_object = state.game_object.set_on(position);
                RedHatBoyStateMachine::Idle(state.into())
            }
            RedHatBoyStateMachine::Running(mut state) => {
                state.game_object = state.game_object.set_on(position);
                RedHatBoyStateMachine::Running(state.into())
            }
            RedHatBoyStateMachine::Sliding(mut state) => {
                state.game_object = state.game_object.set_on(position);
                RedHatBoyStateMachine::Sliding(state.into())
            }
            RedHatBoyStateMachine::Falling(mut state) => {
                state.game_object = state.game_object.set_on(position);
                RedHatBoyStateMachine::Falling(state.into())
            }
            RedHatBoyStateMachine::KnockedOut(mut state) => {
                state.game_object = state.game_object.set_on(position);
                RedHatBoyStateMachine::KnockedOut(state.into())
            }
        }
    }

    fn state_name(&self) -> &str {
        match self {
            RedHatBoyStateMachine::Idle(_) => IDLE_FRAME_NAME,
            RedHatBoyStateMachine::Running(_) => RUN_FRAME_NAME,
            RedHatBoyStateMachine::Jumping(_) => JUMPING_FRAME_NAME,
            RedHatBoyStateMachine::Sliding(_) => SLIDING_FRAME_NAME,
            RedHatBoyStateMachine::Falling(_) => FALLING_FRAME_NAME,
            RedHatBoyStateMachine::KnockedOut(_) => FALLING_FRAME_NAME,
        }
    }

    fn game_object(&self) -> &GameObject {
        match self {
            RedHatBoyStateMachine::Idle(state) => &state.game_object,
            RedHatBoyStateMachine::Running(state) => &state.game_object,
            RedHatBoyStateMachine::Jumping(state) => &state.game_object,
            RedHatBoyStateMachine::Sliding(state) => &state.game_object,
            RedHatBoyStateMachine::Falling(state) => &state.game_object,
            RedHatBoyStateMachine::KnockedOut(state) => &state.game_object,
        }
    }

    fn update(self) -> Self {
        match self {
            RedHatBoyStateMachine::Idle(mut state) => {
                state.game_object = state.game_object.update(IDLE_FRAMES);
                RedHatBoyStateMachine::Idle(state)
            }
            RedHatBoyStateMachine::Running(mut state) => {
                state.game_object = state.game_object.update(RUNNING_FRAMES);
                RedHatBoyStateMachine::Running(state)
            }
            RedHatBoyStateMachine::Jumping(mut state) => {
                state.game_object = state.game_object.update(JUMPING_FRAMES);

                if state.game_object.position.y >= FLOOR {
                    RedHatBoyStateMachine::Running(state.into())
                } else {
                    RedHatBoyStateMachine::Jumping(state)
                }
            }
            RedHatBoyStateMachine::Sliding(mut state) => {
                state.game_object = state.game_object.update(SLIDING_FRAMES);

                if state.game_object.frame >= SLIDING_FRAMES {
                    RedHatBoyStateMachine::Running(state.into())
                } else {
                    RedHatBoyStateMachine::Sliding(state)
                }
            }
            RedHatBoyStateMachine::Falling(mut state) => {
                state.game_object = state.game_object.update(FALLING_FRAMES);

                if state.game_object.frame >= FALLING_FRAMES {
                    RedHatBoyStateMachine::KnockedOut(state.into())
                } else {
                    RedHatBoyStateMachine::Falling(state)
                }
            }
            RedHatBoyStateMachine::KnockedOut(_) => self,
        }
    }
}

#[derive(Clone)]
struct RedHatBoyState<S> {
    game_object: GameObject,
    _state: S,
}

#[derive(Copy, Clone)]
struct Idle;

impl RedHatBoyState<Idle> {
    fn new(audio: Audio, jump_sound: Sound) -> Self {
        RedHatBoyState {
            game_object: GameObject {
                frame: 0,
                position: Point {
                    x: STARTING_POINT,
                    y: FLOOR,
                },
                velocity: Point { x: 0, y: 0 },
                audio,
                jump_sound,
            },
            _state: Idle {},
        }
    }
}

#[derive(Copy, Clone)]
struct Running;

impl From<RedHatBoyState<Idle>> for RedHatBoyState<Running> {
    fn from(mut machine: RedHatBoyState<Idle>) -> Self {
        machine.game_object = machine.game_object.reset_frame().run_right();
        RedHatBoyState {
            game_object: machine.game_object,
            _state: Running {},
        }
    }
}

impl From<RedHatBoyState<Jumping>> for RedHatBoyState<Running> {
    fn from(mut machine: RedHatBoyState<Jumping>) -> Self {
        machine.game_object = machine.game_object.reset_frame();
        RedHatBoyState {
            game_object: machine.game_object,
            _state: Running {},
        }
    }
}

impl From<RedHatBoyState<Sliding>> for RedHatBoyState<Running> {
    fn from(mut machine: RedHatBoyState<Sliding>) -> Self {
        machine.game_object = machine.game_object.reset_frame();
        RedHatBoyState {
            game_object: machine.game_object,
            _state: Running {},
        }
    }
}

#[derive(Copy, Clone)]
struct Jumping;

impl From<RedHatBoyState<Running>> for RedHatBoyState<Jumping> {
    fn from(mut machine: RedHatBoyState<Running>) -> Self {
        machine.game_object = machine
            .game_object
            .reset_frame()
            .set_vertical_velocity(JUMP_SPEED)
            .play_jump_sound();
        RedHatBoyState {
            game_object: machine.game_object,
            _state: Jumping {},
        }
    }
}

#[derive(Copy, Clone)]
struct Sliding;

impl From<RedHatBoyState<Running>> for RedHatBoyState<Sliding> {
    fn from(mut machine: RedHatBoyState<Running>) -> Self {
        machine.game_object = machine.game_object.reset_frame();
        RedHatBoyState {
            game_object: machine.game_object,
            _state: Sliding {},
        }
    }
}

#[derive(Copy, Clone)]
struct Falling;

impl From<RedHatBoyState<Running>> for RedHatBoyState<Falling> {
    fn from(machine: RedHatBoyState<Running>) -> Self {
        RedHatBoyState {
            game_object: machine.game_object.reset_frame().stop(),
            _state: Falling {},
        }
    }
}

impl From<RedHatBoyState<Sliding>> for RedHatBoyState<Falling> {
    fn from(machine: RedHatBoyState<Sliding>) -> Self {
        RedHatBoyState {
            game_object: machine.game_object.reset_frame().stop(),
            _state: Falling {},
        }
    }
}

impl From<RedHatBoyState<Jumping>> for RedHatBoyState<Falling> {
    fn from(machine: RedHatBoyState<Jumping>) -> Self {
        RedHatBoyState {
            game_object: machine.game_object.reset_frame().stop(),
            _state: Falling {},
        }
    }
}

#[derive(Copy, Clone)]
struct KnockedOut;
impl From<RedHatBoyState<Falling>> for RedHatBoyState<KnockedOut> {
    fn from(machine: RedHatBoyState<Falling>) -> Self {
        RedHatBoyState {
            game_object: machine.game_object,
            _state: KnockedOut {},
        }
    }
}

#[derive(Clone)]
struct GameObject {
    frame: u8,
    position: Point,
    velocity: Point,
    audio: Audio,
    jump_sound: Sound,
}

impl GameObject {
    fn update(mut self, frame_count: u8) -> Self {
        if self.velocity.y < TERMINAL_VELOCITY {
            self.velocity.y += GRAVITY;
        }

        if self.frame < frame_count {
            self.frame += 1;
        } else {
            self.frame = 0;
        }

        self.position.y += self.velocity.y;

        if self.position.y > FLOOR {
            self.position.y = FLOOR;
        }

        self
    }

    fn reset_frame(mut self) -> Self {
        self.frame = 0;
        self
    }

    fn set_vertical_velocity(mut self, y: i16) -> Self {
        self.velocity.y = y;
        self
    }

    fn run_right(mut self) -> Self {
        self.velocity.x += RUNNING_SPEED;
        self
    }

    fn stop(mut self) -> Self {
        self.velocity.x = 0;
        self
    }

    fn set_on(mut self, position: i16) -> Self {
        self.position.y = position;
        self
    }

    fn play_jump_sound(self) -> Self {
        if let Err(err) = self.audio.play_sound(&self.jump_sound) {
            log!("Error playing jump sound {:#?}", err);
        }
        self
    }
}

struct Walk {
    obstacle_sheet: Rc<SpriteSheet>,
    stone: HtmlImageElement,
    boy: RedHatBoy,
    backgrounds: [Image; 2],
    obstacles: Vec<Box<dyn Obstacle>>,
    timeline: i16,
}

pub struct Barrier {
    image: Image,
}

impl Barrier {
    pub fn new(image: Image) -> Self {
        Barrier { image }
    }
}

impl Obstacle for Barrier {
    fn check_intersection(&self, boy: &mut RedHatBoy) {
        if boy.bounding_box().intersects(self.image.bounding_box()) {
            boy.kill()
        }
    }

    fn draw(&self, renderer: &Renderer) {
        self.image.draw(renderer);
    }

    fn move_horizontally(&mut self, x: i16) {
        self.image.move_horizontally(x);
    }

    fn right(&self) -> i16 {
        self.image.right()
    }
}

#[async_trait(?Send)]
impl Game for WalkTheDog {
    async fn initialize(self) -> Result<Box<dyn Game>> {
        match self.machine {
            None => {
                let rhb_sheet = browser::fetch_json("rhb.json").await?;
                let background = engine::load_image("BG.png").await?;
                let stone = engine::load_image("Stone.png").await?;

                let tiles = browser::fetch_json("tiles.json").await?;

                let sprite_sheet = Rc::new(SpriteSheet::new(
                    tiles.into_serde::<Sheet>()?,
                    engine::load_image("tiles.png").await?,
                ));

                let audio = Audio::new()?;
                let sound = audio.load_sound("SFX_Jump_23.mp3").await?;
                let background_music = audio.load_sound("background_song.mp3").await?;
                audio.play_looping_sound(&background_music)?;

                let rhb = RedHatBoy::new(
                    rhb_sheet.into_serde::<Sheet>()?,
                    engine::load_image("rhb.png").await?,
                    audio,
                    sound,
                );

                let background_width = background.width() as i16;
                let starting_obstacles = rock_and_platform(stone.clone(), sprite_sheet.clone(), 0);
                let timeline = rightmost(&starting_obstacles);

                let machine = WalkTheDogStateMachine::Ready(WalkTheDogState {
                    _state: Ready {
                        walk: Walk {
                            boy: rhb,
                            backgrounds: [
                                Image::new(background.clone(), Point { x: 0, y: 0 }),
                                Image::new(
                                    background,
                                    Point {
                                        x: background_width,
                                        y: 0,
                                    },
                                ),
                            ],
                            obstacles: starting_obstacles,
                            obstacle_sheet: sprite_sheet,
                            stone,
                            timeline,
                        },
                    },
                });

                Ok(Box::new(WalkTheDog {
                    machine: Some(machine),
                }))
            }
            Some(_) => Err(anyhow!("Error: Game is already initialized!")),
        }
    }

    fn update(&mut self, keystate: &KeyState) {
        if let Some(machine) = self.machine.take() {
            self.machine.replace(machine.update(keystate));
        }
        assert!(self.machine.is_some());
    }

    fn draw(&self, renderer: &Renderer) {
        renderer.clear(&Rect::new(Point { x: 0, y: 0 }, 600, 600));

        if let Some(machine) = &self.machine {
            match machine {
                WalkTheDogStateMachine::Ready(state) => {
                    state._state.walk.backgrounds.iter().for_each(|background| {
                        background.draw(renderer);
                    });
                    state._state.walk.boy.draw(renderer);

                    state._state.walk.obstacles.iter().for_each(|obstacle| {
                        obstacle.draw(renderer);
                    });
                }
                _ => log!("Doomed"),
            }
        }
    }
}

impl Walk {
    fn velocity(&self) -> i16 {
        -self.boy.walking_speed()
    }

    fn generate_next_segment(&mut self) {
        let mut rng = thread_rng();
        let next_segment = rng.gen_range(0..2);

        let mut next_obstacles = match next_segment {
            0 => rock_and_platform(
                self.stone.clone(),
                self.obstacle_sheet.clone(),
                self.timeline + 20,
            ),
            1 => platform_and_rock(
                self.stone.clone(),
                self.obstacle_sheet.clone(),
                self.timeline + 20,
            ),
            _ => vec![],
        };
        self.timeline = rightmost(&next_obstacles);
        self.obstacles.append(&mut next_obstacles);
    }
}

fn rightmost(obstacle_list: &Vec<Box<dyn Obstacle>>) -> i16 {
    obstacle_list
        .iter()
        .map(|obstacle| obstacle.right())
        .max_by(|x, y| x.cmp(&y))
        .unwrap_or(0)
}
