use winit::{
    dpi::PhysicalPosition,
    event::*,
};
use crate::input_code::*;
use std::collections::HashMap;
use std::{cmp::Eq, hash::Hash};
#[cfg(not(feature = "glium-types"))]
type Vec2 = (f32, f32);
#[cfg(feature = "glium-types")]
type Vec2 = glium_types::vectors::Vec2;
fn v(a: f32, b: f32) -> Vec2 {
    #[cfg(not(feature = "glium-types"))]
    { (a, b) }
    #[cfg(feature = "glium-types")]
    { Vec2::new(a, b) }
}


/// A struct that handles all your input needs once you've hooked it up to winit and gilrs.
/// ```
/// struct App {
///     window: Option<Window>,
///     input: InputMap<()>,
///     gilrs: Gilrs
/// }
/// impl ApplicationHandler for App {
///     fn resumed(&mut self, event_loop: &ActiveEventLoop) {
///         self.window = Some(event_loop.create_window(Window::default_attributes()).unwrap());
///     }
///     fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
///         self.input.update_with_window_event(&event);
///         if let WindowEvent::CloseRequested = &event { event_loop.exit() }
///     }
///     fn device_event(&mut self, _: &ActiveEventLoop, _: DeviceId, event: DeviceEvent) {
///         self.input.update_with_device_event(&event);
///     }
///     fn about_to_wait(&mut self, _: &ActiveEventLoop) {
///         self.input.update_with_gilrs(&mut self.gilrs);
/// 
///         // put your code here
///
///         self.input.init();
///     }
/// }
/// ```
pub struct InputMap<F: Hash + Eq + Clone + Copy> {
    /// Stores what each input code is bound to
    pub binds: HashMap<InputCode, Vec<F>>,
    /// f32 is current val, 1st bool is pressed and 2nd bool is released.
    action_val: HashMap<F, (f32, bool, bool)>,
    /// The mouse position
    pub mouse_pos: Vec2,
    /// The last input event, even if it isn't in the binds. Useful for handling rebinding
    pub recently_pressed: Option<InputCode>,
    /// The text typed this loop
    pub text_typed: Option<String>,
    /// Since most values are from 0-1 reducing the mouse sensitivity will result in better
    /// consistancy
    pub mouse_scale: f32,
    /// Since most values are from 0-1 reducing the scroll sensitivity will result in better
    /// consistancy
    pub scroll_scale: f32,
    /// The minimum value something has to be at to count as being pressed. Values over 1 will
    /// result in regular buttons being unusable
    pub press_sensitivity: f32
}
impl<F: Hash + Eq + Clone + Copy> Default for InputMap<F> {
    fn default() -> Self {
        Self {
            mouse_scale: 0.1,
            press_sensitivity: 0.5,
            scroll_scale:      0.1,
            mouse_pos: v(0.0, 0.0),
            recently_pressed: None,
            text_typed:    None,
            binds:      HashMap::<InputCode,    Vec<F>>::new(),
            action_val: HashMap::<F, (f32, bool, bool)>::new()
        }
    }
}
impl<F: Hash + Eq + Clone + Copy> InputMap<F> {
    /// Create new input system. It's recommended to use the `input_map!` macro to reduce boilerplate
    /// and increase readability.
    /// ```
    /// use Action::*;
    /// use winit_input_map::*;
    /// use winit::keyboard::KeyCode;
    /// #[derive(Hash, PartialEq, Eq, Clone, Copy)]
    /// enum Action {
    ///     Forward,
    ///     Back,
    ///     Pos,
    ///     Neg
    /// }
    /// //doesnt have to be the same ordered as the enum.
    /// let mut input = Input::new([
    ///     (vec![Input::keycode(KeyCode::KeyW)], Forward),
    ///     (vec![Input::keycode(KeyCode::KeyA)], Pos),
    ///     (vec![Input::keycode(KeyCode::KeyS)], Back),
    ///     (vec![Input::keycode(KeyCode::KeyD)], Neg)
    /// ]);
    /// ```
    pub fn new(binds: &[(F, Vec<InputCode>)]) -> Self {
        let mut result = Self::default();
        for (i, binds) in binds {
            for bind in binds {
                result.mut_bind(*bind).push(*i);
            }
        }
        result.binds.shrink_to_fit();
        result
    }
    /// Use if you dont want to have any actions and binds. Will still have access to everything else.
    pub fn empty() -> InputMap<()> {
        InputMap::<()>::default()
    }
    /// Gets a mutable vector of what actions input_code is bound to
    pub fn mut_bind(&mut self, input_code: InputCode) -> &mut Vec<F> {
        let has_val = self.binds.contains_key(&input_code);
        (if has_val { self.binds.get_mut(&input_code) } else {
            self.binds.insert(input_code, vec![]);
            self.binds.get_mut(&input_code)
        }).unwrap()
    }
    /// Updates the input map using a winit event. Make sure to call `input.init()` when your done with
    /// the input this loop.
    /// ```
    /// use winit::{event::*, window::Window, event_loop::EventLoop};
    /// use winit_input_map::InputMap;
    ///
    /// let mut event_loop = EventLoop::new().unwrap();
    /// event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    /// let _window = Window::new(&event_loop).unwrap();
    ///
    /// let mut input = input_map!();
    ///
    /// event_loop.run(|event, target|{
    ///     input.update_with_winit(&event);
    ///     match &event{
    ///         Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => target.exit(),
    ///         Event::AboutToWait => input.init(),
    ///         _ => ()
    ///     }
    /// });
    /// ```
    #[deprecated = "use `update_with_window_event` and `update_with_device_event`"]
    pub fn update_with_winit(&mut self, event: &Event<()>) {
        match event {
            Event::WindowEvent { event, .. } => self.update_with_window_event(event),
            Event::DeviceEvent { event, .. } => self.update_with_device_event(event),
            _ => ()
        }
    }
    pub fn update_with_device_event(&mut self, event: &DeviceEvent) {
        match event {
            DeviceEvent::MouseMotion { delta } => {
                let x = delta.0 as f32 * self.mouse_scale;
                let y = delta.1 as f32 * self.mouse_scale;
                self.modify_val(DeviceInput::MouseMoveX(AxisSign::Pos).into(), |v| *v += x.max(0.0));
                self.modify_val(DeviceInput::MouseMoveX(AxisSign::Neg).into(), |v| *v += (-x).max(0.0));
                self.modify_val(DeviceInput::MouseMoveY(AxisSign::Pos).into(), |v| *v += y.max(0.0));
                self.modify_val(DeviceInput::MouseMoveY(AxisSign::Neg).into(), |v| *v += (-y).max(0.0));
            },
            DeviceEvent::MouseWheel { delta } => {
                let (x, y) = match delta {
                    MouseScrollDelta::LineDelta(x, y) => (*x, *y),
                    MouseScrollDelta::PixelDelta(PhysicalPosition { x, y }) => (*x as f32, *y as f32)
                };
                let (x, y) = (x * self.mouse_scale, y * self.mouse_scale);
                self.modify_val(DeviceInput::MouseScroll(AxisSign::Pos ).into(), |v| *v += y.max(0.0));
                self.modify_val(DeviceInput::MouseScroll(AxisSign::Neg ).into(), |v| *v += (-y).max(0.0));
                self.modify_val(DeviceInput::MouseScrollX(AxisSign::Pos).into(), |v| *v += x.max(0.0));
                self.modify_val(DeviceInput::MouseScrollX(AxisSign::Neg).into(), |v| *v += (-x).max(0.0));
            },
             _ => (),
        }
    }
    pub fn update_with_window_event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::CursorMoved { position, .. } => self.update_mouse(*position),
            WindowEvent::MouseInput { state, button, .. } => self.update_buttons(state, *button),
            WindowEvent::KeyboardInput { event, .. } => self.update_keys(event),
            _ => ()
        }
    }
    #[cfg(feature = "gamepad")]
    pub fn update_with_gilrs(&mut self, gilrs: &mut gilrs::Gilrs) {
        while let Some(ev) = gilrs.next_event() {
            self.update_gamepad(ev);
        }
    }
    /// Makes the input map ready to recieve new events.
    pub fn init(&mut self) {
        self.update_val(DeviceInput::MouseMoveX(  AxisSign::Pos).into(), 0.0);
        self.update_val(DeviceInput::MouseMoveX(  AxisSign::Neg).into(), 0.0);
        self.update_val(DeviceInput::MouseMoveY(  AxisSign::Pos).into(), 0.0);
        self.update_val(DeviceInput::MouseMoveY(  AxisSign::Neg).into(), 0.0);
        self.update_val(DeviceInput::MouseScroll( AxisSign::Pos).into(), 0.0);
        self.update_val(DeviceInput::MouseScroll( AxisSign::Neg).into(), 0.0);
        self.update_val(DeviceInput::MouseScrollX(AxisSign::Pos).into(), 0.0);
        self.update_val(DeviceInput::MouseScrollX(AxisSign::Neg).into(), 0.0);
        self.action_val.iter_mut().for_each(|(_, i)|
            *i = (i.0, false, false)
        );
        self.recently_pressed = None;
        self.text_typed = None;
    }
    fn update_mouse(&mut self, position: PhysicalPosition<f64>) {
        self.mouse_pos = v(position.x as f32, position.y as f32);
    }
    fn update_keys(&mut self, event: &KeyEvent) {
        let input_code = event.physical_key.into();

        if let (Some(string), Some(new)) = (&mut self.text_typed, &event.text) {
            string.push_str(new);
        } else { self.text_typed = event.text.as_ref().map(|i| i.to_string()) }

        self.update_val(input_code, event.state.is_pressed() as u8 as f32);
    }
    fn update_buttons(&mut self, state: &ElementState, button: MouseButton) {
        let input_code = button.into();
        self.update_val(input_code, state.is_pressed() as u8 as f32);
    }
    /// updates provided input code
    fn update_val(&mut self, input_code: InputCode, val: f32) {
        let pressed = val >= self.press_sensitivity;
        if pressed { self.recently_pressed = Some(input_code) }
        if let Some(binds) = self.binds.get(&input_code) {
            for &action in binds {
                let jpressed = pressed && !self.pressing(action);
                let released = !pressed && self.pressing(action);
                self.action_val.insert(action, (val, jpressed, released));
            }
        }
    }
    fn modify_val<FN: Fn(&mut f32)>(&mut self, input_code: InputCode, f: FN) {
        if let Some(binds) = self.binds.get(&input_code) {
            for &action in binds {
                let mut val = self.action_val.get(&action)
                    .unwrap_or(&(0.0, false, false)).0;
                f(&mut val);
                let pressed = val >= self.press_sensitivity;
                if pressed { self.recently_pressed = Some(input_code) }
                
                let jpressed = pressed && !self.pressing(action);
                let released = !pressed && self.pressing(action);
                self.action_val.insert(action, (val, jpressed, released));
            }
        }
    }
    #[cfg(feature = "gamepad")]
    fn update_gamepad(&mut self, event: gilrs::Event) {
        let gilrs::Event { id, event, .. } = event;

        use gilrs::ev::EventType;
        match event {
            EventType::ButtonPressed(b, _) => {
                let a: GamepadInput = b.into();
                self.update_val(a.with_id(id), 1.0);
                self.update_val(b.into(),      1.0);
            },
            EventType::ButtonReleased(b, _) => {
                let a: GamepadInput = b.into();
                self.update_val(a.with_id(id), 0.0);
                self.update_val(b.into(),      0.0);
            },
            EventType::ButtonChanged(b, v, _) => {
                let a: GamepadInput = b.into();
                self.update_val(b.into(),                    v);
                self.update_val(a.with_id(id), v);
            },
            EventType::AxisChanged(b, v, _) => {
                let dir_pos = v.max(0.0);
                let dir_neg = (-v).max(0.0);
                let input_pos = InputCode::gamepad_axis_pos(b);
                let input_neg = InputCode::gamepad_axis_neg(b);

                self.update_val(input_pos,                    dir_pos);
                self.update_val(input_neg,                    dir_neg);
                self.update_val(input_pos.set_gamepad_id(id), dir_pos);
                self.update_val(input_neg.set_gamepad_id(id), dir_neg);
            }
            _ => ()
        }
    }
    /// Checks if action is being pressed currently. same as `input.action_val(action) >=
    /// input.press_sensitivity`
    pub fn pressing(&self, action: F) -> bool {
        self.action_val(action) >= self.press_sensitivity
    }
    /// Checks how much action is being pressed. May be higher than 1 in the case of scroll wheels
    /// and mouse movement.
    pub fn action_val(&self, action: F) -> f32 {
        if let Some(&(v, _, _)) = self.action_val.get(&action) { v } else {  0.0  }
    }
    /// checks if action was just pressed
    pub fn pressed(&self, action: F) -> bool {
        if let Some(&(_, v, _)) = self.action_val.get(&action) { v } else { false }
    }
    /// checks if action was just released
    pub fn released(&self, action: F) -> bool {
        if let Some(&(_, _, v)) = self.action_val.get(&action) { v } else { false }
    }
    /// Returns f32 based on how much pos and neg are pressed. may return values higher than 1.0 in
    /// the case of mouse movement and scrolling. usefull for movement controls. for 2d values see
    /// `[dir]` and `[dir_max_len_1]`
    /// ```
    /// let move_dir = input.axis(Neg, Pos);
    /// ```
    /// same as `input.action_val(pos) - input.action_val(neg)`
    pub fn axis(&self, pos: F, neg: F) -> f32 {
        self.action_val(pos) - self.action_val(neg)
    }
    /// Returns a vector based off of x and y axis. For movement controls see `dir_max_len_1`
    pub fn dir(&self, pos_x: F, neg_x: F, pos_y: F, neg_y: F) -> Vec2 {
        v(self.axis(pos_x, neg_x), self.axis(pos_y, neg_y))
    }
    /// Returns a vector based off of x and y axis with a maximum length of 1 (the same as a normalised
    /// vector). If this undesirable see `dir`
    pub fn dir_max_len_1(&self, pos_x: F, neg_x: F, pos_y: F, neg_y: F) -> Vec2 {
        let (x, y) = (self.axis(pos_x, neg_x), self.axis(pos_y, neg_y));
        // if lower than 1, set to 1. since x/1 = x, that means anything lower than 1 is left unchanged
        let length = (x*x + y*y).sqrt().max(1.0);
        v(x/length, y/length)
    }
}
