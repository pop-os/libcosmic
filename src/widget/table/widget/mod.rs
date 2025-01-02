mod state;
use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
};

use iced_core::{
    image::{self, Renderer as ImageRenderer},
    Layout,
};
use iced_core::{layout, text::Renderer as TextRenderer};
use iced_core::{renderer::Quad, Renderer as IcedRenderer};

use derive_setters::Setters;
use iced_core::{
    text::{LineHeight, Shaping, Wrapping},
    widget::{tree, Tree},
    Text, Widget,
};
use palette::named::BLACK;
use slotmap::SecondaryMap;
use state::State;

use super::model::{
    category::{ItemCategory, ItemInterface},
    selection::Selectable,
    Entity, Model,
};
use crate::{
    ext::CollectionWidget,
    theme,
    widget::{
        self, container, divider,
        dnd_destination::DragId,
        menu::{self, menu_roots_children, menu_roots_diff, MenuBarState},
    },
    Apply, Element,
};
use iced::{
    alignment,
    clipboard::{dnd::DndAction, mime::AllowedMimeTypes},
    Alignment, Background, Border, Length, Padding, Point, Rectangle, Size, Vector,
};

// THIS IS A PLACEHOLDER UNTIL A MORE SOPHISTICATED WIDGET CAN BE DEVELOPED

#[derive(Setters)]
#[must_use]
pub struct TableView<'a, SelectionMode, Item, Category, Message>
where
    Category: ItemCategory + Hash + 'static,
    Item: ItemInterface<Category>,
    Model<SelectionMode, Item, Category>: Selectable,
    SelectionMode: Default,
    Message: Clone + 'static,
{
    /// iced widget ID
    pub(super) id: widget::Id,
    /// The table model
    #[setters(skip)]
    pub(super) model: &'a Model<SelectionMode, Item, Category>,

    // === Element Layout ===
    /// Desired width of the widget.
    pub(super) width: Length,
    /// Desired height of the widget.
    pub(super) height: Length,
    /// Spacing between items and the dividers
    pub(super) item_spacing: u16,
    /// Spacing between text and icons in items
    pub(super) icon_spacing: u16,
    /// Size of the icon
    pub(super) icon_size: u16,
    /// The size of a single indent
    pub(super) indent_spacing: u16,
    /// The padding for the entire table
    #[setters(into)]
    pub(super) element_padding: Padding,
    /// The padding for each item
    #[setters(into)]
    pub(super) item_padding: Padding,
    /// the horizontal padding for each divider
    #[setters(into)]
    pub(super) divider_padding: Padding,
    /// Size of the font.
    pub(super) font_size: f32,

    /// The context tree to show on right clicking an item
    #[setters(skip)]
    pub(super) item_context_tree: Option<Vec<menu::Tree<'a, Message, crate::Renderer>>>,
    /// The context tree to show on right clicking a category
    #[setters(skip)]
    pub(super) category_context_tree: Option<Vec<menu::Tree<'a, Message, crate::Renderer>>>,

    // === Item Mouse Events ===
    /// Message to emit when the user left clicks on a table item
    #[setters(skip)]
    pub(super) on_item_left: Option<Box<dyn Fn(Entity) -> Message + 'a>>,
    /// Message to emit when the user double clicks on a table item
    #[setters(skip)]
    pub(super) on_item_double: Option<Box<dyn Fn(Entity) -> Message + 'a>>,
    /// Message to emit when the user middle clicks on a table item
    #[setters(skip)]
    pub(super) on_item_middle: Option<Box<dyn Fn(Entity) -> Message + 'a>>,
    /// Message to emit when the user right clicks on a table item
    #[setters(skip)]
    pub(super) on_item_right: Option<Box<dyn Fn(Entity) -> Message + 'a>>,

    // === Category Mouse Events ===
    /// Message to emit when the user clicks on a category
    #[setters(skip)]
    pub(super) on_category_select: Option<Box<dyn Fn(Category) -> Message + 'a>>,
    /// Message to emit when the user right clicks on a category
    #[setters(skip)]
    pub(super) on_category_context: Option<Box<dyn Fn(Category) -> Message + 'a>>,

    // === Drag n Drop ===
    /// Message to emit on the DND drop event
    #[setters(skip)]
    pub(super) on_dnd_drop:
        Option<Box<dyn Fn(Entity, Vec<u8>, String, DndAction) -> Message + 'static>>,
    /// MIME Types for the Drag n Drop
    pub(super) mimes: Vec<String>,
    /// Message to emit on the DND enter event
    #[setters(skip)]
    pub(super) on_dnd_enter: Option<Box<dyn Fn(Entity, Vec<String>) -> Message + 'static>>,
    /// Message to emit on the DND leave event
    #[setters(skip)]
    pub(super) on_dnd_leave: Option<Box<dyn Fn(Entity) -> Message + 'static>>,
    /// The Drag ID of the table
    #[setters(strip_option)]
    pub(super) drag_id: Option<DragId>,
}

// PRIVATE INTERFACE
impl<'a, SelectionMode, Item, Category, Message>
    TableView<'a, SelectionMode, Item, Category, Message>
where
    Category: ItemCategory + Hash + 'static,
    Item: ItemInterface<Category>,
    Model<SelectionMode, Item, Category>: Selectable,
    SelectionMode: Default,
    Message: Clone + 'static,
{
    fn max_item_dimensions(
        &self,
        state: &mut State<Category>,
        renderer: &crate::Renderer,
    ) -> (f32, f32) {
        let mut width = 0.0f32;
        let mut height = 0.0f32;
        let font = renderer.default_font();

        for key in self.model.iter() {
            let (button_width, button_height) = self.item_dimensions(state, font, key);

            state
                .item_layout
                .push(Size::new(button_width, button_height));

            height = height.max(button_height);
            width = width.max(button_width);
        }

        for size in &mut state.item_layout {
            size.height = height;
        }

        (width, height)
    }

    fn item_dimensions(
        &self,
        state: &mut State<Category>,
        font: crate::font::Font,
        id: Entity,
    ) -> (f32, f32) {
        let mut width = 0f32;
        let mut height = 0f32;
        let mut icon_spacing = 0f32;

        if let Some((item, paragraphs)) = self.model.item(id).zip(state.paragraphs.entry(id)) {
            let paragraphs = paragraphs.or_default();
            for category in self.model.categories.iter().copied() {
                if let Some(text) = item.get_text(category) {
                    if !text.is_empty() {
                        icon_spacing = f32::from(self.icon_spacing);
                        let paragraph = if let Some(entry) = paragraphs.get(&category) {
                            entry.clone()
                        } else {
                            crate::Plain::new(Text {
                                content: text.as_str(),
                                size: iced::Pixels(self.font_size),
                                bounds: Size::INFINITY
                                    .apply(|inf| Size::new(category.width() as f32, inf.height)),
                                font,
                                horizontal_alignment: alignment::Horizontal::Left,
                                vertical_alignment: alignment::Vertical::Center,
                                shaping: Shaping::Advanced,
                                wrapping: Wrapping::default(),
                                line_height: LineHeight::default(),
                            })
                        };

                        let size = paragraph.min_bounds();
                        width += size.width;
                        height = height.max(size.height);
                    }
                }
                // Add icon to measurement if icon was given.
                if let Some(_) = item.get_icon(category) {
                    width += f32::from(self.icon_size) + icon_spacing;
                    height = height.max(f32::from(self.icon_size));
                }
            }
        }

        // Add indent to measurement if found.
        if let Some(indent) = self.model.indent(id) {
            width = f32::from(indent).mul_add(f32::from(self.indent_spacing), width);
        }

        // Add button padding to the max size found
        width += f32::from(self.item_padding.left) + f32::from(self.item_padding.right);
        height += f32::from(self.item_padding.top) + f32::from(self.item_padding.top);

        (width, height)
    }
}

// PUBLIC INTERFACE
impl<'a, SelectionMode, Item, Category, Message>
    TableView<'a, SelectionMode, Item, Category, Message>
where
    SelectionMode: Default,
    Model<SelectionMode, Item, Category>: Selectable,
    Category: ItemCategory + Hash + 'static,
    Item: ItemInterface<Category>,
    Message: Clone + 'static,
{
    /// Creates a new table view with the given model
    pub fn new(model: &'a Model<SelectionMode, Item, Category>) -> Self {
        let cosmic_theme::Spacing {
            space_xxxs,
            space_xxs,
            ..
        } = theme::active().cosmic().spacing;

        Self {
            id: widget::Id::unique(),
            model,
            width: Length::Fill,
            height: Length::Shrink,
            item_spacing: 0,
            element_padding: Padding::from(0),
            divider_padding: Padding::from(0).left(space_xxxs).right(space_xxxs),
            item_padding: Padding::from(space_xxs).into(),
            icon_spacing: space_xxxs,
            icon_size: 24,
            indent_spacing: space_xxs,
            font_size: 14.0,
            item_context_tree: None,
            category_context_tree: None,
            on_category_select: None,
            on_category_context: None,
            on_item_left: None,
            on_item_double: None,
            on_item_middle: None,
            on_item_right: None,
            on_dnd_drop: None,
            mimes: Vec::new(),
            on_dnd_enter: None,
            on_dnd_leave: None,
            drag_id: None,
        }
    }

    /// Sets the message to be emitted on left click
    pub fn on_item_left<F>(mut self, on_left: F) -> Self
    where
        F: Fn(Entity) -> Message + 'a,
    {
        self.on_item_left = Some(Box::new(on_left));
        self
    }

    /// Sets the message to be emitted on double click
    pub fn on_item_double<F>(mut self, on_double: F) -> Self
    where
        F: Fn(Entity) -> Message + 'a,
    {
        self.on_item_double = Some(Box::new(on_double));
        self
    }

    /// Sets the message to be emitted on middle click
    pub fn on_item_middle<F>(mut self, on_middle: F) -> Self
    where
        F: Fn(Entity) -> Message + 'a,
    {
        self.on_item_middle = Some(Box::new(on_middle));
        self
    }

    /// Sets the message to be emitted on right click
    pub fn on_item_right<F>(mut self, on_right: F) -> Self
    where
        F: Fn(Entity) -> Message + 'a,
    {
        self.on_item_right = Some(Box::new(on_right));
        self
    }

    pub fn item_context(mut self, context_menu: Option<Vec<menu::Tree<'a, Message>>>) -> Self
    where
        Message: 'static,
    {
        self.item_context_tree =
            context_menu.map(|menus| vec![menu::Tree::with_children(widget::row(), menus)]);

        if let Some(ref mut context_menu) = self.item_context_tree {
            context_menu.iter_mut().for_each(menu::Tree::set_index);
        }

        self
    }

    pub fn on_category_select<F>(mut self, on_select: F) -> Self
    where
        F: Fn(Category) -> Message + 'a,
    {
        self.on_category_select = Some(Box::new(on_select));
        self
    }

    pub fn on_category_context<F>(mut self, on_select: F) -> Self
    where
        F: Fn(Category) -> Message + 'a,
    {
        self.on_category_context = Some(Box::new(on_select));
        self
    }

    /// Handle the dnd drop event.
    pub fn on_dnd_drop<D: AllowedMimeTypes>(
        mut self,
        dnd_drop_handler: impl Fn(Entity, Option<D>, DndAction) -> Message + 'static,
    ) -> Self {
        self.on_dnd_drop = Some(Box::new(move |entity, data, mime, action| {
            dnd_drop_handler(entity, D::try_from((data, mime)).ok(), action)
        }));
        self.mimes = D::allowed().iter().cloned().collect();
        self
    }

    /// Handle the dnd enter event.
    pub fn on_dnd_enter(
        mut self,
        dnd_enter_handler: impl Fn(Entity, Vec<String>) -> Message + 'static,
    ) -> Self {
        self.on_dnd_enter = Some(Box::new(dnd_enter_handler));
        self
    }

    /// Handle the dnd leave event.
    pub fn on_dnd_leave(mut self, dnd_leave_handler: impl Fn(Entity) -> Message + 'static) -> Self {
        self.on_dnd_leave = Some(Box::new(dnd_leave_handler));
        self
    }
}

// Widget implementation
impl<'a, SelectionMode, Item, Category, Message> Widget<Message, crate::Theme, crate::Renderer>
    for TableView<'a, SelectionMode, Item, Category, Message>
where
    SelectionMode: Default,
    Model<SelectionMode, Item, Category>: Selectable,
    Category: ItemCategory + Hash + 'static,
    Item: ItemInterface<Category>,
    Message: Clone + 'static,
{
    fn children(&self) -> Vec<Tree> {
        let mut children = Vec::new();

        // Item context tree
        if let Some(ref item_context) = self.item_context_tree {
            let mut tree = Tree::empty();
            tree.state = tree::State::new(MenuBarState::default());
            tree.children = menu_roots_children(&item_context);
            children.push(tree);
        }

        // Category context tree
        if let Some(ref category_context) = self.category_context_tree {
            let mut tree = Tree::empty();
            tree.state = tree::State::new(MenuBarState::default());
            tree.children = menu_roots_children(&category_context);
            children.push(tree);
        }

        children
    }

    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State<Category>>()
    }

    fn state(&self) -> tree::State {
        #[allow(clippy::default_trait_access)]
        tree::State::new(State::<Category> {
            num_items: self.model.order.len(),
            selected: self.model.active.clone(),
            paragraphs: SecondaryMap::new(),
            item_layout: Vec::new(),
            text_hashes: SecondaryMap::new(),

            sort_hash: HashMap::new(),
            header_paragraphs: HashMap::new(),
            category_layout: Vec::new(),
        })
    }

    fn diff(&mut self, tree: &mut Tree) {
        let state = tree.state.downcast_mut::<State<Category>>();

        let sort_state = self.model.sort;
        let mut hasher = DefaultHasher::new();
        sort_state.hash(&mut hasher);
        let cat_hash = hasher.finish();
        for category in self.model.categories.iter().copied() {
            if let Some(prev_hash) = state.sort_hash.insert(category, cat_hash) {
                if prev_hash == cat_hash {
                    continue;
                }
            }

            let category_name = category.to_string();
            let text = Text {
                content: category_name.as_str(),
                size: iced::Pixels(self.font_size),
                bounds: Size::INFINITY,
                font: crate::font::bold(),
                horizontal_alignment: alignment::Horizontal::Left,
                vertical_alignment: alignment::Vertical::Center,
                shaping: Shaping::Advanced,
                wrapping: Wrapping::default(),
                line_height: LineHeight::default(),
            };

            if let Some(header) = state.header_paragraphs.get_mut(&category) {
                header.update(text);
            } else {
                state
                    .header_paragraphs
                    .insert(category, crate::Plain::new(text));
            }
        }

        for key in self.model.iter() {
            if let Some(item) = self.model.item(key) {
                // TODO: add hover support if design approves it
                let button_state = self.model.is_active(key);

                let mut hasher = DefaultHasher::new();
                button_state.hash(&mut hasher);
                // hash each text
                for category in self.model.categories.iter().copied() {
                    let text = item.get_text(category);

                    text.hash(&mut hasher);
                }
                let text_hash = hasher.finish();

                if let Some(prev_hash) = state.text_hashes.insert(key, text_hash) {
                    // If the text didn't change, don't update the paragraph
                    if prev_hash == text_hash {
                        continue;
                    }
                }

                state.selected.insert(key, button_state);

                // Update the paragraph if the text changed
                for category in self.model.categories.iter().copied() {
                    if let Some(text) = item.get_text(category) {
                        let text = Text {
                            content: text.as_ref(),
                            size: iced::Pixels(self.font_size),
                            bounds: Size::INFINITY,
                            font: crate::font::default(),
                            horizontal_alignment: alignment::Horizontal::Left,
                            vertical_alignment: alignment::Vertical::Center,
                            shaping: Shaping::Advanced,
                            wrapping: Wrapping::default(),
                            line_height: LineHeight::default(),
                        };

                        if let Some(item) = state.paragraphs.get_mut(key) {
                            if let Some(paragraph) = item.get_mut(&category) {
                                paragraph.update(text);
                            } else {
                                item.insert(category, crate::Plain::new(text));
                            }
                        } else {
                            let mut hm = HashMap::new();
                            hm.insert(category, crate::Plain::new(text));
                            state.paragraphs.insert(key, hm);
                        }
                    }
                }
            }
        }

        // BUG: IF THE CATEGORY_CONTEXT AND ITEM_CONTEXT ARE CLEARED AND A DIFFERENT ONE IS SET, IT BREAKS
        // Diff the item context menu
        if let Some(ref mut item_context) = self.item_context_tree {
            if let Some(ref mut category_context) = self.category_context_tree {
                if tree.children.is_empty() {
                    let mut child_tree = Tree::empty();
                    child_tree.state = tree::State::new(MenuBarState::default());
                    tree.children.push(child_tree);

                    let mut child_tree = Tree::empty();
                    child_tree.state = tree::State::new(MenuBarState::default());
                    tree.children.push(child_tree);
                } else {
                    tree.children.truncate(2);
                }
                menu_roots_diff(item_context, &mut tree.children[0]);
                menu_roots_diff(category_context, &mut tree.children[1]);
            } else {
                if tree.children.is_empty() {
                    let mut child_tree = Tree::empty();
                    child_tree.state = tree::State::new(MenuBarState::default());
                    tree.children.push(child_tree);
                } else {
                    tree.children.truncate(1);
                }
                menu_roots_diff(item_context, &mut tree.children[0]);
            }
        } else {
            if let Some(ref mut category_context) = self.category_context_tree {
                if tree.children.is_empty() {
                    let mut child_tree = Tree::empty();
                    child_tree.state = tree::State::new(MenuBarState::default());
                    tree.children.push(child_tree);
                } else {
                    tree.children.truncate(1);
                }
                menu_roots_diff(category_context, &mut tree.children[0]);
            } else {
                tree.children.clear();
            }
        }
    }

    fn size(&self) -> iced::Size<Length> {
        Size::new(self.width, self.height)
    }

    fn layout(
        &self,
        tree: &mut Tree,
        renderer: &crate::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let state = tree.state.downcast_mut::<State<Category>>();
        let size = {
            state.item_layout.clear();
            state.category_layout.clear();
            let limits = limits.width(self.width);

            for category in self.model.categories.iter().copied() {
                state
                    .category_layout
                    .push(Size::new(category.width() as f32, 20.0));
            }

            let (width, item_height) = self.max_item_dimensions(state, renderer);

            for size in &mut state.item_layout {
                size.width = width.max(limits.max().width);
            }

            let spacing = f32::from(self.item_spacing);
            let mut height = state.category_layout[0].height;
            for _ in self.model.iter() {
                height += spacing + 1.0 + spacing;
                height += item_height;
            }

            limits.height(Length::Fixed(height)).resolve(
                self.width,
                self.height,
                Size::new(width, height),
            )
        };
        layout::Node::new(size)
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut crate::Renderer,
        theme: &crate::Theme,
        style: &iced_core::renderer::Style,
        layout: iced_core::Layout<'_>,
        cursor: iced_core::mouse::Cursor,
        viewport: &iced::Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State<Category>>();
        let cosmic = theme.cosmic();
        let accent = theme::active().cosmic().accent_color();

        let bounds = layout.bounds().shrink(self.element_padding);

        let header_quad = Rectangle::new(bounds.position(), Size::new(bounds.width, 20.0)).shrink(
            Padding::new(0.0)
                .left(self.item_padding.left)
                .right(self.item_padding.right),
        );

        // TODO: HEADER TEXT
        let mut category_origin = header_quad.position();
        for (nth, category) in self.model.categories.iter().copied().enumerate() {
            let category_quad = Rectangle::new(category_origin, state.category_layout[nth]);
            if let Some(category_quad) = category_quad.intersection(&bounds) {
                renderer.fill_paragraph(
                    state.header_paragraphs.get(&category).unwrap().raw(),
                    Point::new(category_quad.position().x, category_quad.center_y()),
                    cosmic.on_bg_color().into(),
                    category_quad,
                );
            } else {
                break;
            }

            category_origin.x += category_quad.width;
        }

        let body_quad = Rectangle::new(
            layout.position() + Vector::new(0.0, 20.0),
            Size::new(bounds.width, bounds.height - 20.0),
        );

        if self.model.order.is_empty() {
            let divider_quad = layout
                .bounds()
                .shrink(self.divider_padding)
                .apply(|mut dq| {
                    dq.height = 1.0;
                    dq
                });

            // If empty, draw a single divider and quit
            renderer.fill_quad(
                Quad {
                    bounds: divider_quad,
                    border: Default::default(),
                    shadow: Default::default(),
                },
                Background::Color(cosmic.bg_divider().into()),
            );
        } else {
            let mut divider_quad = body_quad.shrink(self.divider_padding).apply(|mut dq| {
                dq.height = 1.0;
                dq
            });
            let mut item_quad = body_quad.shrink(self.element_padding);
            for (nth, entity) in self.model.iter().enumerate() {
                // draw divider above

                renderer.fill_quad(
                    Quad {
                        bounds: divider_quad,
                        border: Default::default(),
                        shadow: Default::default(),
                    },
                    Background::Color(cosmic.bg_divider().into()),
                );

                divider_quad.y += 1.0;
                item_quad.y += 1.0;
                item_quad.width = state.item_layout[nth].width;
                item_quad.height = state.item_layout[nth].height;

                if state.selected.get(entity).copied().unwrap_or(false) {
                    renderer.fill_quad(
                        Quad {
                            bounds: item_quad,
                            border: Border {
                                color: iced::Color::TRANSPARENT,
                                width: 0.0,
                                radius: cosmic.radius_xs().into(),
                            },
                            shadow: Default::default(),
                        },
                        Background::Color(accent.into()),
                    );
                }

                let content_quad = item_quad.clone().shrink(self.item_padding);
                let mut item_content_quad = content_quad.clone();
                if let Some(item) = self.model.item(entity) {
                    for (nth, category) in self.model.categories.iter().copied().enumerate() {
                        // TODO: Icons and text
                        let mut icon_spacing = 0;
                        if let Some((_, _)) = item.get_icon(category).zip(item.get_text(category)) {
                            icon_spacing = self.icon_spacing;
                        }

                        item_content_quad.width = state.category_layout[nth].width;

                        if let Some(mut item_content_quad) =
                            item_content_quad.intersection(&content_quad)
                        {
                            let mut offset = Point::<f32>::default();
                            if let Some(icon) = item.get_icon(category) {
                                let layout_node = layout::Node::new(Size {
                                    width: self.icon_size as f32,
                                    height: self.icon_size as f32,
                                })
                                .move_to(Point {
                                    x: item_content_quad.x,
                                    y: item_content_quad.y,
                                });
                                Widget::<Message, crate::Theme, crate::Renderer>::draw(
                                    Element::<Message>::from(icon.clone()).as_widget(),
                                    &Tree::empty(),
                                    renderer,
                                    theme,
                                    style,
                                    Layout::new(&layout_node),
                                    cursor,
                                    viewport,
                                );
                                offset.x += self.icon_size as f32;
                            }

                            if let Some(text) = item.get_text(category) {
                                offset.x += icon_spacing as f32;

                                item_content_quad.x = item_content_quad.x + offset.x;
                                item_content_quad.width -= offset.x;

                                renderer.fill_paragraph(
                                    state
                                        .paragraphs
                                        .get(entity)
                                        .unwrap()
                                        .get(&category)
                                        .unwrap()
                                        .raw(),
                                    Point::new(
                                        item_content_quad.x,
                                        item_content_quad.y + item_content_quad.height / 2.0,
                                    ),
                                    cosmic.on_bg_color().into(),
                                    item_content_quad,
                                );
                            }
                        } else {
                            break;
                        }

                        item_content_quad.x += state.category_layout[nth].width;
                    }
                }

                item_quad.y += state.item_layout[nth].height;
                divider_quad.y += item_quad.height;
            }
        }
    }
}

impl<'a, SelectionMode, Item, Category, Message>
    TableView<'a, SelectionMode, Item, Category, Message>
where
    SelectionMode: Default,
    Model<SelectionMode, Item, Category>: Selectable,
    Category: ItemCategory + Hash + 'static,
    Item: ItemInterface<Category>,
    Message: Clone + 'static,
{
    #[must_use]
    pub fn element_standard(&self) -> Element<'a, Message> {
        let cosmic_theme::Spacing { space_xxxs, .. } = theme::active().cosmic().spacing;

        let header_row = self
            .model
            .categories
            .iter()
            .copied()
            .map(|category| {
                widget::row()
                    .spacing(space_xxxs)
                    .push(widget::text::heading(category.to_string()))
                    .push_maybe(if let Some(sort) = self.model.sort {
                        if sort.0 == category {
                            match sort.1 {
                                true => Some(widget::icon::from_name("pan-up-symbolic").icon()),
                                false => Some(widget::icon::from_name("pan-down-symbolic").icon()),
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    })
                    .apply(container)
                    .width(category.width())
                    .apply(widget::mouse_area)
                    .apply(|mouse_area| {
                        if let Some(ref on_category_select) = self.on_category_select {
                            mouse_area.on_press((on_category_select)(category))
                        } else {
                            mouse_area
                        }
                    })
                    .apply(Element::from)
            })
            .collect::<Vec<Element<'a, Message>>>()
            .apply(widget::row::with_children)
            .padding(
                Padding::default()
                    .left(self.item_padding.left)
                    .right(self.item_padding.right),
            )
            .apply(Element::from);
        let items_full = if self.model.items.is_empty() {
            vec![divider::horizontal::default()
                .apply(container)
                .padding(self.divider_padding)
                .apply(Element::from)]
        } else {
            self.model
                .iter()
                .map(|entity| {
                    let item = self.model.item(entity).unwrap();
                    let categories = &self.model.categories;
                    let selected = self.model.is_active(entity);

                    vec![
                        divider::horizontal::default()
                            .apply(container)
                            .padding(self.divider_padding)
                            .apply(Element::from),
                        categories
                            .iter()
                            .map(|category| {
                                widget::row()
                                    .spacing(self.icon_spacing)
                                    .push_maybe(
                                        item.get_icon(*category)
                                            .map(|icon| icon.size(self.icon_size)),
                                    )
                                    .push_maybe(
                                        item.get_text(*category)
                                            .map(|text| widget::text::body(text)),
                                    )
                                    .align_y(Alignment::Center)
                                    .apply(container)
                                    .width(category.width())
                                    .align_y(Alignment::Center)
                                    .apply(Element::from)
                            })
                            .collect::<Vec<Element<'a, Message>>>()
                            .apply(widget::row::with_children)
                            .apply(container)
                            .padding(self.item_padding)
                            .class(theme::Container::custom(move |theme| {
                                widget::container::Style {
                                    icon_color: if selected {
                                        Some(theme.cosmic().on_accent_color().into())
                                    } else {
                                        None
                                    },
                                    text_color: if selected {
                                        Some(theme.cosmic().on_accent_color().into())
                                    } else {
                                        None
                                    },
                                    background: if selected {
                                        Some(iced::Background::Color(
                                            theme.cosmic().accent_color().into(),
                                        ))
                                    } else {
                                        None
                                    },
                                    border: Border {
                                        radius: theme.cosmic().radius_xs().into(),
                                        ..Default::default()
                                    },
                                    shadow: Default::default(),
                                }
                            }))
                            .apply(widget::mouse_area)
                            .apply(|mouse_area| {
                                if let Some(ref on_item_select) = self.on_item_left {
                                    mouse_area.on_press((on_item_select)(entity))
                                } else {
                                    mouse_area
                                }
                            })
                            .apply(Element::from),
                    ]
                })
                .flatten()
                .collect::<Vec<Element<'a, Message>>>()
        };
        vec![vec![header_row], items_full]
            .into_iter()
            .flatten()
            .collect::<Vec<Element<'a, Message>>>()
            .apply(widget::column::with_children)
            .spacing(self.item_spacing)
            .padding(self.element_padding)
            .apply(Element::from)
    }

    #[must_use]
    pub fn element_compact(&self) -> Element<'a, Message> {
        let cosmic_theme::Spacing { space_xxxs, .. } = theme::active().cosmic().spacing;
        self.model
            .iter()
            .map(|entity| {
                let item = self.model.item(entity).unwrap();
                let selected = self.model.is_active(entity);
                widget::column()
                    .spacing(self.item_spacing)
                    .push(
                        widget::divider::horizontal::default()
                            .apply(container)
                            .padding(self.divider_padding),
                    )
                    .push(
                        widget::row()
                            .spacing(space_xxxs)
                            .align_y(Alignment::Center)
                            .push_maybe(
                                item.get_icon(Category::default()).map(|icon| icon.size(48)),
                            )
                            .push(
                                widget::column()
                                    .push_maybe(
                                        item.get_text(Category::default())
                                            .map(|text| widget::text::body(text)),
                                    )
                                    .push({
                                        let mut elements = self
                                            .model
                                            .categories
                                            .iter()
                                            .skip_while(|cat| **cat != Category::default())
                                            .map(|category| {
                                                item.get_text(*category)
                                                    .map(|text| {
                                                        vec![
                                                            widget::text::caption(text)
                                                                .apply(Element::from),
                                                            widget::text::caption("-")
                                                                .apply(Element::from),
                                                        ]
                                                    })
                                                    .unwrap_or_default()
                                            })
                                            .flatten()
                                            .collect::<Vec<Element<'a, Message>>>();
                                        elements.pop();
                                        elements
                                            .apply(widget::row::with_children)
                                            .spacing(space_xxxs)
                                            .wrap()
                                    }),
                            )
                            .apply(container)
                            .padding(self.item_padding)
                            .width(iced::Length::Fill)
                            .class(theme::Container::custom(move |theme| {
                                widget::container::Style {
                                    icon_color: if selected {
                                        Some(theme.cosmic().on_accent_color().into())
                                    } else {
                                        None
                                    },
                                    text_color: if selected {
                                        Some(theme.cosmic().on_accent_color().into())
                                    } else {
                                        None
                                    },
                                    background: if selected {
                                        Some(iced::Background::Color(
                                            theme.cosmic().accent_color().into(),
                                        ))
                                    } else {
                                        None
                                    },
                                    border: Border {
                                        radius: theme.cosmic().radius_xs().into(),
                                        ..Default::default()
                                    },
                                    shadow: Default::default(),
                                }
                            }))
                            .apply(widget::mouse_area)
                            .apply(|ma| {
                                if let Some(on_item_select) = &self.on_item_left {
                                    ma.on_press((on_item_select)(entity))
                                } else {
                                    ma
                                }
                            }),
                    )
                    .apply(Element::from)
            })
            .collect::<Vec<Element<'a, Message>>>()
            .apply(widget::column::with_children)
            .spacing(self.item_spacing)
            .padding(self.element_padding)
            .apply(Element::from)
    }
}
