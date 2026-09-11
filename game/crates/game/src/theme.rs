//! Shared game palette. Health and selection are blue; progress and gains are
//! green; damage, harmful effects, and alerts are red. Labels and symbols carry
//! the same meaning so that color is never the only signal.

use bevy::prelude::Color;

pub(crate) const CANVAS: Color = Color::oklch(0.985, 0.004, 90.0);
pub(crate) const SURFACE: Color = Color::oklch(1.0, 0.0, 0.0);
pub(crate) const INK: Color = Color::oklch(0.205, 0.009, 260.0);
pub(crate) const MUTED: Color = Color::oklch(0.510, 0.014, 260.0);
pub(crate) const BORDER: Color = Color::oklch(0.770, 0.008, 260.0);
pub(crate) const TRACK: Color = Color::oklch(0.925, 0.008, 260.0);

pub(crate) const SELECTED: Color = Color::oklch(0.565, 0.220, 260.0);
pub(crate) const HP: Color = Color::oklch(0.595, 0.195, 255.0);
pub(crate) const XP: Color = Color::oklch(0.565, 0.145, 155.0);
pub(crate) const POSITIVE: Color = Color::oklch(0.505, 0.135, 155.0);
pub(crate) const NEGATIVE: Color = Color::oklch(0.570, 0.220, 20.0);
pub(crate) const ALERT: Color = Color::oklch(0.630, 0.245, 20.0);
pub(crate) const OVERLAY: Color = Color::oklcha(0.205, 0.009, 260.0, 0.45);

pub(crate) const SMALL: f32 = 10.0;
pub(crate) const BODY: f32 = 12.0;
pub(crate) const TITLE: f32 = 14.0;
