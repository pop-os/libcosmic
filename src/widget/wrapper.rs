use std::{
    borrow::Borrow,
    cell::RefCell,
    rc::Rc,
    thread::{self, ThreadId},
};

use crate::Element;
use iced::{Length, Rectangle, Size, event};
use iced_core::{Widget, id::Id, widget, widget::tree};

#[derive(Debug)]
pub struct RcWrapper<T> {
    pub(crate) data: Rc<RefCell<T>>,
    pub(crate) thread_id: ThreadId,
}

impl<T: Default> Default for RcWrapper<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T> Clone for RcWrapper<T> {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
            thread_id: self.thread_id,
        }
    }
}

unsafe impl<M: 'static> Send for RcWrapper<M> {}
unsafe impl<M: 'static> Sync for RcWrapper<M> {}

impl<T> RcWrapper<T> {
    pub fn new(element: T) -> Self {
        Self {
            data: Rc::new(RefCell::new(element)),
            thread_id: thread::current().id(),
        }
    }

    /// # Panics
    ///
    /// Will panic if used outside of original thread.
    pub fn with_data<O>(&self, f: impl FnOnce(&T) -> O) -> O {
        assert_eq!(self.thread_id, thread::current().id());
        let my_ref: &T = &RefCell::borrow(self.data.as_ref());
        f(my_ref)
    }

    /// # Panics
    ///
    /// Will panic if used outside of original thread.
    pub fn with_data_mut<O>(&self, f: impl FnOnce(&mut T) -> O) -> O {
        assert_eq!(self.thread_id, thread::current().id());
        let my_refmut: &mut T = &mut RefCell::borrow_mut(self.data.as_ref());
        f(my_refmut)
    }

    /// # Panics
    ///
    /// Will panic if used outside of original thread.
    pub(crate) unsafe fn as_ptr(&self) -> *mut T {
        assert_eq!(self.thread_id, thread::current().id());
        RefCell::as_ptr(self.data.as_ref())
    }
}

#[derive(Clone)]
pub struct RcElementWrapper<M> {
    pub(crate) element: RcWrapper<Element<'static, M>>,
}

impl<M> RcElementWrapper<M> {
    #[must_use]
    pub fn new(element: Element<'static, M>) -> Self {
        RcElementWrapper {
            element: RcWrapper::new(element),
        }
    }
}

impl<M: 'static> Borrow<dyn Widget<M, crate::Theme, crate::Renderer>> for RcElementWrapper<M> {
    fn borrow(&self) -> &(dyn Widget<M, crate::Theme, crate::Renderer> + 'static) {
        self
    }
}

impl<M> Widget<M, crate::Theme, crate::Renderer> for RcElementWrapper<M> {
    fn size(&self) -> Size<Length> {
        self.element.with_data(|e| e.as_widget().size())
    }

    fn size_hint(&self) -> Size<Length> {
        self.element.with_data(move |e| e.as_widget().size_hint())
    }

    fn layout(
        &self,
        tree: &mut tree::Tree,
        renderer: &crate::Renderer,
        limits: &crate::iced_core::layout::Limits,
    ) -> crate::iced_core::layout::Node {
        self.element
            .with_data_mut(|e| e.as_widget_mut().layout(tree, renderer, limits))
    }

    fn draw(
        &self,
        tree: &tree::Tree,
        renderer: &mut crate::Renderer,
        theme: &crate::Theme,
        style: &crate::iced_core::renderer::Style,
        layout: crate::iced_core::Layout<'_>,
        cursor: crate::iced_core::mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.element.with_data(move |e| {
            e.as_widget()
                .draw(tree, renderer, theme, style, layout, cursor, viewport);
        });
    }

    fn tag(&self) -> tree::Tag {
        self.element.with_data(|e| e.as_widget().tag())
    }

    fn state(&self) -> tree::State {
        self.element.with_data(|e| e.as_widget().state())
    }

    fn children(&self) -> Vec<tree::Tree> {
        self.element.with_data(|e| e.as_widget().children())
    }

    fn diff(&mut self, tree: &mut tree::Tree) {
        self.element.with_data_mut(|e| e.as_widget_mut().diff(tree));
    }

    fn operate(
        &self,
        state: &mut tree::Tree,
        layout: crate::iced_core::Layout<'_>,
        renderer: &crate::Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        self.element.with_data(|e| {
            e.as_widget().operate(state, layout, renderer, operation);
        });
    }

    fn on_event(
        &mut self,
        state: &mut tree::Tree,
        event: crate::iced::Event,
        layout: crate::iced_core::Layout<'_>,
        cursor: crate::iced_core::mouse::Cursor,
        renderer: &crate::Renderer,
        clipboard: &mut dyn crate::iced_core::Clipboard,
        shell: &mut crate::iced_core::Shell<'_, M>,
        viewport: &Rectangle,
    ) -> event::Status {
        self.element.with_data_mut(|e| {
            e.as_widget_mut().on_event(
                state, event, layout, cursor, renderer, clipboard, shell, viewport,
            )
        })
    }

    fn mouse_interaction(
        &self,
        state: &tree::Tree,
        layout: crate::iced_core::Layout<'_>,
        cursor: crate::iced_core::mouse::Cursor,
        viewport: &Rectangle,
        renderer: &crate::Renderer,
    ) -> crate::iced_core::mouse::Interaction {
        self.element.with_data(|e| {
            e.as_widget()
                .mouse_interaction(state, layout, cursor, viewport, renderer)
        })
    }

    fn overlay<'a>(
        &'a mut self,
        state: &'a mut tree::Tree,
        layout: crate::iced_core::Layout<'_>,
        renderer: &crate::Renderer,
        translation: crate::iced_core::Vector,
    ) -> Option<crate::iced_core::overlay::Element<'a, M, crate::Theme, crate::Renderer>> {
        assert_eq!(self.element.thread_id, thread::current().id());
        Rc::get_mut(&mut self.element.data).and_then(|e| {
            e.get_mut()
                .as_widget_mut()
                .overlay(state, layout, renderer, translation)
        })
    }

    fn id(&self) -> Option<Id> {
        self.element.with_data_mut(|e| e.as_widget_mut().id())
    }

    fn set_id(&mut self, id: Id) {
        self.element.with_data_mut(|e| e.as_widget_mut().set_id(id));
    }

    fn drag_destinations(
        &self,
        state: &tree::Tree,
        layout: crate::iced_core::Layout<'_>,
        renderer: &crate::Renderer,
        dnd_rectangles: &mut crate::iced_core::clipboard::DndDestinationRectangles,
    ) {
        self.element.with_data_mut(|e| {
            e.as_widget_mut()
                .drag_destinations(state, layout, renderer, dnd_rectangles);
        });
    }
}

impl<Message: 'static> From<RcElementWrapper<Message>> for Element<'static, Message> {
    fn from(wrapper: RcElementWrapper<Message>) -> Self {
        Element::new(wrapper)
    }
}

impl<Message: 'static> From<Element<'static, Message>> for RcElementWrapper<Message> {
    fn from(e: Element<'static, Message>) -> Self {
        RcElementWrapper::new(e)
    }
}
