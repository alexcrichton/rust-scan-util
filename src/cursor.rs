use super::{Tokenizer, Whitespace};
use super::{ScanError, OtherScanError};

use std::str::CharRange;

#[deriving(Clone, Show)]
pub struct Cursor<'a, Tok: Tokenizer, Sp: Whitespace> {
	slice: &'a str,
	offset: uint,
	tc: Tok,
	sp: Sp,
}

impl<'a, Tok: Tokenizer, Sp: Whitespace> Cursor<'a, Tok, Sp> {
	pub fn new<'b>(s: &'b str, tc: Tok, sp: Sp) -> Cursor<'b, Tok, Sp> {
		Cursor {
			slice: s,
			offset: 0,
			tc: tc,
			sp: sp,
		}
	}

	pub fn expect_tok(&self, s: &str) -> Result<Cursor<'a, Tok, Sp>, ScanError> {
		debug!("{}.expect_tok({})", self, s);
		match self.pop_token() {
			None => Err(OtherScanError(format!("expected `{}`, found end of input", s), self.offset)),
			Some((tok, cur)) => {
				return if eq_ignore_case(s, tok) {
					Ok(cur)
				} else {
					Err(OtherScanError(format!("expected `{}`, got `{}`", s, tok), self.offset))
				}
			}
		}
	}

	pub fn consumed(&self) -> uint {
		self.offset
	}

	pub fn pop_token(&self) -> Option<(&'a str, Cursor<'a, Tok, Sp>)> {
		debug!("{}.pop_token()", self);
		let cur = self;

		// First, check to see if there is a whitespace token.  This allows the space policy to do things like ignore most whitespace, but turn line breaks into explicit tokens.
		match self.sp.token_len(cur.tail_str()) {
			Some(end) => return Some((cur.str_slice_to(end), cur.slice_from(end))),
			None => ()
		}

		// If that didn't work, strip out leading whitespace.
		let cur = self.pop_ws();

		// Do not assume that empty input means we can't match a token; the token class might, for example, turn end-of-input into an explicit token.
		let tail_str = cur.tail_str();
		match self.tc.token_len(tail_str) {
			Some(end) => Some((cur.str_slice_to(end), cur.slice_from(end))),
			None => {
				// One of two things: either we have some input left and will thus return a single-character token, or there is nothing left whereby we return None.
				if cur.is_empty() {
					return None;
				} else {
					let CharRange { ch: _, next } = tail_str.char_range_at(0);
					Some((cur.str_slice_to(next), cur.slice_from(next)))
				}
			},
		}
	}

	pub fn pop_ws(&self) -> Cursor<'a, Tok, Sp> {
		debug!("{}.pop_ws()", self);

		self.slice_from(self.sp.strip_len(self.tail_str()))
	}

	pub fn slice_from(&self, from: uint) -> Cursor<'a, Tok, Sp> {
		Cursor {
			offset: ::std::cmp::min(self.slice.len(), self.offset + from),
			..self.clone()
		}
	}

	pub fn str_slice_to(&self, to: uint) -> &'a str {
		self.tail_str().slice_to(to)
	}

	pub fn tail_str(&self) -> &'a str {
		self.slice.slice_from(self.offset)
	}

	pub fn is_empty(&self) -> bool {
		self.offset == self.slice.len()
	}
}

fn eq_ignore_case<S: Str, T: Str>(left: S, right: T) -> bool {
	let left = left.as_slice();
	let right = right.as_slice();

	(left.len() == right.len() 
		&& (left.chars().zip(right.chars())
			.all(|(l,r)| l.to_lowercase() == r.to_lowercase())))
}
