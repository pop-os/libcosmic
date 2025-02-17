use std::{
    cell::RefCell,
    rc::Rc,
    thread::{self, ThreadId},
};

use crate::Element;
use iced::{event, Length, Rectangle, Size};
use iced_core::{id::Id, widget, widget::tree, Widget};

#[derive(Clone)]
pub struct RcElementWrapper<M> {
    pub(crate) element: Rc<RefCell<Element<'static, M>>>,
    pub(crate) thread_id: ThreadId,
}

impl<M> RcElementWrapper<M> {
    pub fn new(element: Element<'static, M>) -> Self {
        Self {
            element: Rc::new(RefCell::new(element)),
            thread_id: thread::current().id(),
        }
    }

    pub fn with_element<T>(&self, f: impl FnOnce(&Element<'static, M>) -> T) -> T {
        assert_eq!(self.thread_id, thread::current().id());
        let my_ref: &Element<'static, M> = &RefCell::borrow(self.element.as_ref());
        f(my_ref)
    }

    pub fn with_element_mut<T>(&self, f: impl FnOnce(&mut Element<'static, M>) -> T) -> T {
        assert_eq!(self.thread_id, thread::current().id());
        let my_refmut: &mut Element<'static, M> = &mut RefCell::borrow_mut(self.element.as_ref());
        f(my_refmut)
    }

    pub(crate) unsafe fn as_ptr(&self) -> *mut Element<'static, M> {
        assert_eq!(self.thread_id, thread::current().id());
        RefCell::as_ptr(self.element.as_ref())
    }
}

unsafe impl<M: 'static> Send for RcElementWrapper<M> {}
unsafe impl<M: 'static> Sync for RcElementWrapper<M> {}

impl<M> Widget<M, crate::Theme, crate::Renderer> for RcElementWrapper<M> {
    fn size(&self) -> Size<Length> {
        self.with_element(|e| e.as_widget().size())
    }

    fn size_hint(&self) -> Size<Length> {
        self.element.borrow_mut().as_widget().size_hint()
    }

    fn layout(
        &self,
        tree: &mut tree::Tree,
        renderer: &crate::Renderer,
        limits: &crate::iced_core::layout::Limits,
    ) -> crate::iced_core::layout::Node {
        self.with_element_mut(|e| e.as_widget_mut().layout(tree, renderer, limits))
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
        self.with_element(move |e| {
            e.as_widget()
                .draw(tree, renderer, theme, style, layout, cursor, viewport)
        })
    }

    fn tag(&self) -> tree::Tag {
        self.with_element(|e| e.as_widget().tag())
    }

    fn state(&self) -> tree::State {
        self.with_element(|e| e.as_widget().state())
    }

    fn children(&self) -> Vec<tree::Tree> {
        self.with_element(|e| e.as_widget().children())
    }

    fn diff(&mut self, tree: &mut tree::Tree) {
        self.with_element_mut(|e| e.as_widget_mut().diff(tree))
    }

    fn operate(
        &self,
        state: &mut tree::Tree,
        layout: crate::iced_core::Layout<'_>,
        renderer: &crate::Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        self.with_element(|e| {
            e.as_widget().operate(state, layout, renderer, operation);
        })
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
        self.with_element_mut(|e| {
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
        self.with_element(|e| {
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
        assert_eq!(self.thread_id, thread::current().id());
        Rc::get_mut(&mut self.element).and_then(|e| {
            e.get_mut()
                .as_widget_mut()
                .overlay(state, layout, renderer, translation)
        })
    }

    fn id(&self) -> Option<Id> {
        self.with_element_mut(|e| e.as_widget_mut().id())
    }

    fn set_id(&mut self, id: Id) {
        self.with_element_mut(|e| e.as_widget_mut().set_id(id))
    }

    fn drag_destinations(
        &self,
        state: &tree::Tree,
        layout: crate::iced_core::Layout<'_>,
        renderer: &crate::Renderer,
        dnd_rectangles: &mut crate::iced_core::clipboard::DndDestinationRectangles,
    ) {
        self.with_element_mut(|e| {
            e.as_widget_mut()
                .drag_destinations(state, layout, renderer, dnd_rectangles)
        })
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
