//! Reui aims to be a large widget toolkit for Reclutch.
//! Beyond this, it also defines a framework to create widgets from.

#[macro_use]
pub extern crate reclutch;

#[allow(unused_imports)]
#[macro_use]
extern crate reui_derive;

pub use reui_derive::PipelineEvent;

pub use paste;

#[macro_use]
pub mod base;
pub mod draw;
pub mod error;
#[macro_use]
pub mod pipe;
pub mod themes;
pub mod ui;

#[cfg(feature = "app")]
pub mod app;

pub mod prelude {
    pub use crate::base::{Layout, Movable, Rectangular, Repaintable, Resizable, WidgetChildren};
}