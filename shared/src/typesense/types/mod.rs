pub mod automod;
pub mod badge;
pub mod duration_unit;
pub mod emote;
pub mod emote_moderation_request;
pub mod emote_set;
pub mod event;
pub mod page;
pub mod paint;
pub mod product;
pub mod role;
pub mod ticket;
pub mod user;

pub use macros::TypesenseCollection;
pub use typesense_codegen;

pub trait TypesenseCollection: Send + Sync {
	const COLLECTION_NAME: &'static str;

	type Id: std::fmt::Debug + Clone + serde::Serialize + serde::de::DeserializeOwned;

	fn schema() -> typesense_codegen::models::CollectionSchema;
	fn fields() -> Vec<typesense_codegen::models::Field>;
}

struct TypesenseGenericCollection {
	name: &'static str,
	schema: typesense_codegen::models::CollectionSchema,
}

impl TypesenseGenericCollection {
	fn new<C: TypesenseCollection>() -> Self {
		Self {
			name: C::COLLECTION_NAME,
			schema: C::schema(),
		}
	}

	pub fn determine_migration(&self, _resp: CollectionResponse) -> anyhow::Result<Option<CollectionUpdateSchema>> {
		tracing::warn!("collection migration not implemented");
		Ok(None)
	}

	#[tracing::instrument(skip(self, config), fields(collection = self.name))]
	pub async fn init(self, config: &typesense_codegen::apis::configuration::Configuration) -> anyhow::Result<()> {
		let result = match typesense_codegen::apis::collections_api::get_collection(config, self.name).await {
			Ok(result) => result,
			Err(typesense_codegen::apis::Error::ResponseError(err)) if err.status == hyper::StatusCode::NOT_FOUND => {
				tracing::debug!("collection not found, creating");
				typesense_codegen::apis::collections_api::create_collection(config, self.schema.clone()).await?
			}
			Err(err) => {
				anyhow::bail!("failed to get collection: {err}");
			}
		};

		if let Some(migration) = self.determine_migration(result)? {
			tracing::debug!("applying migration");
			typesense_codegen::apis::collections_api::update_collection(config, self.name, migration).await?;
		}

		Ok(())
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub enum FieldType {
	String,
	ArrayString,
	Int32,
	ArrayInt32,
	Int64,
	ArrayInt64,
	Float,
	ArrayFloat,
	Bool,
	ArrayBool,
	Geopoint,
	ArrayGeopoint,
	Object,
	ArrayObject,
	AutoString,
	Image,
	Auto,
}

pub trait TypesenseType {
	fn typesense_type() -> FieldType;
	fn optional() -> bool {
		false
	}
}

impl<T: TypesenseType> TypesenseType for Option<T> {
	fn typesense_type() -> FieldType {
		T::typesense_type()
	}

	fn optional() -> bool {
		true
	}
}

impl<T: TypesenseType> TypesenseType for Vec<T> {
	fn typesense_type() -> FieldType {
		match T::typesense_type() {
			FieldType::String => FieldType::ArrayString,
			FieldType::Int32 => FieldType::ArrayInt32,
			FieldType::Int64 => FieldType::ArrayInt64,
			FieldType::Float => FieldType::ArrayFloat,
			FieldType::Bool => FieldType::ArrayBool,
			FieldType::Geopoint => FieldType::ArrayGeopoint,
			FieldType::Object => FieldType::ArrayObject,
			FieldType::AutoString => FieldType::ArrayString,
			FieldType::Image => FieldType::ArrayString,
			FieldType::Auto => FieldType::ArrayString,
			r => r,
		}
	}

	fn optional() -> bool {
		true
	}
}

impl std::fmt::Display for FieldType {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			FieldType::String => write!(f, "string"),
			FieldType::ArrayString => write!(f, "string[]"),
			FieldType::Int32 => write!(f, "int32"),
			FieldType::ArrayInt32 => write!(f, "int32[]"),
			FieldType::Int64 => write!(f, "int64"),
			FieldType::ArrayInt64 => write!(f, "int64[]"),
			FieldType::Float => write!(f, "float"),
			FieldType::ArrayFloat => write!(f, "float[]"),
			FieldType::Bool => write!(f, "bool"),
			FieldType::ArrayBool => write!(f, "bool[]"),
			FieldType::Geopoint => write!(f, "geopoint"),
			FieldType::ArrayGeopoint => write!(f, "geopoint[]"),
			FieldType::Object => write!(f, "object"),
			FieldType::ArrayObject => write!(f, "object[]"),
			FieldType::AutoString => write!(f, "string*"),
			FieldType::Image => write!(f, "image"),
			FieldType::Auto => write!(f, "auto"),
		}
	}
}

macro_rules! impl_typesense_type {
	($type:ty, $field_type:ident) => {
		impl $crate::typesense::types::TypesenseType for $type {
			fn typesense_type() -> $crate::typesense::types::FieldType {
				$crate::typesense::types::FieldType::$field_type
			}

			fn optional() -> bool {
				false
			}
		}
	};
}

pub(crate) use impl_typesense_type;
use typesense_codegen::models::{CollectionResponse, CollectionUpdateSchema};

impl_typesense_type!(String, String);
impl_typesense_type!(i32, Int32);
impl_typesense_type!(i64, Int64);
impl_typesense_type!(f32, Float);
impl_typesense_type!(f64, Float);
impl_typesense_type!(bool, Bool);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct TypesenseString<T>(pub T);

impl<T: std::fmt::Display> std::fmt::Display for TypesenseString<T> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.0.fmt(f)
	}
}

impl<T: std::str::FromStr> std::str::FromStr for TypesenseString<T> {
	type Err = <T as std::str::FromStr>::Err;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(Self(s.parse()?))
	}
}

impl<T: std::fmt::Display> serde::Serialize for TypesenseString<T> {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		self.to_string().serialize(serializer)
	}
}

impl<'de, T: std::str::FromStr> serde::Deserialize<'de> for TypesenseString<T>
where
	T::Err: std::fmt::Display,
{
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		let s = String::deserialize(deserializer)?;
		s.parse().map_err(serde::de::Error::custom)
	}
}

impl<T> TypesenseType for TypesenseString<T> {
	fn typesense_type() -> FieldType {
		FieldType::String
	}
}

impl<T> From<T> for TypesenseString<T> {
	fn from(value: T) -> Self {
		Self(value)
	}
}

impl<T> TypesenseString<T> {
	pub fn into_inner(self) -> T {
		self.0
	}
}

fn typesense_collections() -> impl IntoIterator<Item = TypesenseGenericCollection> {
	std::iter::empty()
		.chain(event::typesense_collections())
		.chain(automod::typesense_collections())
		.chain(page::typesense_collections())
		.chain(ticket::typesense_collections())
		.chain(user::typesense_collections())
		.chain(product::typesense_collections())
		.chain(role::typesense_collections())
		.chain(paint::typesense_collections())
		.chain(badge::typesense_collections())
		.chain(emote::typesense_collections())
		.chain(emote_moderation_request::typesense_collections())
		.chain(emote_set::typesense_collections())
}

pub async fn init_typesense(client: &typesense_codegen::apis::configuration::Configuration) -> anyhow::Result<()> {
	let collections = typesense_collections().into_iter().collect::<Vec<_>>();

	for collection in collections {
		collection.init(client).await?;
	}

	Ok(())
}
