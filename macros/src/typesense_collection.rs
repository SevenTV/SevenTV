use darling::{ast, FromDeriveInput, FromField, FromMeta};
use proc_macro2::{Span, TokenStream};
use quote::quote;

// #[derive(TypesenseCollection)]
// #[typesense(collection_name = "invoices")]
// pub struct Invoice {
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
//
// trait TypesenseCollection {
//    const COLLECTION_NAME: &'static str;
//    fn schema() -> typesense_codegen::models::CollectionSchema;
// }
//
// pub enum FieldType {
// 	...
// }
//
// trait TypesenseType {
// 		fn typesense_type() -> typesense_codegen::models::FieldType;
// }

#[derive(FromDeriveInput, Debug)]
#[darling(attributes(typesense), supports(struct_named))]
struct StructAttributes {
	ident: syn::Ident,
	generics: syn::Generics,
	data: ast::Data<(), FieldAttributes>,

	#[darling(default)]
	nested_fields: Option<syn::LitBool>,

	#[darling(default)]
	index_symbols: Option<Vec<syn::LitStr>>,

	#[darling(default)]
	token_separators: Option<Vec<syn::LitStr>>,

	#[darling(default, rename = "crate")]
	crate_: Option<syn::Path>,

	#[darling(default)]
	collection_name: Option<syn::LitStr>,
}

#[derive(FromField, Debug)]
#[darling(attributes(typesense))]
struct FieldAttributes {
	ident: Option<syn::Ident>,
	ty: syn::Type,

	#[darling(default)]
	default_sort: bool,

	#[darling(default)]
	id: bool,

	#[darling(default)]
	name: Option<syn::LitStr>,
	#[darling(default)]
	field: Option<FieldType>,
	#[darling(default)]
	optional: Option<bool>,
	#[darling(default)]
	facet: Option<syn::LitBool>,
	#[darling(default)]
	index: Option<syn::LitBool>,
	#[darling(default)]
	locale: Option<syn::LitStr>,
	#[darling(default)]
	sort: Option<syn::LitBool>,
	#[darling(default)]
	infix: Option<syn::LitBool>,
	#[darling(default)]
	num_dim: Option<syn::LitInt>,
	#[darling(default)]
	drop: Option<syn::LitBool>,
	#[darling(default)]
	nested: Option<Nested>,
}

#[derive(Debug)]
enum Nested {
	True,
	From(syn::Type),
}

impl FromMeta for Nested {
	fn from_word() -> darling::Result<Self> {
		Ok(Nested::True)
	}

	fn from_value(value: &syn::Lit) -> darling::Result<Self> {
		match value {
			syn::Lit::Str(value) => Ok(Nested::From(syn::parse_str(&value.value())?)),
			_ => Err(darling::Error::unsupported_format("expected a string")),
		}
	}
}

#[derive(Debug, Clone, Copy, FromMeta)]
enum FieldType {
	#[darling(rename = "string")]
	String,
	#[darling(rename = "int32")]
	Int32,
	#[darling(rename = "int64")]
	Int64,
	#[darling(rename = "float")]
	Float,
	#[darling(rename = "bool")]
	Bool,
	#[darling(rename = "geopoint")]
	Geopoint,
	#[darling(rename = "object")]
	Object,
	#[darling(rename = "string*")]
	AutoString,
	#[darling(rename = "image")]
	Image,
	#[darling(rename = "auto")]
	Auto,
}

fn expand_optional<T: quote::ToTokens>(optional: Option<T>) -> TokenStream {
	optional
		.map(|value| quote! { Some(#value.into()) })
		.unwrap_or_else(|| quote! { None })
}

pub fn derive(input: TokenStream) -> syn::Result<TokenStream> {
	let input = syn::parse2(input)?;
	let input = StructAttributes::from_derive_input(&input)?;

	let collection_name = input
		.collection_name
		.unwrap_or_else(|| syn::LitStr::new(&input.ident.to_string(), input.ident.span()));
	let crate_ = input
		.crate_
		.unwrap_or_else(|| syn::parse_str("crate::typesense::types").unwrap());

	let fields = input.data.take_struct().expect("expected struct").fields;

	let default_sort = fields.iter().filter(|field| field.default_sort).collect::<Vec<_>>();

	if default_sort.len() > 1 {
		return Err(syn::Error::new(
			default_sort[1].ident.as_ref().unwrap().span(),
			"only one field can be the default sort field",
		));
	}

	let id = fields.iter().filter(|field| field.id).collect::<Vec<_>>();

	if id.len() > 1 {
		return Err(syn::Error::new(
			id[1].ident.as_ref().unwrap().span(),
			"only one field can be the id field",
		));
	}

	let id = id
		.first()
		.map(|f| f.ty.clone())
		.or_else(|| {
			fields
				.iter()
				.find(|f| *f.ident.as_ref().unwrap() == "id")
				.map(|f| f.ty.clone())
		})
		.ok_or_else(|| syn::Error::new(Span::call_site(), "id field not found"))?;

	let default_sort = expand_optional(
		default_sort
			.iter()
			.map(|field| {
				let name = field.name.clone().unwrap_or_else(|| {
					syn::LitStr::new(
						&field.ident.as_ref().unwrap().to_string(),
						field.ident.as_ref().unwrap().span(),
					)
				});
				quote! {
					#name.to_string()
				}
			})
			.next(),
	);

	let schema_fields = fields.iter().map(|field| {
		let name = field.name.clone().unwrap_or_else(|| {
			syn::LitStr::new(
				&field.ident.as_ref().unwrap().to_string(),
				field.ident.as_ref().unwrap().span(),
			)
		});

		let ty = field
			.field
			.map(|r#type| {
				let ty = match r#type {
					FieldType::String => quote! { FieldType::String },
					FieldType::Int32 => quote! { FieldType::Int32 },
					FieldType::Int64 => quote! { FieldType::Int64 },
					FieldType::Float => quote! { FieldType::Float },
					FieldType::Bool => quote! { FieldType::Bool },
					FieldType::Geopoint => quote! { FieldType::Geopoint },
					FieldType::Object => quote! { FieldType::Object },
					FieldType::AutoString => quote! { FieldType::AutoString },
					FieldType::Image => quote! { FieldType::Image },
					FieldType::Auto => quote! { FieldType::Auto },
				};

				quote! {
					#crate_::#ty
				}
			})
			.unwrap_or_else(|| {
				let ty = &field.ty;
				quote! {
					<#ty as #crate_::TypesenseType>::typesense_type()
				}
			});

		let optional = if field.optional.is_none() && field.field.is_none() {
			let ty = &field.ty;
			quote! { Some(<#ty as #crate_::TypesenseType>::optional()) }
		} else {
			expand_optional(field.optional.as_ref())
		};

		let facet = expand_optional(field.facet.as_ref());
		let index = expand_optional(field.index.as_ref());
		let locale = expand_optional(field.locale.as_ref());
		let sort = expand_optional(field.sort.as_ref());
		let infix = expand_optional(field.infix.as_ref());
		let num_dim = expand_optional(field.num_dim.as_ref());
		let drop = expand_optional(field.drop.as_ref());

		let nested = if let Some(nested) = &field.nested {
			let ty = match nested {
				Nested::True => &field.ty,
				Nested::From(ty) => ty,
			};
			quote! {
				for mut nested_field in <#ty as #crate_::TypesenseCollection>::fields() {
					nested_field.name = format!("{}.{}", #name, nested_field.name);
					fields.push(nested_field);
				}
			}
		} else {
			quote! {}
		};

		quote! {
			fields.push(#crate_::typesense_rs::models::Field {
				name: #name.to_string(),
				r#type: #ty.to_string(),
				optional: #optional,
				facet: #facet,
				index: #index,
				locale: #locale,
				sort: #sort,
				infix: #infix,
				num_dim: #num_dim,
				drop: #drop,
				..Default::default()
			});
			#nested
		}
	});

	let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
	let ident = &input.ident;

	let token_separators = expand_optional(input.token_separators.map(|token_separators| {
		let token_separators = token_separators.iter().map(|token_separator| {
			quote! {
				#token_separator.to_string()
			}
		});

		quote! {
			vec![
				#(#token_separators,)*
			]
		}
	}));

	let index_symbols = expand_optional(input.index_symbols.map(|index_symbols| {
		let index_symbols = index_symbols.iter().map(|index_symbol| {
			quote! {
				#index_symbol.to_string()
			}
		});

		quote! {
			vec![
				#(#index_symbols,)*
			]
		}
	}));

	let nested_fields = expand_optional(input.nested_fields);

	Ok(quote! {
		impl #impl_generics #crate_::TypesenseCollection for #ident #ty_generics #where_clause {
			const COLLECTION_NAME: &'static str = #collection_name;

			type Id = #id;

			fn fields() -> Vec<#crate_::typesense_rs::models::Field> {
				let mut fields = Vec::new();
				#(#schema_fields)*
				fields
			}

			fn schema() -> #crate_::typesense_rs::models::CollectionSchema {
				#crate_::typesense_rs::models::CollectionSchema {
					name: Self::COLLECTION_NAME.into(),
					fields: Self::fields(),
					default_sorting_field: #default_sort,
					token_separators: #token_separators,
					symbols_to_index: #index_symbols,
					enable_nested_fields: #nested_fields,
					..Default::default()
				}
			}
		}
	})
}
