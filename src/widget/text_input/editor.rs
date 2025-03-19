// Copyright 2019 H�ctor Ram�n, Iced contributors
// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MIT

use super::{cursor::Cursor, value::Value};

pub struct Editor<'a> {
    value: &'a mut Value,
    cursor: &'a mut Cursor,
}

impl<'a> Editor<'a> {
    #[inline]
    pub fn new(value: &'a mut Value, cursor: &'a mut Cursor) -> Editor<'a> {
        Editor { value, cursor }
    }

    #[must_use]
    #[inline]
    pub fn contents(&self) -> String {
        self.value.to_string()
    }

    pub fn insert(&mut self, character: char) {
        if let Some((left, right)) = self.cursor.selection(self.value) {
            self.cursor.move_left(self.value);
            self.value.remove_many(left, right);
        }

        self.value.insert(self.cursor.end(self.value), character);
        self.cursor.move_right(self.value);
    }

    pub fn paste(&mut self, content: Value) {
        let length = content.len();
        if let Some((left, right)) = self.cursor.selection(self.value) {
            self.cursor.move_left(self.value);
            self.value.remove_many(left, right);
        }

        self.value.insert_many(self.cursor.end(self.value), content);

        self.cursor.move_right_by_amount(self.value, length);
    }

    pub fn backspace(&mut self) {
        if let Some((start, end)) = self.cursor.selection(self.value) {
            self.cursor.move_left(self.value);
            self.value.remove_many(start, end);
        } else {
            let start = self.cursor.start(self.value);

            if start > 0 {
                self.cursor.move_left(self.value);
                self.value.remove(start - 1);
            }
        }
    }

    pub fn delete(&mut self) {
        if self.cursor.selection(self.value).is_some() {
            self.backspace();
        } else {
            let end = self.cursor.end(self.value);

            if end < self.value.len() {
                self.value.remove(end);
            }
        }
    }
}
