use proc_macro_crate::{crate_name, FoundCrate};
use quote::{format_ident, quote, quote_spanned};
use syn::parse::Parser;
use syn::spanned::Spanned;

const ATTRIBUTE_NAME: &str = "query";

#[derive(Debug)]
struct Struct {
	path: syn::Path,
	fields: Vec<Field>,
}

impl Struct {
	fn from_syn(s: syn::ExprStruct) -> Result<Self, syn::Error> {
		let path = s.path;
		let fields = s.fields.into_iter().map(Field::from_syn).collect::<Result<Vec<_>, _>>()?;

		Ok(Self { path, fields })
	}

	fn to_bson(&self) -> proc_macro2::TokenStream {
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
	query_selector: Option<QuerySelector>,
	flatten: bool,
	contains: bool,
	elem_match: bool,
	serde: Option<Option<syn::Path>>,
	index: Option<syn::Expr>,
}

#[derive(Debug, Clone)]
enum QuerySelector {
	Eq(syn::LitStr),
	Gt(syn::LitStr),
	Gte(syn::LitStr),
	Lt(syn::LitStr),
	Lte(syn::LitStr),
	Ne(syn::LitStr),
	In(syn::LitStr),
	Nin(syn::LitStr),
	Cin(syn::LitStr),
	Cnin(syn::LitStr),
	All(syn::LitStr),
}

impl syn::parse::Parse for QuerySelector {
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
		let selector = input.parse::<syn::LitStr>()?;
		match selector.value().as_str().trim_start_matches('$') {
			"eq" => Ok(Self::Eq(selector)),
			"gt" => Ok(Self::Gt(selector)),
			"gte" => Ok(Self::Gte(selector)),
			"lt" => Ok(Self::Lt(selector)),
			"lte" => Ok(Self::Lte(selector)),
			"ne" => Ok(Self::Ne(selector)),
			"in" => Ok(Self::In(selector)),
			"nin" => Ok(Self::Nin(selector)),
			"cin" => Ok(Self::Cin(selector)),
			"cnin" => Ok(Self::Cnin(selector)),
			"all" => Ok(Self::All(selector)),
			_ => Err(syn::Error::new(selector.span(), "unknown selector")),
		}
	}
}

impl Field {
	fn from_syn(field: syn::FieldValue) -> Result<Self, syn::Error> {
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
		let mut contains = false;
		let mut elem_match = false;
		let mut serde = None;
		let mut query_selector = None;
		let mut index = None;
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
				} else if meta.path.is_ident("selector") {
					if query_selector.is_some() {
						return Err(meta.error("duplicated `selector` attribute"));
					} else if flatten {
						return Err(meta.error("`selector` attribute cannot be used with `flatten` attribute"));
					}

					let selector = match meta.value()?.parse::<QuerySelector>() {
						Ok(selector) => selector,
						Err(e) => return Err(e),
					};
					query_selector = Some(selector);
				} else if meta.path.is_ident("flatten") {
					if flatten {
						return Err(meta.error("duplicated `flatten` attribute"));
					} else if query_selector.is_some() {
						return Err(meta.error("`flatten` attribute cannot be used with `selector` attribute"));
					} else if contains {
						return Err(meta.error("`flatten` attribute cannot be used with `contains` attribute"));
					}

					flatten = true;
				} else if meta.path.is_ident("index") {
					if index.is_some() {
						return Err(meta.error("duplicated `index` attribute"));
					}
					let lit: syn::LitStr = meta.value()?.parse()?;
					index = Some(syn::parse_str(&lit.value())?);
				} else if meta.path.is_ident("contains") {
					if contains {
						return Err(meta.error("duplicated `contains` attribute"));
					}
					contains = true;
				} else if meta.path.is_ident("elem_match") {
					if elem_match {
						return Err(meta.error("duplicated `elem_match` attribute"));
					}
					elem_match = true;
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
				if (flatten || elem_match)
					&& expr.fields.iter().all(|field| matches!(field.member, syn::Member::Named(_))) =>
			{
				FieldKind::Struct(mapper_struct(expr)?)
			}
			expr => FieldKind::Value(expr),
		};

		Ok(Self {
			name,
			rename,
			serde,
			kind,
			query_selector,
			flatten,
			contains,
			index,
			elem_match,
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

		let value = if let Some(selector) = &self.query_selector {
			let selector = match selector {
				QuerySelector::Eq(spanned) => quote_spanned! { spanned.span() => "$eq" },
				QuerySelector::Gt(spanned) => quote_spanned! { spanned.span() => "$gt" },
				QuerySelector::Gte(spanned) => quote_spanned! { spanned.span() => "$gte" },
				QuerySelector::Lt(spanned) => quote_spanned! { spanned.span() => "$lt" },
				QuerySelector::Lte(spanned) => quote_spanned! { spanned.span() => "$lte" },
				QuerySelector::Ne(spanned) => quote_spanned! { spanned.span() => "$ne" },
				QuerySelector::In(spanned) => quote_spanned! { spanned.span() => "$in" },
				QuerySelector::Nin(spanned) => quote_spanned! { spanned.span() => "$nin" },
				QuerySelector::All(spanned) => quote_spanned! { spanned.span() => "$all" },
				QuerySelector::Cin(spanned) => quote_spanned! { spanned.span() => "$in" },
				QuerySelector::Cnin(spanned) => quote_spanned! { spanned.span() => "$nin" },
			};
			quote! { {
				let mut ____doc = bson::Document::new();
				____doc.insert(#selector, #value);
				____doc
			} }
		} else {
			quote! { #value }
		};

		if self.elem_match {
			quote! {
				{
					let mut ____doc = bson::Document::new();
					____doc.insert("$elemMatch", #value);
					____doc
				}
			}
		} else {
			value
		}
	}

	fn to_bson(&self) -> proc_macro2::TokenStream {
		let key = self.bson_key();
		let key = if let Some(index) = &self.index {
			quote_spanned! { index.span() => {
				{
					let ____idx: usize = #index;
					format!("{}.{____idx}", #key)
				}
			} }
		} else {
			quote! { #key }
		};

		let key = quote! { let ____key = #key; };

		let value = if self.flatten {
			let value = self.bson_value();
			quote! {
				{
					match #value {
						bson::Bson::Document(doc) => {
							for (key, value) in doc {
								____doc.insert(format!("{____key}.{}",key), value);
							}
						}
						v => {
							____doc.insert(____key, v);
						}
					}
				}
			}
		} else {
			let value = self.bson_value();
			quote! {
				____doc.insert(____key, #value);
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
	/// Struct
	Struct(proc_macro2::TokenStream),
}

pub fn mapper_struct(ast: syn::ExprStruct) -> Result<proc_macro2::TokenStream, syn::Error> {
	let root_stuct = Struct::from_syn(ast)?;

	let found_crate = crate_name("shared").expect("shared is present in `Cargo.toml`");
	let shared_crate = match found_crate {
		FoundCrate::Itself => quote!(crate),
		FoundCrate::Name(name) => {
			let name = format_ident!("{name}");
			quote!(::#name)
		}
	};

	let path = quote! { #shared_crate::database::queries::filter };

	let variables = {
		let variables = root_stuct.fields.iter().map(|field| {
			let name = &field.name;
			let expr = match &field.kind {
				FieldKind::Struct(s) => quote! { #s },
				FieldKind::Value(expr) => quote! { #expr },
			};
			quote! { let #name = #expr; }
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

			let is_array = field.index.is_some()
				|| field.contains
				|| matches!(
					field.query_selector,
					Some(QuerySelector::All(_) | QuerySelector::Cin(_) | QuerySelector::Cnin(_))
				);
			let input_array = matches!(
				field.query_selector,
				Some(
					QuerySelector::All(_)
						| QuerySelector::In(_)
						| QuerySelector::Nin(_)
						| QuerySelector::Cin(_)
						| QuerySelector::Cnin(_)
				)
			);

			let flatten = field.flatten || field.elem_match;

			let assert_fn = match (is_array, input_array, flatten) {
				(true, true, false) => quote! {
					____assert_types_array_input
				},
				(true, false, false) => quote! {
					____assert_types_array
				},
				(false, true, false) => quote! {
					____assert_types_input_array
				},
				(false, false, false) => quote! {
					____assert_types
				},
				(true, true, true) => quote! {
					____assert_types_array_input_flatten
				},
				(true, false, true) => quote! {
					____assert_types_array_flatten
				},
				(false, true, true) => quote! {
					____assert_types_input_array_flatten
				},
				(false, false, true) => quote! {
					____assert_types_flatten
				},
			};

			quote_spanned! { name.span() => #assert_fn(#struct_ident().#name, #name); }
		});

		quote! {
			#[inline(always)]
			#[doc(hidden)]
			fn ____make<T>() -> T {
				unreachable!()
			}

			#[inline(always)]
			#[doc(hidden)]
			fn ____assert_types<T>(_: T, _: impl #path::__AssertFilterBounds<T>) -> ! {
				unreachable!()
			}

			#[inline(always)]
			#[doc(hidden)]
			fn ____assert_types_array<T>(_: impl #path::__ArrayLike<T>, _: impl #path::__AssertFilterBounds<T>) -> ! {
				unreachable!()
			}

			#[inline(always)]
			#[doc(hidden)]
			fn ____assert_types_array_input<T, U: #path::__AssertFilterBounds<T>>(_: impl #path::__ArrayLike<T>, _: impl #path::__ArrayLike<U>) -> ! {
				unreachable!()
			}

			#[inline(always)]
			#[doc(hidden)]
			fn ____assert_types_input_array<T, U: #path::__AssertFilterBounds<T>>(_: T, _: impl #path::__ArrayLike<U>) -> ! {
				unreachable!()
			}

			#[inline(always)]
			#[doc(hidden)]
			fn ____assert_types_flatten<T>(_: T, _: impl #path::__AssertFilterBoundsFlatten<T>) -> ! {
				unreachable!()
			}

			#[inline(always)]
			#[doc(hidden)]
			fn ____assert_types_array_flatten<T>(_: impl #path::__ArrayLike<T>, _: impl #path::__AssertFilterBoundsFlatten<T>) -> ! {
				unreachable!()
			}

			#[inline(always)]
			#[doc(hidden)]
			fn ____assert_types_array_input_flatten<T, U: #path::__AssertFilterBoundsFlatten<T>>(_: impl #path::__ArrayLike<T>, _: impl #path::__ArrayLike<U>) -> ! {
				unreachable!()
			}

			#[inline(always)]
			#[doc(hidden)]
			fn ____assert_types_input_array_flatten<T, U: #path::__AssertFilterBoundsFlatten<T>>(_: T, _: impl #path::__ArrayLike<U>) -> ! {
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
			#path::Value::<#ty>::new({
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

pub(crate) fn proc_macro(input: proc_macro2::TokenStream) -> Result<proc_macro2::TokenStream, syn::Error> {
	map_expr(syn::parse2::<syn::Expr>(input)?)
}

pub fn map_expr(ast: syn::Expr) -> Result<proc_macro2::TokenStream, syn::Error> {
	match ast {
		syn::Expr::Struct(ast) => Ok(mapper_struct(ast)?),
		_ => Err(syn::Error::new(ast.span(), "Expected struct")),
	}
}
