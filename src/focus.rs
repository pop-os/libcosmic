use iced::{Rectangle, window};
use iced_core::widget::operation::{Focusable, Outcome};
use iced_core::widget::{Operation, operation};
use iced_widget::Id;

/// A summary of the focusable widgets present on a widget tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Count {
    /// The index of the current focused widget, if any.
    pub focused: Option<usize>,

    /// The total amount of focusable widgets.
    pub total: usize,

    /// window id being operated on
    pub window_id: Vec<window::Id>,
}

impl Count {
    fn new(window_id: Vec<window::Id>) -> Self {
        Count {
            focused: None,
            total: 0,
            window_id,
        }
    }
}

/// Produces an [`Operation`] that generates a [`Count`] and chains it with the
/// provided function to build a new [`Operation`].
pub fn count(window_id: Vec<window::Id>) -> impl Operation<Count> {
    struct CountFocusable {
        count: Count,
        cur_window_id: window::Id,
    }

    impl Operation<Count> for CountFocusable {
        fn focusable(&mut self, _id: Option<&Id>, _bounds: Rectangle, state: &mut dyn Focusable) {
            if self.count.window_id.contains(&self.cur_window_id) || self.count.window_id.is_empty()
            {
                if state.is_focused() {
                    self.count.focused = Some(self.count.total);
                }

                self.count.total += 1;
            }
        }

        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<Count>)) {
            operate(self);
        }

        fn finish(&self) -> Outcome<Count> {
            Outcome::Some(self.count.clone())
        }

        fn set_window_id(&mut self, id: window::Id) {
            self.cur_window_id = id;
        }
    }

    CountFocusable {
        count: Count::new(window_id),
        cur_window_id: window::Id::NONE,
    }
}

/// Produces an [`Operation`] that searches for the current focused widget, and
/// - if found, focuses the previous focusable widget.
/// - if not found, focuses the last focusable widget.
pub fn focus_previous<T>(window_id: Vec<window::Id>) -> impl Operation<T>
where
    T: Send + 'static,
{
    struct FocusPrevious {
        count: Count,
        current: usize,
        cur_window_id: window::Id,
    }

    impl<T> Operation<T> for FocusPrevious {
        fn focusable(&mut self, _id: Option<&Id>, _bounds: Rectangle, state: &mut dyn Focusable) {
            if self.count.total == 0
                || (!self.count.window_id.contains(&self.cur_window_id)
                    && !self.count.window_id.is_empty())
            {
                return;
            }

            match self.count.focused {
                None if self.current == self.count.total - 1 => state.focus(),
                Some(0) if self.current == 0 && self.count.total == 1 => {}
                Some(0) if self.current == 0 => state.unfocus(),
                Some(0) if self.current == self.count.total - 1 => state.focus(),
                Some(0) => {}
                Some(focused) if focused == self.current => state.unfocus(),
                Some(focused) if focused - 1 == self.current => state.focus(),
                _ => {}
            }

            self.current += 1;
        }

        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<T>)) {
            operate(self);
        }

        fn set_window_id(&mut self, id: window::Id) {
            self.cur_window_id = id;
        }
    }

    operation::then(count(window_id), |count| FocusPrevious {
        count,
        current: 0,
        cur_window_id: window::Id::NONE,
    })
}

/// Produces an [`Operation`] that searches for the current focused widget, and
/// - if found, focuses the next focusable widget.
/// - if not found, focuses the first focusable widget.
pub fn focus_next<T>(window_id: Vec<window::Id>) -> impl Operation<T>
where
    T: Send + 'static,
{
    struct FocusNext {
        count: Count,
        current: usize,
        cur_window_id: window::Id,
    }

    impl<T> Operation<T> for FocusNext {
        fn focusable(&mut self, _id: Option<&Id>, _bounds: Rectangle, state: &mut dyn Focusable) {
            if self.count.total == 0
                || (!self.count.window_id.contains(&self.cur_window_id)
                    && !self.count.window_id.is_empty())
            {
                return;
            }

            match self.count.focused {
                None if self.current == 0 => state.focus(),
                Some(focused) if focused == self.current && self.count.total == 1 => {}
                Some(focused) if focused == self.current => state.unfocus(),
                Some(focused) if focused + 1 == self.current => state.focus(),
                Some(focused) if focused == self.count.total - 1 && self.current == 0 => {
                    state.focus()
                }
                _ => {}
            }

            self.current += 1;
        }

        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<T>)) {
            operate(self);
        }

        fn set_window_id(&mut self, id: window::Id) {
            self.cur_window_id = id;
        }
    }

    operation::then(count(window_id), move |count| FocusNext {
        count,
        current: 0,
        cur_window_id: window::Id::NONE,
    })
}
