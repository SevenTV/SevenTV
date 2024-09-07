use proc_macro_crate::{crate_name, FoundCrate};
use quote::{format_ident, quote, quote_spanned};
use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::Token;

const ATTRIBUTE_NAME: &str = "query";

use crate::mongo_filter_query;

fn parse_mode(ast: &mut syn::Expr) -> Result<Mode, syn::Error> {
	let attrs = match ast {
		syn::Expr::Array(ast) => &mut ast.attrs,
		syn::Expr::Assign(ast) => &mut ast.attrs,
		syn::Expr::Async(ast) => &mut ast.attrs,
		syn::Expr::Await(ast) => &mut ast.attrs,
		syn::Expr::Binary(ast) => &mut ast.attrs,
		syn::Expr::Block(ast) => &mut ast.attrs,
		syn::Expr::Break(ast) => &mut ast.attrs,
		syn::Expr::Call(ast) => &mut ast.attrs,
		syn::Expr::Cast(ast) => &mut ast.attrs,
		syn::Expr::Closure(ast) => &mut ast.attrs,
		syn::Expr::Const(ast) => &mut ast.attrs,
		syn::Expr::Continue(ast) => &mut ast.attrs,
		syn::Expr::Field(ast) => &mut ast.attrs,
		syn::Expr::ForLoop(ast) => &mut ast.attrs,
		syn::Expr::Group(ast) => &mut ast.attrs,
		syn::Expr::If(ast) => &mut ast.attrs,
		syn::Expr::Index(ast) => &mut ast.attrs,
		syn::Expr::Infer(ast) => &mut ast.attrs,
		syn::Expr::Let(ast) => &mut ast.attrs,
		syn::Expr::Lit(ast) => &mut ast.attrs,
		syn::Expr::Loop(ast) => &mut ast.attrs,
		syn::Expr::Macro(ast) => &mut ast.attrs,
		syn::Expr::Match(ast) => &mut ast.attrs,
		syn::Expr::MethodCall(ast) => &mut ast.attrs,
		syn::Expr::Paren(ast) => &mut ast.attrs,
		syn::Expr::Path(ast) => &mut ast.attrs,
		syn::Expr::Range(ast) => &mut ast.attrs,
		syn::Expr::Reference(ast) => &mut ast.attrs,
		syn::Expr::Repeat(ast) => &mut ast.attrs,
		syn::Expr::Return(ast) => &mut ast.attrs,
		syn::Expr::Struct(ast) => &mut ast.attrs,
		syn::Expr::Try(ast) => &mut ast.attrs,
		syn::Expr::TryBlock(ast) => &mut ast.attrs,
		syn::Expr::Tuple(ast) => &mut ast.attrs,
		syn::Expr::Unary(ast) => &mut ast.attrs,
		syn::Expr::Unsafe(ast) => &mut ast.attrs,
		syn::Expr::While(ast) => &mut ast.attrs,
		syn::Expr::Yield(ast) => &mut ast.attrs,
		_ => return Err(syn::Error::new(ast.span(), "expected attributes")),
	};

	let query_attrs = attrs
		.iter()
		.filter(|attr| attr.meta.path().is_ident(ATTRIBUTE_NAME))
		.cloned()
		.collect::<Vec<_>>();
	attrs.retain(|attr| !query_attrs.contains(attr));

	let mut mode = None;
	for attr in query_attrs {
		let syn::Meta::List(syn::MetaList { tokens, .. }) = attr.meta.clone() else {
			return Err(syn::Error::new(attr.meta.span(), "Expected list meta"));
		};

		// parse delimiter ','
		let parser = syn::meta::parser(|meta| {
			if meta.path.is_ident("push") {
				if mode.is_some() {
					return Err(meta.error("duplicated action attribute"));
				}
				mode = Some(Mode::Push(meta.path.clone()));
			} else if meta.path.is_ident("pull") {
				if mode.is_some() {
					return Err(meta.error("duplicated action attribute"));
				}
				mode = Some(Mode::Pull(meta.path.clone()));
			} else if meta.path.is_ident("add_to_set") {
				if mode.is_some() {
					return Err(meta.error("duplicated action attribute"));
				}
				mode = Some(Mode::AddToSet(meta.path.clone()));
			} else if meta.path.is_ident("pop") {
				if mode.is_some() {
					return Err(meta.error("duplicated action attribute"));
				}
				mode = Some(Mode::Pop(meta.path.clone()));
			} else if meta.path.is_ident("pull_all") {
				if mode.is_some() {
					return Err(meta.error("duplicated action attribute"));
				}
				mode = Some(Mode::PullAll(meta.path.clone()));
			} else if meta.path.is_ident("set_on_insert") {
				if mode.is_some() {
					return Err(meta.error("duplicated action attribute"));
				}
				mode = Some(Mode::SetOnInsert(meta.path.clone()));
			} else if meta.path.is_ident("unset") {
				if mode.is_some() {
					return Err(meta.error("duplicated action attribute"));
				}
				mode = Some(Mode::Unset(meta.path.clone()));
			} else if meta.path.is_ident("inc") {
				if mode.is_some() {
					return Err(meta.error("duplicated action attribute"));
				}
				mode = Some(Mode::Inc(meta.path.clone()));
			} else if meta.path.is_ident("mul") {
				if mode.is_some() {
					return Err(meta.error("duplicated action attribute"));
				}
				mode = Some(Mode::Mul(meta.path.clone()));
			} else if meta.path.is_ident("max") {
				if mode.is_some() {
					return Err(meta.error("duplicated action attribute"));
				}
				mode = Some(Mode::Max(meta.path.clone()));
			} else if meta.path.is_ident("min") {
				if mode.is_some() {
					return Err(meta.error("duplicated action attribute"));
				}
				mode = Some(Mode::Min(meta.path.clone()));
			} else if meta.path.is_ident("set") {
				if mode.is_some() {
					return Err(meta.error("duplicated action attribute"));
				}
				mode = Some(Mode::Set(meta.path.clone()));
			} else if meta.path.is_ident("bit") {
				if mode.is_some() {
					return Err(meta.error("duplicated action attribute"));
				}
				mode = Some(Mode::Bit(meta.path.clone()));
			} else {
				return Err(meta.error("unknown attribute"));
			}

			Ok(())
		});

		parser.parse2(tokens)?;
	}

	mode.ok_or_else(|| syn::Error::new(ast.span(), "Expected `set`, `unset`, `inc`, `mul`, `setOnInsert`, `push`, `pull`, `addToSet`, `pop`, `pullAll`, `setOnInsert` attribute"))
}

pub(crate) fn proc_macro(input: proc_macro2::TokenStream) -> Result<proc_macro2::TokenStream, syn::Error> {
	let found_crate = crate_name("shared").expect("shared is present in `Cargo.toml`");
	let shared_crate = match found_crate {
		FoundCrate::Itself => quote!(crate),
		FoundCrate::Name(name) => {
			let name = format_ident!("{name}");
			quote!(::#name)
		}
	};

	let path = quote! { #shared_crate::database::queries::update };

	struct Exprs(Vec<syn::Expr>);

	impl syn::parse::Parse for Exprs {
		fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
			let result = Punctuated::<syn::Expr, Token![,]>::parse_terminated(input)?;
			Ok(Self(result.into_iter().collect()))
		}
	}

	let exprs = syn::parse2::<Exprs>(input)?
		.0
		.into_iter()
		.map(|mut expr| {
			let mode = parse_mode(&mut expr)?;
			map_expr(expr, mode.clone()).map(|ts| (mode, ts))
		})
		.collect::<Result<Vec<_>, _>>()?;

	if exprs.len() == 1 {
		let (_, ts) = exprs.into_iter().next().unwrap();
		Ok(ts)
	} else {
		let exprs = exprs.into_iter().map(|(mode, ts)| {
			let path = mode.path();
			quote! {
				.#path(#ts)
			}
		});

		Ok(quote! {
			#path::Update::builder()
				#(#exprs)*
				.build()
		})
	}
}

fn map_expr(ast: syn::Expr, mode: Mode) -> Result<proc_macro2::TokenStream, syn::Error> {
	match ast {
		syn::Expr::Struct(ast) => Ok(mapper_struct(ast, mode)?),
		r => Ok(quote! { #r }),
	}
}

#[derive(Debug)]
struct Struct {
	path: syn::Path,
	fields: Vec<Field>,
	mode: Mode,
	expr: syn::ExprStruct,
}

impl Struct {
	fn from_syn(expr: syn::ExprStruct, mode: Mode) -> Result<Self, syn::Error> {
		let fields = if !matches!(mode, Mode::SetOnInsert(_)) {
			expr.fields
				.iter()
				.cloned()
				.map(|field| Field::from_syn(mode.clone(), field))
				.collect::<Result<Vec<_>, _>>()?
		} else {
			vec![]
		};

		Ok(Self {
			path: expr.path.clone(),
			fields,
			mode,
			expr,
		})
	}

	fn to_bson(&self) -> proc_macro2::TokenStream {
		if let Mode::SetOnInsert(_) = &self.mode {
			let expr = &self.expr;
			return quote! { bson::to_document(&#expr).expect("Failed to serialize") };
		};

		let fields = self.fields.iter().map(|field| field.to_bson());
		quote! {
			{
				let mut ____doc = bson::Document::new();
				#(#fields)*
				____doc
			}
		}
	}
}

#[derive(Debug)]
struct Field {
	name: syn::Ident,
	rename: Option<syn::LitStr>,
	kind: FieldKind,
	index: Option<syn::Expr>,
	mode: Mode,
	each: bool,
	flatten: bool,
	optional: bool,
	position: Option<syn::Expr>,
	slice: Option<syn::Expr>,
	span: proc_macro2::Span,
	serde: Option<Option<syn::Path>>,
	bit_mode: Option<BitMode>,
}

#[derive(Debug)]
enum BitMode {
	And(syn::LitStr),
	Or(syn::LitStr),
	Xor(syn::LitStr),
}

impl BitMode {
	fn from_str(s: syn::LitStr) -> Result<Self, syn::Error> {
		match s.value().as_str().to_lowercase().trim_start_matches('$') {
			"and" => Ok(Self::And(s)),
			"or" => Ok(Self::Or(s)),
			"xor" => Ok(Self::Xor(s)),
			_ => Err(syn::Error::new(s.span(), "invalid bit mode")),
		}
	}
}

#[derive(Debug, Clone)]
enum Mode {
	Set(syn::Path),
	SetOnInsert(syn::Path),
	Unset(syn::Path),
	Inc(syn::Path),
	Mul(syn::Path),
	Max(syn::Path),
	Min(syn::Path),
	Push(syn::Path),
	Pull(syn::Path),
	AddToSet(syn::Path),
	Pop(syn::Path),
	PullAll(syn::Path),
	Bit(syn::Path),
}

impl Mode {
	fn path(&self) -> &syn::Path {
		match self {
			Mode::Set(pth) => pth,
			Mode::SetOnInsert(pth) => pth,
			Mode::Unset(pth) => pth,
			Mode::Inc(pth) => pth,
			Mode::Mul(pth) => pth,
			Mode::Max(pth) => pth,
			Mode::Min(pth) => pth,
			Mode::Push(pth) => pth,
			Mode::Pull(pth) => pth,
			Mode::AddToSet(pth) => pth,
			Mode::Pop(pth) => pth,
			Mode::PullAll(pth) => pth,
			Mode::Bit(pth) => pth,
		}
	}
}

impl Field {
	fn from_syn(mode: Mode, field: syn::FieldValue) -> Result<Self, syn::Error> {
		let span = field.span();

		let name = match field.member {
			syn::Member::Named(field) => field,
			syn::Member::Unnamed(field) => {
				return Err(syn::Error::new(field.span(), "The root struct cannot be unnamed"));
			}
		};

		let attrs: Vec<_> = field
			.attrs
			.iter()
			.filter(|attr| attr.meta.path().is_ident(ATTRIBUTE_NAME))
			.cloned()
			.collect();

		let mut rename = None;
		let mut flatten = false;
		let mut each = false;
		let mut optional = false;
		let mut index = None;
		let mut position = None;
		let mut slice = None;
		let mut bit_mode = None;
		let mut serde = None;
		for attr in &attrs {
			let syn::Meta::List(syn::MetaList { tokens, .. }) = attr.meta.clone() else {
				return Err(syn::Error::new(attr.meta.span(), "Expected list meta"));
			};

			// parse delimiter ','
			let parser = syn::meta::parser(|meta| {
				if meta.path.is_ident("rename") {
					if rename.is_some() {
						return Err(meta.error("duplicated `rename` attribute"));
					}
					rename = Some(meta.value()?.parse()?);
				} else if meta.path.is_ident("index") {
					if index.is_some() {
						return Err(meta.error("duplicated `index` attribute"));
					} else if each {
						return Err(meta.error("`each` attribute cannot be used with `index`"));
					}
					let lit: syn::LitStr = meta.value()?.parse()?;
					if lit.value() == "$" {
						index = Some(syn::parse2(quote! { update::ArrayIndex::DollarSign })?);
					} else {
						index = Some(syn::parse_str(&lit.value())?);
					}
				} else if meta.path.is_ident("each") && matches!(mode, Mode::Push(_) | Mode::AddToSet(_)) {
					if each {
						return Err(meta.error("duplicated `each` attribute"));
					}

					each = true;
				} else if meta.path.is_ident("position") && matches!(mode, Mode::Push(_)) {
					if position.is_some() {
						return Err(meta.error("duplicated `position` attribute"));
					}

					let lit: syn::LitStr = meta.value()?.parse()?;
					position = Some(syn::parse_str(&lit.value())?);
				} else if meta.path.is_ident("slice") && matches!(mode, Mode::Push(_)) {
					if slice.is_some() {
						return Err(meta.error("duplicated `slice` attribute"));
					}

					let lit: syn::LitStr = meta.value()?.parse()?;
					slice = Some(syn::parse_str(&lit.value())?);
				} else if meta.path.is_ident("flatten") {
					if flatten {
						return Err(meta.error("duplicated `flatten` attribute"));
					}
					flatten = true;
				} else if meta.path.is_ident("bit") && matches!(mode, Mode::Bit(_)) {
					if bit_mode.is_some() {
						return Err(meta.error("duplicated `bit` attribute"));
					}

					let lit: syn::LitStr = meta.value()?.parse()?;
					bit_mode = Some(BitMode::from_str(lit)?);
				} else if meta.path.is_ident("optional") {
					if optional {
						return Err(meta.error("duplicated `optional` attribute"));
					}

					optional = true;
				} else if meta.path.is_ident("serde") {
					if serde.is_some() {
						return Err(meta.error("duplicated `serde` attribute"));
					}

					if let Ok(lit) = meta.value() {
						let value: syn::LitStr = lit.parse()?;
						let path = syn::parse_str(&value.value())?;
						serde = Some(Some(path));
					} else {
						serde = Some(None);
					}
				} else {
					return Err(meta.error("unknown attribute"));
				}

				Ok(())
			});

			parser.parse2(tokens)?;
		}

		let kind = match field.expr {
			syn::Expr::Struct(expr)
				if flatten && expr.fields.iter().all(|field| matches!(field.member, syn::Member::Named(_))) =>
			{
				FieldKind::Struct(mapper_struct(expr, mode.clone())?)
			}
			syn::Expr::Struct(expr) if !flatten && matches!(mode, Mode::Pull(_)) => {
				FieldKind::Struct(mongo_filter_query::mapper_struct(expr)?)
			}
			expr => FieldKind::Value(expr),
		};

		Ok(Self {
			name,
			rename,
			kind,
			index,
			mode,
			each,
			flatten,
			span,
			position,
			slice,
			bit_mode,
			optional,
			serde,
		})
	}

	fn bson_key(&self) -> syn::LitStr {
		self.rename
			.clone()
			.unwrap_or_else(|| syn::LitStr::new(&self.name.to_string(), self.name.span()))
	}

	fn bson_value(&self) -> proc_macro2::TokenStream {
		let name = &self.name;

		let value = match &self.serde {
			Some(Some(serde)) => quote_spanned! { self.name.span() => {
				#serde(&#name, bson::ser::Serializer::new()).expect("Failed to serialize")
			} },
			Some(None) => quote_spanned! { self.name.span() => bson::to_bson(&#name).expect("Failed to serialize") },
			None => quote_spanned! { self.name.span() => bson::bson!(#name) },
		};

		let value = if self.optional {
			quote_spanned! { name.span() => #name.map(|#name| #value) }
		} else {
			quote_spanned! { name.span() => Some(#value) }
		};

		if self.each || self.position.is_some() || self.slice.is_some() {
			let value_transform = if !self.each {
				quote! {
					let value = bson::bson!([#value]);
				}
			} else {
				quote! {}
			};

			let position = self.position.as_ref().map(|expr| {
				quote! {
					let position: i64 = #expr;
					____doc.insert("$position", bson::bson!(position));
				}
			});

			let slice = self.slice.as_ref().map(|expr| {
				quote! {
					let slice: i64 = #expr;
					____doc.insert("$slice", bson::bson!(slice));
				}
			});

			quote! {
				{
					#value.map(|value| {
						#value_transform
						let mut ____doc = bson::Document::new();
						#position
						#slice
						____doc.insert("$each", value);
						____doc
					})
				}
			}
		} else {
			quote! { #value }
		}
	}

	fn to_bson(&self) -> proc_macro2::TokenStream {
		let key = self.bson_key();
		let key = if let Some(index) = &self.index {
			quote_spanned! { index.span() => {
				{
					let ____idx = update::ArrayIndex::from(#index);
					format!("{}.{____idx}", #key)
				}
			} }
		} else {
			quote! { #key }
		};

		let key = quote! { let ____key = #key; };
		let value = self.bson_value();

		let value = if self.flatten {
			let unset_filter_out = if matches!(self.mode, Mode::Unset(_)) {
				quote! {
					Some(bson::Bson::Boolean(false)) => {},
				}
			} else {
				quote! {}
			};

			quote! {
				{
					match #value {
						Some(bson::Bson::Document(doc)) => {
							for (key, value) in doc {
								____doc.insert(format!("{}.{}", ____key, key), value);
							}
						}
						#unset_filter_out
						Some(v) => {
							____doc.insert(____key, v);
						}
						None => {}
					}
				}
			}
		} else if let Some(bit_mode) = &self.bit_mode {
			let bit_mode = match bit_mode {
				BitMode::And(lit) => quote_spanned! { lit.span() => "and" },
				BitMode::Or(lit) => quote_spanned! { lit.span() => "or" },
				BitMode::Xor(lit) => quote_spanned! { lit.span() => "xor" },
			};

			quote! {
				{
					if let Some(value) = #value {
						____doc.insert(____key, {
							let mut ____doc = bson::Document::new();
							____doc.insert(#bit_mode, value);
							____doc
						});
					}
				}
			}
		} else {
			quote! {
				if let Some(value) = #value {
					____doc.insert(____key, value);
				}
			}
		};

		quote! {
			{
				#key
				#value
			}
		}
	}
}

#[derive(Debug)]
enum FieldKind {
	/// Treat the field expr as a expr and use bson::to_bson to serialize it.
	Value(syn::Expr),
	/// Treat the field as a struct and type check it.
	Struct(proc_macro2::TokenStream),
}

fn mapper_struct(ast: syn::ExprStruct, mode: Mode) -> Result<proc_macro2::TokenStream, syn::Error> {
	let root_stuct = Struct::from_syn(ast, mode)?;

	let found_crate = crate_name("shared").expect("shared is present in `Cargo.toml`");
	let shared_crate = match found_crate {
		FoundCrate::Itself => quote!(crate),
		FoundCrate::Name(name) => {
			let name = format_ident!("{name}");
			quote!(::#name)
		}
	};

	let path = quote! { #shared_crate::database::queries::update };

	let mode = match &root_stuct.mode {
		Mode::Set(pth) => quote_spanned! { pth.span() => Set },
		Mode::Unset(pth) => quote_spanned! { pth.span() => Unset },
		Mode::Inc(pth) => quote_spanned! { pth.span() => Inc },
		Mode::Mul(pth) => quote_spanned! { pth.span() => Mul },
		Mode::Max(pth) => quote_spanned! { pth.span() => Max },
		Mode::Min(pth) => quote_spanned! { pth.span() => Min },
		Mode::SetOnInsert(pth) => quote_spanned! { pth.span() => SetOnInsert },
		Mode::Push(pth) => quote_spanned! { pth.span() => Push },
		Mode::Pull(pth) => quote_spanned! { pth.span() => Pull },
		Mode::AddToSet(pth) => quote_spanned! { pth.span() => AddToSet },
		Mode::Pop(pth) => quote_spanned! { pth.span() => Pop },
		Mode::PullAll(pth) => quote_spanned! { pth.span() => PullAll },
		Mode::Bit(pth) => quote_spanned! { pth.span() => Bit },
	};

	let mode = quote! {
		#path::#mode
	};

	let variables = {
		let variables = root_stuct.fields.iter().map(|field| {
			let name = &field.name;
			let expr = match &field.kind {
				FieldKind::Struct(s) => quote! { #s },
				FieldKind::Value(expr) => quote! { #expr },
			};
			quote_spanned! { field.span => let #name = #expr; }
		});

		quote! {
			#(#variables)*
		}
	};

	let asserts = {
		let ty = &root_stuct.path;
		let struct_ident = format_ident!("____struct");

		let fields = root_stuct.fields.iter().map(|field| {
			let name = &field.name;

			let is_array = field.each
				|| field.index.is_some()
				|| (matches!(
					field.mode,
					Mode::Push(_) | Mode::Pull(_) | Mode::AddToSet(_) | Mode::Pop(_) | Mode::PullAll(_)
				) && !field.flatten);

			let input_array = field.each || (matches!(field.mode, Mode::PullAll(_)) && !field.flatten);

			let assert_fn = match (is_array, input_array, field.flatten, field.optional) {
				(false, false, false, false) => quote! {
					____assert_types
				},
				(false, _, true, false) => quote! {
					____assert_flatten
				},
				(true, _, true, false) => quote! {
					____assert_array_flatten
				},
				(_, true, false, false) => quote! {
					____assert_array_input
				},
				(true, false, false, false) => quote! {
					____assert_array
				},
				(false, false, false, true) => quote! {
					____assert_types_optional
				},
				(false, _, true, true) => quote! {
					____assert_flatten_optional
				},
				(true, _, true, true) => quote! {
					____assert_array_flatten_optional
				},
				(_, true, false, true) => quote! {
					____assert_array_input_optional
				},
				(true, false, false, true) => quote! {
					____assert_array_optional
				},
			};

			quote_spanned! { field.span => #assert_fn(#struct_ident().#name, #name); }
		});

		quote! {
			#[inline(always)]
			#[doc(hidden)]
			fn ____make<T>() -> T {
				unreachable!()
			}

			#[inline(always)]
			#[doc(hidden)]
			fn ____assert_types<T>(_: T, _: impl #path::__AssertUpdateBounds<#mode<T>,T>) -> ! {
				unreachable!()
			}

			#[inline(always)]
			#[doc(hidden)]
			fn ____assert_flatten<T>(_: T, _: impl #path::__AssertUpdateBoundsFlatten<#mode<T>, T>) -> ! {
				unreachable!()
			}

			#[inline(always)]
			#[doc(hidden)]
			fn ____assert_array_flatten<T>(_: impl #path::__ArrayLike<T>, _: impl #path::__AssertUpdateBoundsFlatten<#mode<T>, T>) -> ! {
				unreachable!()
			}

			#[inline(always)]
			#[doc(hidden)]
			fn ____assert_array_input<T, U: #path::__AssertUpdateBounds<#mode<T>, T>>(_: impl #path::__ArrayLike<T>, _: impl #path::__ArrayLike<U>) -> ! {
				unreachable!()
			}

			#[inline(always)]
			#[doc(hidden)]
			fn ____assert_array<T>(_: impl #path::__ArrayLike<T>, _: impl #path::__AssertUpdateBounds<#mode<T>, T>) -> ! {
				unreachable!()
			}

			#[inline(always)]
			#[doc(hidden)]
			fn ____assert_types_optional<T>(_: T, _: Option<impl #path::__AssertUpdateBounds<#mode<T>,T>>) -> ! {
				unreachable!()
			}

			#[inline(always)]
			#[doc(hidden)]
			fn ____assert_flatten_optional<T>(_: T, _: Option<impl #path::__AssertUpdateBoundsFlatten<#mode<T>, T>>) -> ! {
				unreachable!()
			}

			#[inline(always)]
			#[doc(hidden)]
			fn ____assert_array_flatten_optional<T>(_: impl #path::__ArrayLike<T>, _: Option<impl #path::__AssertUpdateBoundsFlatten<#mode<T>, T>>) -> ! {
				unreachable!()
			}

			#[inline(always)]
			#[doc(hidden)]
			fn ____assert_array_input_optional<T, U: #path::__AssertUpdateBounds<#mode<T>, T>>(_: impl #path::__ArrayLike<T>, _: Option<impl #path::__ArrayLike<U>>) -> ! {
				unreachable!()
			}

			#[inline(always)]
			#[doc(hidden)]
			fn ____assert_array_optional<T>(_: impl #path::__ArrayLike<T>, _: Option<impl #path::__AssertUpdateBounds<#mode<T>, T>>) -> ! {
				unreachable!()
			}

			#[doc(hidden)]
			let #struct_ident = || ____make::<#ty>();

			#(#fields)*
		}
	};

	let bson_doc = root_stuct.to_bson();

	let ty = &root_stuct.path;

	Ok(quote! {
		{
			#[allow(clippy::redundant_locals)]
			#mode::<#ty>::new({
				#variables

				#[allow(unused_imports, unused_variables, dead_code, unused_unsafe, unsafe_code, unused_doc_comments, unreachable_code)]
				if false {
					#asserts
				}

				#bson_doc
			})
		}
	})
}
