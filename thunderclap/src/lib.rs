//! Thunderclap aims to be a large widget toolkit for Reclutch.
//! Beyond this, it also defines a framework to create widgets from.

#[macro_use]
pub extern crate reclutch;

#[allow(unused_imports)]
#[macro_use]
extern crate thunderclap_macros;

pub use thunderclap_macros::{
    rooftop, widget, DropNotifier, HasVisibility, LayableWidget, Movable, Repaintable, Resizable,
};

pub use paste;

#[macro_use]
pub mod base;
pub mod draw;
pub mod error;
pub mod geom;
#[cfg(feature = "core-widgets")]
pub mod ui;

#[cfg(feature = "app")]
pub mod app;
#[cfg(feature = "default-themes")]
pub mod themes;

pub mod prelude {
    pub use crate::{
        base::{Layout, Movable, Rectangular, Repaintable, Resizable, WidgetChildren},
        geom::{ContextuallyMovable, ContextuallyRectangular},
    };
}
