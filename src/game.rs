use std::rc::Rc;

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use futures::channel::mpsc::UnboundedReceiver;
use rand::prelude::*;
use web_sys::HtmlImageElement;

use self::red_hat_boy_states::*;
use crate::{
    browser,
    engine::{
        self, Audio, Cell, Game, Image, KeyState, Point, Rect, Renderer, Sheet, Sound, SpriteSheet,
    },
    segments::*,
};

const HEIGHT: i16 = 600;
const TIMELINE_MINIMUM: i16 = 1000;
const OBSTACLE_BUFFER: i16 = 20;

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
    fn new(walk: Walk) -> Self {
        WalkTheDogStateMachine::Ready(WalkTheDogState::new(walk))
    }

    fn update(self, keystate: &KeyState) -> Self {
        match self {
            WalkTheDogStateMachine::Ready(state) => state.update(keystate).into(),
            WalkTheDogStateMachine::Walking(state) => state.update(keystate).into(),
            WalkTheDogStateMachine::GameOver(state) => state.update().into(),
        }
    }

    fn draw(&self, renderer: &Renderer) {
        match self {
            WalkTheDogStateMachine::Ready(state) => state.draw(renderer),
            WalkTheDogStateMachine::Walking(state) => state.draw(renderer),
            WalkTheDogStateMachine::GameOver(state) => state.draw(renderer),
        };
    }
}

struct WalkTheDogState<T> {
    _state: T,
    walk: Walk,
}

impl<T> WalkTheDogState<T> {
    fn draw(&self, renderer: &Renderer) {
        self.walk.draw(renderer);
    }
}

struct Ready;

impl WalkTheDogState<Ready> {
    fn new(walk: Walk) -> WalkTheDogState<Ready> {
        browser::draw_ui("<p id='score'>0</>").unwrap();
        WalkTheDogState {
            _state: Ready,
            walk,
        }
    }

    fn run_right(&mut self) {
        self.walk.boy.run_right();
    }

    fn start_running(mut self) -> WalkTheDogState<Walking> {
        self.run_right();

        WalkTheDogState {
            _state: Walking,
            walk: self.walk,
        }
    }

    fn update(mut self, keystate: &KeyState) -> ReadyEndState {
        self.walk.boy.update();
        if keystate.is_pressed("ArrowRight") {
            ReadyEndState::Complete(self.start_running())
        } else {
            ReadyEndState::Continue(self)
        }
    }
}

enum ReadyEndState {
    Complete(WalkTheDogState<Walking>),
    Continue(WalkTheDogState<Ready>),
}

impl From<ReadyEndState> for WalkTheDogStateMachine {
    fn from(state: ReadyEndState) -> Self {
        match state {
            ReadyEndState::Complete(walking) => walking.into(),
            ReadyEndState::Continue(ready) => ready.into(),
        }
    }
}

struct Walking;

impl WalkTheDogState<Walking> {
    fn end_game(self) -> WalkTheDogState<GameOver> {
        let receiver = browser::draw_ui("<button id='new_game'>New Game</button>")
            .and_then(|_unit| browser::find_html_element_by_id("new_game"))
            .map(engine::add_click_handler)
            .unwrap();

        WalkTheDogState {
            _state: GameOver {
                new_game_event: receiver,
            },

            walk: self.walk,
        }
    }

    fn update(mut self, keystate: &KeyState) -> WalkingEndState {
        if keystate.is_pressed("Space") {
            self.walk.boy.jump();
        }

        if keystate.is_pressed("ArrowDown") {
            self.walk.boy.slide();
        }

        self.walk.boy.update();

        let walking_speed = self.walk.velocity();
        let [first_background, second_background] = &mut self.walk.backgrounds;
        first_background.move_horizontally(walking_speed);
        second_background.move_horizontally(walking_speed);

        if first_background.right() < 0 {
            first_background.set_x(second_background.right());
        }
        if second_background.right() < 0 {
            second_background.set_x(first_background.right());
        }

        self.walk.obstacles.retain(|obstacle| obstacle.right() > 0);

        self.walk.obstacles.iter_mut().for_each(|obstacle| {
            obstacle.move_horizontally(walking_speed);
            obstacle.check_intersection(&mut self.walk.boy);
        });

        if self.walk.timeline < TIMELINE_MINIMUM {
            self.walk.generate_next_segment();
        } else {
            self.walk.timeline += walking_speed;
        }

        self.walk.score += 1;

        if self.walk.knocked_out() {
            WalkingEndState::Complete(self.end_game())
        } else {
            WalkingEndState::Continue(self)
        }
    }
}

enum WalkingEndState {
    Continue(WalkTheDogState<Walking>),
    Complete(WalkTheDogState<GameOver>),
}

impl From<WalkingEndState> for WalkTheDogStateMachine {
    fn from(state: WalkingEndState) -> Self {
        match state {
            WalkingEndState::Continue(walking) => walking.into(),
            WalkingEndState::Complete(game_over) => game_over.into(),
        }
    }
}

struct GameOver {
    new_game_event: UnboundedReceiver<()>,
}

impl WalkTheDogState<GameOver> {
    fn update(mut self) -> GameOverEndState {
        if self._state.new_game_pressed() {
            GameOverEndState::Complete(self.new_game())
        } else {
            GameOverEndState::Continue(self)
        }
    }

    fn new_game(self) -> WalkTheDogState<Ready> {
        if let Err(err) = browser::hide_ui() {
            error!("Error hiding the browser {:#?}", err);
        }

        WalkTheDogState {
            _state: Ready,
            walk: Walk::reset(self.walk),
        }
    }
}

enum GameOverEndState {
    Continue(WalkTheDogState<GameOver>),
    Complete(WalkTheDogState<Ready>),
}

impl From<GameOverEndState> for WalkTheDogStateMachine {
    fn from(state: GameOverEndState) -> Self {
        match state {
            GameOverEndState::Continue(game_over) => game_over.into(),
            GameOverEndState::Complete(ready) => ready.into(),
        }
    }
}

impl GameOver {
    fn new_game_pressed(&mut self) -> bool {
        matches!(self.new_game_event.try_next(), Ok(Some(())))
    }
}

impl From<WalkTheDogState<Walking>> for WalkTheDogStateMachine {
    fn from(state: WalkTheDogState<Walking>) -> Self {
        WalkTheDogStateMachine::Walking(state)
    }
}

impl From<WalkTheDogState<GameOver>> for WalkTheDogStateMachine {
    fn from(state: WalkTheDogState<GameOver>) -> Self {
        WalkTheDogStateMachine::GameOver(state)
    }
}

impl From<WalkTheDogState<Ready>> for WalkTheDogStateMachine {
    fn from(state: WalkTheDogState<Ready>) -> Self {
        WalkTheDogStateMachine::Ready(state)
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
    sprites: Vec<Cell>,
    bounding_boxes: Vec<Rect>,
    position: Point,
}

impl Platform {
    pub fn new(
        sheet: Rc<SpriteSheet>,
        position: Point,
        sprite_names: &[&str],
        bounding_boxes: &[Rect],
    ) -> Self {
        let sprites = sprite_names
            .iter()
            .filter_map(|sprite_name| sheet.cell(sprite_name).cloned())
            .collect();

        let bounding_boxes = bounding_boxes
            .iter()
            .map(|bounding_box| {
                Rect::new_from_x_y(
                    bounding_box.x() + position.x,
                    bounding_box.y() + position.y,
                    bounding_box.width,
                    bounding_box.height,
                )
            })
            .collect();

        Platform {
            sheet,
            position,
            sprites,
            bounding_boxes,
        }
    }

    fn bounding_boxes(&self) -> &Vec<Rect> {
        &self.bounding_boxes
    }
}

impl Obstacle for Platform {
    fn check_intersection(&self, boy: &mut RedHatBoy) {
        if let Some(box_to_land_on) = self
            .bounding_boxes()
            .iter()
            .find(|&bounding_box| boy.bounding_box().intersects(bounding_box))
        {
            if boy.velocity_y() > 0 && boy.pos_y() < self.position.y {
                boy.land_on(box_to_land_on.y());
            } else {
                boy.knock_out();
            }
        }
    }

    fn draw(&self, renderer: &Renderer) {
        let mut x = 0;
        self.sprites.iter().for_each(|sprite| {
            self.sheet.draw(
                renderer,
                &Rect::new_from_x_y(
                    sprite.frame.x,
                    sprite.frame.y,
                    sprite.frame.w,
                    sprite.frame.h,
                ),
                // Just use position and the standard widths in the tileset
                &Rect::new_from_x_y(
                    self.position.x + x,
                    self.position.y,
                    sprite.frame.w,
                    sprite.frame.h,
                ),
            );
            x += sprite.frame.w;
        });
    }

    fn move_horizontally(&mut self, x: i16) {
        self.position.x += x;
        self.bounding_boxes.iter_mut().for_each(|bounding_box| {
            bounding_box.set_x(bounding_box.position.x + x);
        })
    }

    fn right(&self) -> i16 {
        self.bounding_boxes()
            .last()
            .unwrap_or(&Rect::default())
            .right()
    }
}

pub struct RedHatBoy {
    state_machine: RedHatBoyStateMachine,
    sprite_sheet: Sheet,
    image: HtmlImageElement,
}

impl RedHatBoy {
    fn new(sprite_sheet: Sheet, image: HtmlImageElement, audio: Audio, sound: Sound) -> Self {
        RedHatBoy {
            state_machine: RedHatBoyStateMachine::Idle(RedHatBoyState::new(audio, sound)),
            sprite_sheet,
            image,
        }
    }

    fn reset(boy: Self) -> Self {
        RedHatBoy::new(
            boy.sprite_sheet,
            boy.image,
            boy.state_machine.context().audio.clone(),
            boy.state_machine.context().jump_sound.clone(),
        )
    }

    fn run_right(&mut self) {
        self.state_machine = self.state_machine.clone().transition(Event::Run);
    }

    fn slide(&mut self) {
        self.state_machine = self.state_machine.clone().transition(Event::Slide);
    }

    fn jump(&mut self) {
        self.state_machine = self.state_machine.clone().transition(Event::Jump);
    }

    fn knock_out(&mut self) {
        self.state_machine = self.state_machine.clone().transition(Event::KnockOut);
    }

    fn land_on(&mut self, position: i16) {
        self.state_machine = self.state_machine.clone().transition(Event::Land(position));
    }

    fn update(&mut self) {
        self.state_machine = self.state_machine.clone().update();
    }

    fn knocked_out(&self) -> bool {
        self.state_machine.knocked_out()
    }

    fn pos_y(&self) -> i16 {
        self.state_machine.context().position.y
    }

    fn velocity_y(&self) -> i16 {
        self.state_machine.context().velocity.y
    }

    fn walking_speed(&self) -> i16 {
        self.state_machine.context().velocity.x
    }

    fn frame_name(&self) -> String {
        format!(
            "{} ({}).png",
            self.state_machine.frame_name(),
            (self.state_machine.context().frame / 3) + 1
        )
    }

    fn current_sprite(&self) -> Option<&Cell> {
        self.sprite_sheet.frames.get(&self.frame_name())
    }

    fn bounding_box(&self) -> Rect {
        const X_OFFSET: i16 = 18;
        const Y_OFFSET: i16 = 14;
        const WIDTH_OFFSET: i16 = 28;
        Rect::new_from_x_y(
            self.destination_box().x() + X_OFFSET,
            self.destination_box().y() + Y_OFFSET,
            self.destination_box().width - WIDTH_OFFSET,
            self.destination_box().height - Y_OFFSET,
        )
    }

    fn destination_box(&self) -> Rect {
        let sprite = self.current_sprite().expect("Cell not found");

        Rect::new_from_x_y(
            self.state_machine.context().position.x + sprite.sprite_source_size.x,
            self.state_machine.context().position.y + sprite.sprite_source_size.y,
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
            &self.destination_box(),
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

pub enum Event {
    Run,
    Jump,
    Slide,
    KnockOut,
    Land(i16),
    Update,
}

impl RedHatBoyStateMachine {
    fn transition(self, event: Event) -> Self {
        match (self.clone(), event) {
            (RedHatBoyStateMachine::Idle(state), Event::Run) => state.run().into(),
            (RedHatBoyStateMachine::Running(state), Event::Jump) => state.jump().into(),
            (RedHatBoyStateMachine::Running(state), Event::Slide) => state.slide().into(),
            (RedHatBoyStateMachine::Running(state), Event::KnockOut) => state.knock_out().into(),
            (RedHatBoyStateMachine::Running(state), Event::Land(position)) => {
                state.land_on(position).into()
            }
            (RedHatBoyStateMachine::Jumping(state), Event::Land(position)) => {
                state.land_on(position).into()
            }
            (RedHatBoyStateMachine::Jumping(state), Event::KnockOut) => state.knock_out().into(),
            (RedHatBoyStateMachine::Sliding(state), Event::KnockOut) => state.knock_out().into(),
            (RedHatBoyStateMachine::Sliding(state), Event::Land(position)) => {
                state.land_on(position).into()
            }
            (RedHatBoyStateMachine::Idle(state), Event::Update) => state.update().into(),
            (RedHatBoyStateMachine::Running(state), Event::Update) => state.update().into(),
            (RedHatBoyStateMachine::Jumping(state), Event::Update) => state.update().into(),
            (RedHatBoyStateMachine::Sliding(state), Event::Update) => state.update().into(),
            (RedHatBoyStateMachine::Falling(state), Event::Update) => state.update().into(),
            _ => self,
        }
    }

    fn frame_name(&self) -> &str {
        match self {
            RedHatBoyStateMachine::Idle(state) => state.frame_name(),
            RedHatBoyStateMachine::Running(state) => state.frame_name(),
            RedHatBoyStateMachine::Jumping(state) => state.frame_name(),
            RedHatBoyStateMachine::Sliding(state) => state.frame_name(),
            RedHatBoyStateMachine::Falling(state) => state.frame_name(),
            RedHatBoyStateMachine::KnockedOut(state) => state.frame_name(),
        }
    }

    fn context(&self) -> &RedHatBoyContext {
        match self {
            RedHatBoyStateMachine::Idle(state) => state.context(),
            RedHatBoyStateMachine::Running(state) => state.context(),
            RedHatBoyStateMachine::Jumping(state) => state.context(),
            RedHatBoyStateMachine::Sliding(state) => state.context(),
            RedHatBoyStateMachine::Falling(state) => state.context(),
            RedHatBoyStateMachine::KnockedOut(state) => state.context(),
        }
    }

    fn knocked_out(&self) -> bool {
        matches!(self, RedHatBoyStateMachine::KnockedOut(_))
    }

    fn update(self) -> Self {
        self.transition(Event::Update)
    }
}

impl From<RedHatBoyState<Idle>> for RedHatBoyStateMachine {
    fn from(state: RedHatBoyState<Idle>) -> Self {
        RedHatBoyStateMachine::Idle(state)
    }
}

impl From<RedHatBoyState<Running>> for RedHatBoyStateMachine {
    fn from(state: RedHatBoyState<Running>) -> Self {
        RedHatBoyStateMachine::Running(state)
    }
}

impl From<RedHatBoyState<Sliding>> for RedHatBoyStateMachine {
    fn from(state: RedHatBoyState<Sliding>) -> Self {
        RedHatBoyStateMachine::Sliding(state)
    }
}

impl From<RedHatBoyState<Jumping>> for RedHatBoyStateMachine {
    fn from(state: RedHatBoyState<Jumping>) -> Self {
        RedHatBoyStateMachine::Jumping(state)
    }
}

impl From<RedHatBoyState<Falling>> for RedHatBoyStateMachine {
    fn from(state: RedHatBoyState<Falling>) -> Self {
        RedHatBoyStateMachine::Falling(state)
    }
}

impl From<RedHatBoyState<KnockedOut>> for RedHatBoyStateMachine {
    fn from(state: RedHatBoyState<KnockedOut>) -> Self {
        RedHatBoyStateMachine::KnockedOut(state)
    }
}

impl From<SlidingEndState> for RedHatBoyStateMachine {
    fn from(state: SlidingEndState) -> Self {
        match state {
            SlidingEndState::Sliding(sliding) => sliding.into(),
            SlidingEndState::Running(running) => running.into(),
        }
    }
}

impl From<JumpingEndState> for RedHatBoyStateMachine {
    fn from(state: JumpingEndState) -> Self {
        match state {
            JumpingEndState::Jumping(jumping) => jumping.into(),
            JumpingEndState::Landing(landing) => landing.into(),
        }
    }
}

impl From<FallingEndState> for RedHatBoyStateMachine {
    fn from(state: FallingEndState) -> Self {
        match state {
            FallingEndState::Falling(falling) => falling.into(),
            FallingEndState::KnockedOut(knocked_out) => knocked_out.into(),
        }
    }
}

mod red_hat_boy_states {
    use super::{Audio, Sound, HEIGHT};
    use crate::engine::Point;

    const FLOOR: i16 = 479;
    const PLAYER_HEIGHT: i16 = HEIGHT - FLOOR;
    const RUNNING_SPEED: i16 = 4;
    const STARTING_POINT: i16 = -20;
    const IDLE_FRAMES: u8 = 29;
    const RUNNING_FRAMES: u8 = 23;
    const JUMPING_FRAMES: u8 = 35;
    const SLIDING_FRAMES: u8 = 14;
    const FALLING_FRAMES: u8 = 29;
    const IDLE_FRAME_NAME: &str = "Idle";
    const RUN_FRAME_NAME: &str = "Run";
    const SLIDING_FRAME_NAME: &str = "Slide";
    const JUMPING_FRAME_NAME: &str = "Jump";
    const FALLING_FRAME_NAME: &str = "Dead";
    const JUMP_SPEED: i16 = -25;
    const GRAVITY: i16 = 1;
    const TERMINAL_VELOCITY: i16 = 20;

    #[derive(Clone)]
    pub struct RedHatBoyState<S> {
        context: RedHatBoyContext,
        _state: S,
    }

    impl<S> RedHatBoyState<S> {
        pub fn context(&self) -> &RedHatBoyContext {
            &self.context
        }

        fn update_context(&mut self, frames: u8) {
            self.context = self.context.clone().update(frames);
        }
    }

    #[derive(Copy, Clone)]
    pub struct Idle;

    impl RedHatBoyState<Idle> {
        pub fn new(audio: Audio, jump_sound: Sound) -> Self {
            RedHatBoyState {
                context: RedHatBoyContext {
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

        pub fn frame_name(&self) -> &str {
            IDLE_FRAME_NAME
        }

        pub fn update(mut self) -> RedHatBoyState<Idle> {
            self.update_context(IDLE_FRAMES);
            self
        }

        pub fn run(self) -> RedHatBoyState<Running> {
            RedHatBoyState {
                context: self.context.reset_frame().run_right(),
                _state: Running {},
            }
        }
    }

    #[derive(Copy, Clone)]
    pub struct Running;

    impl RedHatBoyState<Running> {
        pub fn frame_name(&self) -> &str {
            RUN_FRAME_NAME
        }

        pub fn update(mut self) -> RedHatBoyState<Running> {
            self.update_context(RUNNING_FRAMES);
            self
        }

        pub fn jump(self) -> RedHatBoyState<Jumping> {
            RedHatBoyState {
                context: self
                    .context
                    .reset_frame()
                    .set_vertical_velocity(JUMP_SPEED)
                    .play_jump_sound(),
                _state: Jumping {},
            }
        }

        pub fn slide(self) -> RedHatBoyState<Sliding> {
            RedHatBoyState {
                context: self.context.reset_frame(),
                _state: Sliding {},
            }
        }

        pub fn knock_out(self) -> RedHatBoyState<Falling> {
            RedHatBoyState {
                context: self.context.reset_frame().stop(),
                _state: Falling {},
            }
        }

        pub fn land_on(self, position: i16) -> RedHatBoyState<Running> {
            RedHatBoyState {
                context: self.context.set_on(position),
                _state: Running {},
            }
        }
    }

    #[derive(Copy, Clone)]
    pub struct Jumping;

    pub enum JumpingEndState {
        Jumping(RedHatBoyState<Jumping>),
        Landing(RedHatBoyState<Running>),
    }

    impl RedHatBoyState<Jumping> {
        pub fn frame_name(&self) -> &str {
            JUMPING_FRAME_NAME
        }

        pub fn knock_out(self) -> RedHatBoyState<Falling> {
            RedHatBoyState {
                context: self.context.reset_frame().stop(),
                _state: Falling {},
            }
        }

        pub fn update(mut self) -> JumpingEndState {
            self.update_context(JUMPING_FRAMES);

            if self.context.position.y >= FLOOR {
                JumpingEndState::Landing(self.land_on(HEIGHT))
            } else {
                JumpingEndState::Jumping(self)
            }
        }

        pub fn land_on(self, position: i16) -> RedHatBoyState<Running> {
            RedHatBoyState {
                context: self.context.reset_frame().set_on(position),
                _state: Running,
            }
        }
    }

    #[derive(Copy, Clone)]
    pub struct Sliding;

    pub enum SlidingEndState {
        Sliding(RedHatBoyState<Sliding>),
        Running(RedHatBoyState<Running>),
    }

    impl RedHatBoyState<Sliding> {
        pub fn frame_name(&self) -> &str {
            SLIDING_FRAME_NAME
        }

        pub fn stand(self) -> RedHatBoyState<Running> {
            RedHatBoyState {
                context: self.context.reset_frame(),
                _state: Running {},
            }
        }

        pub fn knock_out(self) -> RedHatBoyState<Falling> {
            RedHatBoyState {
                context: self.context.reset_frame().stop(),
                _state: Falling {},
            }
        }

        pub fn update(mut self) -> SlidingEndState {
            self.update_context(SLIDING_FRAMES);

            if self.context.frame >= SLIDING_FRAMES {
                SlidingEndState::Running(self.stand())
            } else {
                SlidingEndState::Sliding(self)
            }
        }

        pub fn land_on(self, position: i16) -> RedHatBoyState<Sliding> {
            RedHatBoyState {
                context: self.context.set_on(position),
                _state: Sliding {},
            }
        }
    }

    #[derive(Copy, Clone)]
    pub struct Falling;

    impl RedHatBoyState<Falling> {
        pub fn frame_name(&self) -> &str {
            FALLING_FRAME_NAME
        }

        pub fn knock_out(self) -> RedHatBoyState<KnockedOut> {
            RedHatBoyState {
                context: self.context,
                _state: KnockedOut {},
            }
        }

        pub fn update(mut self) -> FallingEndState {
            self.update_context(FALLING_FRAMES);
            if self.context.frame >= FALLING_FRAMES {
                FallingEndState::KnockedOut(self.knock_out())
            } else {
                FallingEndState::Falling(self)
            }
        }
    }

    pub enum FallingEndState {
        KnockedOut(RedHatBoyState<KnockedOut>),
        Falling(RedHatBoyState<Falling>),
    }

    #[derive(Copy, Clone)]
    pub struct KnockedOut;

    impl RedHatBoyState<KnockedOut> {
        pub fn frame_name(&self) -> &str {
            FALLING_FRAME_NAME
        }
    }

    #[derive(Clone)]
    pub struct RedHatBoyContext {
        pub frame: u8,
        pub position: Point,
        pub velocity: Point,
        pub audio: Audio,
        pub jump_sound: Sound,
    }

    impl RedHatBoyContext {
        pub fn update(mut self, frame_count: u8) -> Self {
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
            self.velocity.y = 0;
            self
        }

        fn set_on(mut self, position: i16) -> Self {
            let position = position - PLAYER_HEIGHT;
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
}

pub struct Walk {
    obstacle_sheet: Rc<SpriteSheet>,
    stone: HtmlImageElement,
    boy: RedHatBoy,
    backgrounds: [Image; 2],
    obstacles: Vec<Box<dyn Obstacle>>,
    timeline: i16,
    score: u16,
}

impl Walk {
    fn knocked_out(&self) -> bool {
        self.boy.knocked_out()
    }

    fn reset(walk: Self) -> Self {
        let starting_obstacles =
            stone_and_platform(walk.stone.clone(), walk.obstacle_sheet.clone(), 0);
        let timeline = rightmost(&starting_obstacles);

        Walk {
            boy: RedHatBoy::reset(walk.boy),
            backgrounds: walk.backgrounds,
            obstacles: starting_obstacles,
            obstacle_sheet: walk.obstacle_sheet,
            stone: walk.stone,
            score: 0,
            timeline,
        }
    }

    fn draw(&self, renderer: &Renderer) {
        self.backgrounds.iter().for_each(|background| {
            background.draw(renderer);
        });
        self.boy.draw(renderer);

        self.obstacles.iter().for_each(|obstacle| {
            obstacle.draw(renderer);
        });
        browser::find_html_element_by_id("score")
            .map(|element| element.set_inner_html(&format!("Score: {}", self.score)))
            .unwrap();
    }

    fn velocity(&self) -> i16 {
        -self.boy.walking_speed()
    }

    fn generate_next_segment(&mut self) {
        let mut rng = thread_rng();
        let next_segment = rng.gen_range(0..2);

        let mut next_obstacles = match next_segment {
            0 => stone_and_platform(
                self.stone.clone(),
                self.obstacle_sheet.clone(),
                self.timeline + OBSTACLE_BUFFER,
            ),
            1 => platform_and_stone(
                self.stone.clone(),
                self.obstacle_sheet.clone(),
                self.timeline + OBSTACLE_BUFFER,
            ),
            _ => vec![],
        };
        self.timeline = rightmost(&next_obstacles);
        self.obstacles.append(&mut next_obstacles);
    }
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
            boy.knock_out()
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
    async fn initialize(&self) -> Result<Box<dyn Game>> {
        match self.machine {
            None => {
                let sheet = browser::fetch_json("rhb.json")
                    .await?
                    .into_serde::<Sheet>()?;
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

                let rhb = RedHatBoy::new(sheet, engine::load_image("rhb.png").await?, audio, sound);

                let background_width = background.width() as i16;
                let starting_obstacles = stone_and_platform(stone.clone(), sprite_sheet.clone(), 0);
                let timeline = rightmost(&starting_obstacles);

                let machine = WalkTheDogStateMachine::new(Walk {
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
                    score: 0,
                    stone,
                    timeline,
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
        renderer.clear(&Rect::new(Point { x: 0, y: 0 }, 600, HEIGHT));

        if let Some(machine) = &self.machine {
            machine.draw(renderer);
        }
    }
}

fn rightmost(obstacle_list: &[Box<dyn Obstacle>]) -> i16 {
    obstacle_list
        .iter()
        .map(|obstacle| obstacle.right())
        .max_by(|x, y| x.cmp(y))
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::channel::mpsc::unbounded;
    use std::collections::HashMap;
    use web_sys::{AudioBuffer, AudioBufferOptions};

    use wasm_bindgen_test::wasm_bindgen_test;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn test_transition_from_game_over_to_new_game() {
        let (_, receiver) = unbounded();
        let image = HtmlImageElement::new().unwrap();
        let audio = Audio::new().unwrap();
        let options = AudioBufferOptions::new(1, 3000.0);
        let sound = Sound {
            buffer: AudioBuffer::new(&options).unwrap(),
        };
        let rhb = RedHatBoy::new(
            Sheet {
                frames: HashMap::new(),
            },
            image.clone(),
            audio,
            sound,
        );
        let sprite_sheet = SpriteSheet::new(
            Sheet {
                frames: HashMap::new(),
            },
            image.clone(),
        );
        let walk = Walk {
            boy: rhb,
            backgrounds: [
                Image::new(image.clone(), Point { x: 0, y: 0 }),
                Image::new(image.clone(), Point { x: 0, y: 0 }),
            ],
            obstacles: vec![],
            obstacle_sheet: Rc::new(sprite_sheet),
            stone: image.clone(),
            score: 0,
            timeline: 0,
        };

        let document = browser::document().unwrap();
        document
            .body()
            .unwrap()
            .insert_adjacent_html("afterbegin", "<div id='ui'></div>")
            .unwrap();
        browser::draw_ui("<p>This is the UI</p>").unwrap();
        let state = WalkTheDogState {
            _state: GameOver {
                new_game_event: receiver,
            },
            walk: walk,
        };

        state.new_game();

        let ui = browser::find_html_element_by_id("ui").unwrap();
        assert_eq!(ui.child_element_count(), 0);
    }
}
