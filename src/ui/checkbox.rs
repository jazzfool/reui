use {
    crate::{
        base::{self, Repaintable, Resizable},
        draw::{self, state},
        pipe,
        ui::ToggledEvent,
    },
    reclutch::{
        display::{CommandGroup, DisplayCommand, GraphicsDisplay, Point, Rect, Size},
        event::RcEventQueue,
        prelude::*,
    },
    std::marker::PhantomData,
};

/// Events emitted by a checkbox.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CheckboxEvent {
    /// Emitted when the checkbox is pressed or released.
    /// Corresponds to `WindowEvent::MousePress` or `WindowEvent::MouseRelease`.
    Press(ToggledEvent<Point>),
    /// Emitted when the button is checked/unchecked.
    /// Corresponds to `WindowEvent::MouseRelease`.
    Check(ToggledEvent<Point>),
    /// Emitted when the mouse enters (`true`) or leaves (`false`) the checkbox boundaries.
    /// Corresponds to `WindowEvent::MouseMove`.
    MouseHover(ToggledEvent<Point>),
    /// Emitted when focus is gained (`true`) or lost (`false`).
    Focus(ToggledEvent<()>),
}

impl pipe::Event for CheckboxEvent {
    fn get_key(&self) -> &'static str {
        match self {
            CheckboxEvent::Press(ToggledEvent::Start(_)) => "press",
            CheckboxEvent::Press(ToggledEvent::Stop(_)) => "release",
            CheckboxEvent::Check(ToggledEvent::Start(_)) => "check",
            CheckboxEvent::Check(ToggledEvent::Stop(_)) => "uncheck",
            CheckboxEvent::MouseHover(ToggledEvent::Start(_)) => "begin_hover",
            CheckboxEvent::MouseHover(ToggledEvent::Stop(_)) => "end_hover",
            CheckboxEvent::Focus(ToggledEvent::Start(_)) => "focus",
            CheckboxEvent::Focus(ToggledEvent::Stop(_)) => "blur",
        }
    }
}

/// Checkbox widget; useful for boolean input.
#[derive(WidgetChildren)]
#[widget_children_trait(base::WidgetChildren)]
pub struct Checkbox<U, G>
where
    U: base::UpdateAuxiliary + 'static,
    G: base::GraphicalAuxiliary + 'static,
{
    pub event_queue: RcEventQueue<CheckboxEvent>,

    pub checked: base::Observed<bool>,
    pub disabled: base::Observed<bool>,
    rect: Rect,

    command_group: CommandGroup,
    painter: Box<dyn draw::Painter<state::CheckboxState>>,
    layout: base::WidgetLayoutEvents,
    visibility: base::Visibility,
    interaction: state::InteractionState,
    drop_event: RcEventQueue<base::DropEvent>,
    pipe: Option<pipe::Pipeline<Self, U>>,

    phantom_u: PhantomData<U>,
    phantom_g: PhantomData<G>,
}

impl<U, G> Checkbox<U, G>
where
    U: base::UpdateAuxiliary + 'static,
    G: base::GraphicalAuxiliary + 'static,
{
    /// Creates a new checkbox with a specified checked state, disabled state, position and theme.
    pub fn new(
        checked: bool,
        disabled: bool,
        position: Point,
        theme: &dyn draw::Theme,
        u_aux: &mut U,
        g_aux: &mut G,
    ) -> Self {
        let temp_state = state::CheckboxState {
            rect: Default::default(),
            checked,
            state: state::ControlState::Normal(state::InteractionState::empty()),
        };

        let painter = theme.checkbox();
        let rect = Rect::new(position, painter.size_hint(temp_state, g_aux));

        let checked = base::Observed::new(checked);
        let disabled = base::Observed::new(disabled);

        let pipe = pipeline! {
            Self as obj,
            U as _aux,
            _ev in &checked.on_change => { change { obj.command_group.repaint(); } }
            _ev in &disabled.on_change => { change { obj.command_group.repaint(); } }
            event in u_aux.window_queue() => {
                mouse_press {
                    force_event!(event, base::WindowEvent::MousePress);

                    if let Some((pos, _)) = event.with(|(pos, button)| {
                        !*obj.disabled.get()
                            && *button == base::MouseButton::Left
                            && obj.bounds().contains(*pos)
                    }) {
                        obj.interaction.insert(state::InteractionState::PRESSED);
                        obj.event_queue.emit_owned(CheckboxEvent::Press(ToggledEvent::new(
                            true, *pos,
                        )));
                        obj.command_group.repaint();
                    }
                }
                mouse_release {
                    force_event!(event, base::WindowEvent::MouseRelease);

                    if let Some((pos, _)) = event.with(|(_, button)| {
                        !*obj.disabled.get()
                            && *button == base::MouseButton::Left
                            && obj.interaction.contains(state::InteractionState::PRESSED)
                    }) {
                        obj.interaction.remove(state::InteractionState::PRESSED);
                        obj.interaction.insert(state::InteractionState::FOCUSED);
                        obj.event_queue.emit_owned(CheckboxEvent::Press(ToggledEvent::new(
                            false, *pos,
                        )));

                        obj.checked.set(!*obj.checked.get());
                        obj.event_queue.emit_owned(CheckboxEvent::Press(ToggledEvent::new(
                            *obj.checked.get(),
                            *pos,
                        )));

                        obj.command_group.repaint();
                    }
                }
                mouse_move {
                    force_event!(event, base::WindowEvent::MouseMove);

                    if let Some(pos) = event.with(|pos| obj.bounds().contains(*pos)) {
                        if !obj.interaction.contains(state::InteractionState::HOVERED) {
                            obj.interaction.insert(state::InteractionState::HOVERED);
                            obj.event_queue.emit_owned(CheckboxEvent::MouseHover(
                                ToggledEvent::new(true, pos.clone()),
                            ));
                            obj.command_group.repaint();
                        }
                    } else if obj.interaction.contains(state::InteractionState::HOVERED) {
                        obj.interaction.remove(state::InteractionState::HOVERED);
                        obj.event_queue.emit_owned(CheckboxEvent::MouseHover(
                            ToggledEvent::new(false, event.get().clone()),
                        ));
                        obj.command_group.repaint();
                    }
                }
                clear_focus {
                    obj.interaction.remove(state::InteractionState::FOCUSED);
                }
            }
        };

        Checkbox {
            event_queue: Default::default(),

            checked,
            disabled,
            rect,

            command_group: Default::default(),
            painter,
            layout: Default::default(),
            visibility: Default::default(),
            interaction: state::InteractionState::empty(),
            drop_event: Default::default(),
            pipe: pipe.into(),

            phantom_u: Default::default(),
            phantom_g: Default::default(),
        }
    }

    fn derive_state(&self) -> state::CheckboxState {
        state::CheckboxState {
            rect: self.rect,
            checked: *self.checked.get(),
            state: if *self.disabled.get() {
                state::ControlState::Disabled
            } else {
                state::ControlState::Normal(self.interaction)
            },
        }
    }
}

impl<U, G> Widget for Checkbox<U, G>
where
    U: base::UpdateAuxiliary + 'static,
    G: base::GraphicalAuxiliary + 'static,
{
    type UpdateAux = U;
    type GraphicalAux = G;
    type DisplayObject = DisplayCommand;

    fn bounds(&self) -> Rect {
        self.painter.paint_hint(self.rect)
    }

    fn update(&mut self, aux: &mut U) {
        let was_focused = self.interaction.contains(state::InteractionState::FOCUSED);

        let mut pipe = self.pipe.take().unwrap();
        pipe.update(self, aux);
        self.pipe = Some(pipe);

        if was_focused != self.interaction.contains(state::InteractionState::FOCUSED) {
            self.command_group.repaint();
            self.event_queue
                .emit_owned(CheckboxEvent::Focus(ToggledEvent::new(!was_focused, ())));
        }

        if let Some(rect) = self.layout.receive() {
            self.rect = rect;
            self.command_group.repaint();
        }
    }

    fn draw(&mut self, display: &mut dyn GraphicsDisplay, aux: &mut G) {
        let state = self.derive_state();
        let painter = &mut self.painter;
        self.command_group
            .push_with(display, || painter.draw(state, aux), None, None);
    }
}

impl<U, G> base::LayableWidget for Checkbox<U, G>
where
    U: base::UpdateAuxiliary + 'static,
    G: base::GraphicalAuxiliary + 'static,
{
    #[inline]
    fn listen_to_layout(&mut self, layout: impl Into<Option<base::WidgetLayoutEventsInner>>) {
        self.layout.update(layout);
    }

    #[inline]
    fn layout_id(&self) -> Option<u64> {
        self.layout.id()
    }
}

impl<U, G> base::HasVisibility for Checkbox<U, G>
where
    U: base::UpdateAuxiliary + 'static,
    G: base::GraphicalAuxiliary + 'static,
{
    #[inline]
    fn set_visibility(&mut self, visibility: base::Visibility) {
        self.visibility = visibility
    }

    #[inline]
    fn visibility(&self) -> base::Visibility {
        self.visibility
    }
}

impl<U, G> Repaintable for Checkbox<U, G>
where
    U: base::UpdateAuxiliary + 'static,
    G: base::GraphicalAuxiliary + 'static,
{
    #[inline]
    fn repaint(&mut self) {
        self.command_group.repaint();
    }
}

impl<U, G> base::Movable for Checkbox<U, G>
where
    U: base::UpdateAuxiliary + 'static,
    G: base::GraphicalAuxiliary + 'static,
{
    fn set_position(&mut self, position: Point) {
        self.rect.origin = position;
        self.repaint();
        self.layout.notify(self.rect);
    }

    #[inline]
    fn position(&self) -> Point {
        self.rect.origin
    }
}

impl<U, G> Resizable for Checkbox<U, G>
where
    U: base::UpdateAuxiliary + 'static,
    G: base::GraphicalAuxiliary + 'static,
{
    fn set_size(&mut self, size: Size) {
        self.rect.size = size;
        self.repaint();
        self.layout.notify(self.rect);
    }

    #[inline]
    fn size(&self) -> Size {
        self.rect.size
    }
}

impl<U, G> draw::HasTheme for Checkbox<U, G>
where
    U: base::UpdateAuxiliary + 'static,
    G: base::GraphicalAuxiliary + 'static,
{
    #[inline]
    fn theme(&mut self) -> &mut dyn draw::Themed {
        &mut self.painter
    }

    fn resize_from_theme(&mut self, aux: &dyn base::GraphicalAuxiliary) {
        self.set_size(self.painter.size_hint(self.derive_state(), aux));
    }
}

impl<U, G> base::DropNotifier for Checkbox<U, G>
where
    U: base::UpdateAuxiliary + 'static,
    G: base::GraphicalAuxiliary + 'static,
{
    #[inline(always)]
    fn drop_event(&self) -> &RcEventQueue<base::DropEvent> {
        &self.drop_event
    }
}

impl<U, G> Drop for Checkbox<U, G>
where
    U: base::UpdateAuxiliary + 'static,
    G: base::GraphicalAuxiliary + 'static,
{
    fn drop(&mut self) {
        self.drop_event.emit_owned(base::DropEvent);
    }
}