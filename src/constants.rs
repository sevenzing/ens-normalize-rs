#![allow(dead_code)]

use crate::CodePoint;

pub const CP_STOP: CodePoint = 0x2E;
pub const CP_FE0F: CodePoint = 0xFE0F;
pub const CP_APOSTROPHE: CodePoint = 8217;
pub const CP_SLASH: CodePoint = 8260;
pub const CP_MIDDLE_DOT: CodePoint = 12539;
pub const CP_XI_SMALL: CodePoint = 0x3BE;
pub const CP_XI_CAPITAL: CodePoint = 0x39E;
pub const CP_UNDERSCORE: CodePoint = 0x5F;
pub const CP_HYPHEN: CodePoint = 0x2D;

pub const GREEK_GROUP_NAME: &str = "Greek";
pub const MAX_EMOJI_LEN: usize = 0x2d;
pub const STR_FE0F: &str = "\u{fe0f}";
