use proc_macro::TokenStream;

mod mongo_collection;
mod typesense_collection;

#[proc_macro_derive(TypesenseCollection, attributes(typesense))]
pub fn derive_typesense_collection(input: TokenStream) -> TokenStream {
	let tokens = match typesense_collection::derive(input.into()) {
		Ok(output) => output,
		Err(err) => err.to_compile_error(),
	};

	tokens.into()
}

#[proc_macro_derive(MongoCollection, attributes(mongo))]
pub fn derive_mongo_collection(input: TokenStream) -> TokenStream {
	let tokens = match mongo_collection::derive(input.into()) {
		Ok(output) => output,
		Err(err) => err.to_compile_error(),
	};

	tokens.into()
}
