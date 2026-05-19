use std::sync::{Arc, Mutex};

use mlua::{UserData, UserDataFields, UserDataMethods};

use crate::frame::Frame;

/// Lua userdata wrapping a shared Frame. Created fresh each tick call.
pub struct FrameProxy(pub Arc<Mutex<Frame>>);

impl UserData for FrameProxy {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field("ROWS", crate::frame::ROWS);
        fields.add_field("COLS", crate::frame::COLS);
    }

    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("set", |_, this, (row, col, brightness): (usize, usize, u8)| {
            this.0.lock().unwrap().set(row, col, brightness);
            Ok(())
        });

        methods.add_method(
            "fill_rect",
            |_, this, (row, col, h, w, brightness): (usize, usize, usize, usize, u8)| {
                this.0.lock().unwrap().fill_rect(row, col, h, w, brightness);
                Ok(())
            },
        );

        methods.add_method("clear", |_, this, ()| {
            this.0.lock().unwrap().clear();
            Ok(())
        });
    }
}
