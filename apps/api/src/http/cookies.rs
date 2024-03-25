//! Cookie middleware

use std::sync::Arc;

use cookie::{Cookie, CookieBuilder};

use crate::global::Global;

pub fn new_cookie<'c, C: Into<Cookie<'c>>>(global: &Arc<Global>, base: C) -> CookieBuilder<'c> {
	Cookie::build(base)
		.http_only(true)
		.domain(format!(".{}", global.config().api.domain))
		.path("/")
		.secure(true)
		.same_site(cookie::SameSite::None)
}
