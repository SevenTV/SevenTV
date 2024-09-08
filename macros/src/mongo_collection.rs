use darling::FromMeta;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::parse::Parse;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{Data, Fields, Ident, LitInt, LitStr, Meta, Token};

// #[derive(MongoCollection, serde::Deserialize, serde::Serialize)]
// #[mongo(collection_name = "invoices")]
// #[mongo(index(fields("a.b"=1, "c.d"=2), unique))]
// pub struct Invoice {
//    	#[mongo(id)]
//    	#[serde(rename = "_id")]
//    	pub id: InvoiceId,
//   	pub items: Vec<ProductId>,
//  	pub customer_id: CustomerId,
//  	pub user_id: UserId,
// 	pub paypal_payment_ids: Vec<String>,
// 	pub status: InvoiceStatus,
// 	pub note: Option<String>,
//    	pub created_at: i64,
//   	pub updated_at: i64,
//  	pub search_updated_at: i64,
// }

#[derive(Debug)]
struct StructAttributes {
	ident: syn::Ident,
	generics: syn::Generics,
	fields: Vec<FieldAttributes>,

	index: Vec<Index>,
	crate_: Option<syn::Path>,
	collection_name: Option<syn::LitStr>,
	searchable_collection: Option<syn::Type>,
}

fn parse_nested(meta: Meta) -> syn::Result<Meta> {
	match meta {
		Meta::List(list) => {
			let meta = syn::parse2::<Meta>(list.tokens)?;
			Ok(meta)
		}
		_ => Err(syn::Error::new(meta.span(), "invalid mongo attribute")),
	}
}

impl StructAttributes {
	fn from_derive_input(input: &syn::DeriveInput) -> syn::Result<Self> {
		let Data::Struct(data) = &input.data else {
			return Err(syn::Error::new(Span::call_site(), "expected named struct"));
		};

		let Fields::Named(fields) = &data.fields else {
			return Err(syn::Error::new(Span::call_site(), "expected named struct"));
		};

		let attrs = input
			.attrs
			.iter()
			.filter(|attr| attr.path().is_ident("mongo"))
			.map(|attr| parse_nested(attr.meta.clone()));

		let mut index = Vec::new();
		let mut crate_ = None;
		let mut collection_name = None;
		let mut searchable_collection = None;

		for attr in attrs {
			let attr = attr?;
			match attr {
				Meta::List(list) if list.path.is_ident("index") => {
					if list.path.is_ident("index") {
						index.push(syn::parse2(list.tokens.clone())?);
					}
				}
				Meta::NameValue(meta) if meta.path.is_ident("crate") => {
					crate_ = Some(syn::Path::from_expr(&meta.value)?);
				}
				Meta::NameValue(meta) if meta.path.is_ident("collection_name") => {
					collection_name = Some(syn::LitStr::from_expr(&meta.value)?);
				}
				Meta::NameValue(meta) if meta.path.is_ident("search") => {
					searchable_collection = Some(syn::Type::from_expr(&meta.value)?);
				}
				_ => return Err(syn::Error::new(attr.span(), "invalid mongo attribute")),
			}
		}

		Ok(Self {
			ident: input.ident.clone(),
			generics: input.generics.clone(),
			fields: fields
				.named
				.iter()
				.map(FieldAttributes::from_field)
				.collect::<syn::Result<_>>()?,
			index,
			crate_,
			collection_name,
			searchable_collection,
		})
	}
}

#[derive(Debug)]
struct FieldAttributes {
	ident: syn::Ident,
	ty: syn::Type,
	id: bool,
}

impl FieldAttributes {
	fn from_field(field: &syn::Field) -> syn::Result<Self> {
		let ident = field
			.ident
			.as_ref()
			.ok_or_else(|| syn::Error::new(field.span(), "expected field to have an identifier"))?;

		let attrs = field
			.attrs
			.iter()
			.filter(|attr| attr.path().is_ident("mongo"))
			.map(|attr| parse_nested(attr.meta.clone()));

		let mut id = None;

		for attr in attrs {
			let attr = attr?;
			match attr {
				meta if meta.path().is_ident("id") => {
					if id.is_some() {
						return Err(syn::Error::new(meta.span(), "duplicate id field"));
					}

					match meta {
						Meta::Path(meta) => {
							if meta.is_ident("id") {
								id = Some(meta.clone());
							}
						}
						_ => {
							return Err(syn::Error::new(meta.span(), "id must be a path"));
						}
					}
				}
				_ => {
					return Err(syn::Error::new(attr.span(), "invalid mongo attribute"));
				}
			}
		}

		Ok(Self {
			ident: ident.clone(),
			ty: field.ty.clone(),
			id: id.is_some(),
		})
	}
}

#[derive(Debug, Clone)]
enum FieldName {
	Ident(syn::Ident),
	Str(LitStr),
}

impl Parse for FieldName {
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
		let lookahead = input.lookahead1();
		if lookahead.peek(Ident) {
			Ok(Self::Ident(input.parse()?))
		} else if lookahead.peek(LitStr) {
			Ok(Self::Str(input.parse()?))
		} else {
			Err(lookahead.error())
		}
	}
}

#[derive(Debug, Clone)]
struct Field {
	name: FieldName,
	_eq: Token![=],
	value: LitInt,
}

impl Parse for Field {
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
		Ok(Self {
			name: input.parse()?,
			_eq: input.parse()?,
			value: input.parse()?,
		})
	}
}

#[derive(Debug, Clone)]
struct IndexFields(Vec<Field>);

impl Parse for IndexFields {
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
		let input = Punctuated::<Field, Token![,]>::parse_terminated(input)?;
		Ok(Self(input.into_iter().collect()))
	}
}

#[derive(Debug, Clone)]
struct Index {
	fields: IndexFields,
	unique: Option<syn::LitBool>,
	background: Option<syn::LitBool>,
	expire_after: Option<syn::LitInt>,
	sparse: Option<syn::LitBool>,
	from_fn: Option<syn::Path>,
	name: Option<syn::LitStr>,
}

impl Parse for Index {
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
		let fields = Punctuated::<Meta, Token![,]>::parse_terminated(input)?;

		let mut _fields = None;
		let mut _unique = None;
		let mut _background = None;
		let mut _expire_after = None;
		let mut _sparse = None;
		let mut _from_fn = None;
		let mut _name = None;

		for meta in fields {
			match meta {
				meta if meta.path().is_ident("fields") => {
					if _fields.is_some() {
						return Err(syn::Error::new(meta.path().span(), "duplicate fields"));
					}

					match meta {
						Meta::List(list) => {
							_fields = Some(syn::parse2(list.tokens)?);
						}
						_ => return Err(syn::Error::new(meta.span(), "fields must be a list")),
					}
				}
				meta if meta.path().is_ident("name") => {
					if _name.is_some() {
						return Err(syn::Error::new(meta.span(), "duplicate name"));
					}

					match meta {
						Meta::NameValue(_) => {
							_name = Some(syn::LitStr::from_meta(&meta)?);
						}
						_ => return Err(syn::Error::new(meta.span(), "name must be a name value")),
					}
				}
				meta if meta.path().is_ident("unique") => {
					if _unique.is_some() {
						return Err(syn::Error::new(meta.span(), "duplicate unique"));
					}

					match meta {
						Meta::Path(_) => {
							_unique = Some(syn::LitBool::new(true, meta.span()));
						}
						Meta::NameValue(_) => {
							_unique = Some(syn::LitBool::from_meta(&meta)?);
						}
						_ => return Err(syn::Error::new(meta.span(), "unique must be a path or name value")),
					}
				}
				meta if meta.path().is_ident("background") => {
					if _background.is_some() {
						return Err(syn::Error::new(meta.span(), "duplicate background"));
					}

					match meta {
						Meta::Path(_) => {
							_background = Some(syn::LitBool::new(true, meta.span()));
						}
						Meta::NameValue(_) => {
							_background = Some(syn::LitBool::from_meta(&meta)?);
						}
						_ => return Err(syn::Error::new(meta.span(), "background must be a path or name value")),
					}
				}
				meta if meta.path().is_ident("expire_after") => {
					if _expire_after.is_some() {
						return Err(syn::Error::new(meta.span(), "duplicate expire_after"));
					}

					match meta {
						Meta::NameValue(_) => {
							_expire_after = Some(syn::LitInt::from_meta(&meta)?);
						}
						_ => return Err(syn::Error::new(meta.span(), "expire_after must be a name value")),
					}
				}
				meta if meta.path().is_ident("sparse") => {
					if _sparse.is_some() {
						return Err(syn::Error::new(meta.span(), "duplicate sparse"));
					}

					match meta {
						Meta::Path(_) => {
							_sparse = Some(syn::LitBool::new(true, meta.span()));
						}
						Meta::NameValue(_) => {
							_sparse = Some(syn::LitBool::from_meta(&meta)?);
						}
						_ => return Err(syn::Error::new(meta.span(), "sparse must be a path or name value")),
					}
				}
				meta if meta.path().is_ident("from_fn") => {
					if _from_fn.is_some() {
						return Err(syn::Error::new(meta.span(), "duplicate from_fn"));
					}

					match meta {
						Meta::NameValue(meta) => {
							_from_fn = Some(syn::Path::from_expr(&meta.value)?);
						}
						_ => return Err(syn::Error::new(meta.span(), "from_fn must be a name value")),
					}
				}
				_ => {
					return Err(syn::Error::new(meta.span(), "invalid mongo attribute"));
				}
			}
		}

		if let Some(from_fn) = &_from_fn {
			if _unique.is_some() || _background.is_some() || _expire_after.is_some() || _sparse.is_some() {
				return Err(syn::Error::new(
					from_fn.span(),
					"from_fn cannot be used with any of the other options",
				));
			}
		}

		Ok(Self {
			fields: _fields.ok_or_else(|| syn::Error::new(Span::call_site(), "missing fields"))?,
			unique: _unique,
			name: _name,
			background: _background,
			expire_after: _expire_after,
			sparse: _sparse,
			from_fn: _from_fn,
		})
	}
}

pub fn derive(input: TokenStream) -> syn::Result<TokenStream> {
	let input = syn::parse2(input)?;
	let input = StructAttributes::from_derive_input(&input)?;

	let collection_name = input
		.collection_name
		.unwrap_or_else(|| syn::LitStr::new(&input.ident.to_string(), input.ident.span()));

	let crate_ = &input
		.crate_
		.unwrap_or_else(|| syn::parse_str("crate::database::types").unwrap());

	let id = input.fields.iter().filter(|field| field.id).collect::<Vec<_>>();

	if id.len() > 1 {
		return Err(syn::Error::new(
			id[1].ident.span(),
			"only one field can be the default sort field",
		));
	}

	let Some(id) = id.first() else {
		return Err(syn::Error::new(Span::call_site(), "at least one field must be an id field"));
	};

	let id_ident = &id.ident;
	let id_ty = &id.ty;
	let ident = &input.ident;

	let index_fields = input.index.iter().map(|index| {
		let items = index.fields.0.iter().map(|field| {
			let key = match &field.name {
				FieldName::Ident(ident) => ident.to_string(),
				FieldName::Str(s) => s.value(),
			};

			let value = &field.value;

			quote! {
				#key: #value
			}
		});

		let options = index
			.from_fn
			.as_ref()
			.map(|from_fn| {
				quote! {
					#from_fn()
				}
			})
			.unwrap_or_else(|| {
				let mut builder = quote! {
					mongodb::options::IndexOptions::builder()
				};

				if let Some(unique) = &index.unique {
					builder = quote! {
						#builder.unique(#unique)
					}
				}

				if let Some(background) = &index.background {
					builder = quote! {
						#builder.background(#background)
					}
				}

				if let Some(expire_after) = &index.expire_after {
					builder = quote! {
						#builder.expire_after(::std::time::Duration::from_secs(#expire_after))
					}
				}

				if let Some(sparse) = &index.sparse {
					builder = quote! {
						#builder.sparse(#sparse)
					}
				}

				if let Some(name) = &index.name {
					builder = quote! {
						#builder.name(#name)
					}
				}

				quote! {
					#builder.build()
				}
			});

		quote! {
			#crate_::mongodb::IndexModel::builder()
				.keys(#crate_::mongodb::bson::doc! {
					#(#items),*
				})
				.options(#options)
				.build()
		}
	});

	let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

	let search_impl = if let Some(searchable_collection) = input.searchable_collection {
		quote! {
			impl #impl_generics #crate_::SearchableMongoCollection for #ident #ty_generics #where_clause {
				type Typesense = #searchable_collection;
			}
		}
	} else {
		quote! {}
	};

	Ok(quote! {
		#[allow(clippy::all)]
		#[doc(hidden)]
		const _: () = {
			impl #impl_generics #crate_::MongoCollection for #ident #ty_generics #where_clause {
				const COLLECTION_NAME: &'static str = #collection_name;

				type Id = #id_ty;

				fn id(&self) -> Self::Id {
					self.#id_ident.clone()
				}

				fn indexes() -> Vec<#crate_::mongodb::IndexModel> {
					vec![
						#(#index_fields),*
					]
				}
			}

			#search_impl
		};
	})
}
