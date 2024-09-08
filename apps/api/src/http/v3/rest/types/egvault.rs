use shared::database::paint::PaintId;
use shared::database::product::subscription::SubscriptionState;
use shared::database::product::{SubscriptionProductKind, SubscriptionProductVariant};

#[derive(Debug, serde::Serialize)]
pub struct Subscription {
	pub id: String,
	pub provider: Option<Provider>,
	/// Stripe product id
	pub product_id: String,
	/// Stripe price id
	pub plan: String,
	/// always 1
	pub seats: u32,
	/// Id of the user who is subscribed
	pub subscriber_id: String,
	/// Id of the user who is paying the subscription
	pub customer_id: String,
	pub started_at: chrono::DateTime<chrono::Utc>,
	pub ended_at: Option<chrono::DateTime<chrono::Utc>>,
	pub cycle: SubscriptionCycle,
	pub renew: bool,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Provider {
	Paypal,
	Stripe,
}

#[derive(Debug, serde::Serialize)]
pub struct SubscriptionCycle {
	pub timestamp: chrono::DateTime<chrono::Utc>,
	pub unit: Option<SubscriptionCycleUnit>,
	pub value: u32,
	pub status: SubscriptionCycleStatus,
	pub internal: bool,
	pub pending: bool,
	pub trial_end: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SubscriptionCycleUnit {
	/// only one subscription has that
	Day,
	Month,
	Year,
}

impl From<SubscriptionProductKind> for SubscriptionCycleUnit {
	fn from(value: SubscriptionProductKind) -> Self {
		match value {
			SubscriptionProductKind::Monthly => Self::Month,
			SubscriptionProductKind::Yearly => Self::Year,
		}
	}
}

#[derive(Debug, serde::Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SubscriptionCycleStatus {
	Ongoing,
	Ended,
	Canceled,
}

impl From<SubscriptionState> for SubscriptionCycleStatus {
	fn from(value: SubscriptionState) -> Self {
		match value {
			SubscriptionState::Active => Self::Ongoing,
			SubscriptionState::Ended => Self::Ended,
			SubscriptionState::CancelAtEnd => Self::Canceled,
		}
	}
}

#[derive(Debug, serde::Serialize)]
pub struct Product {
	/// "subscription"
	pub name: String,
	pub plans: Vec<Plan>,
	pub current_paints: Vec<PaintId>,
}

#[derive(Debug, serde::Serialize)]
pub struct Plan {
	pub interval_unit: SubscriptionCycleUnit,
	/// always 1
	pub interval: u32,
	pub price: u64,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub discount: Option<f64>,
	pub currency: stripe::Currency,
	pub currency_symbol: &'static str,
	pub currency_name: &'static str,
}

const fn currency_symbol(currency: stripe::Currency) -> (&'static str, &'static str) {
	match currency {
		stripe::Currency::BYN => ("Br", "Belarusian Ruble"),
		stripe::Currency::MMK => ("K", "Myanmar Kyat"),
		stripe::Currency::AED => ("د.إ", "United Arab Emirates Dirham"),
		stripe::Currency::AFN => ("؋", "Afghan Afghani"),
		stripe::Currency::ALL => ("L", "Albanian Lek"),
		stripe::Currency::AMD => ("֏", "Armenian Dram"),
		stripe::Currency::ANG => ("ƒ", "Netherlands Antillean Gulden"),
		stripe::Currency::AOA => ("Kz", "Angolan Kwanza"),
		stripe::Currency::ARS => ("AR$", "Argentine Peso"),
		stripe::Currency::AUD => ("AU$", "Australian Dollar"),
		stripe::Currency::AWG => ("ƒ", "Aruban Florin"),
		stripe::Currency::AZN => ("₼", "Azerbaijani Manat"),
		stripe::Currency::BAM => ("KM", "Bosnia & Herzegovina Convertible Mark"),
		stripe::Currency::BBD => ("BBD$", "Barbadian Dollar"),
		stripe::Currency::BDT => ("৳", "Bangladeshi Taka"),
		stripe::Currency::BGN => ("лв", "Bulgarian Lev"),
		stripe::Currency::BIF => ("FBu", "Burundian Franc"),
		stripe::Currency::BMD => ("BM$", "Bermudian Dollar"),
		stripe::Currency::BND => ("BN$", "Brunei Dollar"),
		stripe::Currency::BOB => ("Bs.", "Bolivian Boliviano"),
		stripe::Currency::BRL => ("R$", "Brazilian Real"),
		stripe::Currency::BSD => ("BS$", "Bahamian Dollar"),
		stripe::Currency::BWP => ("P", "Botswana Pula"),
		stripe::Currency::BZD => ("BZ$", "Belize Dollar"),
		stripe::Currency::CAD => ("CA$", "Canadian Dollar"),
		stripe::Currency::CDF => ("FC", "Congolese Franc"),
		stripe::Currency::CHF => ("CHF", "Swiss Franc"),
		stripe::Currency::CLP => ("CLP$", "Chilean Peso"),
		stripe::Currency::CNY => ("¥", "Chinese Renminbi Yuan"),
		stripe::Currency::COP => ("CO$", "Colombian Peso"),
		stripe::Currency::CRC => ("₡", "Costa Rican Colón"),
		stripe::Currency::CVE => ("CVE$", "Cape Verdean Escudo"),
		stripe::Currency::CZK => ("Kč", "Czech Koruna"),
		stripe::Currency::DJF => ("Fdj", "Djiboutian Franc"),
		stripe::Currency::DKK => ("kr", "Danish Krone"),
		stripe::Currency::DOP => ("RD$", "Dominican Peso"),
		stripe::Currency::DZD => ("دج", "Algerian Dinar"),
		stripe::Currency::EEK => ("kr", "Estonian Kroon (pre-Euro)"),
		stripe::Currency::EGP => ("£", "Egyptian Pound"),
		stripe::Currency::ETB => ("Br", "Ethiopian Birr"),
		stripe::Currency::EUR => ("€", "Euro"),
		stripe::Currency::FJD => ("FJ$", "Fijian Dollar"),
		stripe::Currency::FKP => ("£", "Falkland Islands Pound"),
		stripe::Currency::GBP => ("£", "British Pound"),
		stripe::Currency::GEL => ("₾", "Georgian Lari"),
		stripe::Currency::GIP => ("£", "Gibraltar Pound"),
		stripe::Currency::GMD => ("D", "Gambian Dalasi"),
		stripe::Currency::GNF => ("FG", "Guinean Franc"),
		stripe::Currency::GTQ => ("Q", "Guatemalan Quetzal"),
		stripe::Currency::GYD => ("GY$", "Guyanese Dollar"),
		stripe::Currency::HKD => ("HK$", "Hong Kong Dollar"),
		stripe::Currency::HNL => ("L", "Honduran Lempira"),
		stripe::Currency::HRK => ("kn", "Croatian Kuna"),
		stripe::Currency::HTG => ("G", "Haitian Gourde"),
		stripe::Currency::HUF => ("Ft", "Hungarian Forint"),
		stripe::Currency::IDR => ("Rp", "Indonesian Rupiah"),
		stripe::Currency::ILS => ("₪", "Israeli New Sheqel"),
		stripe::Currency::INR => ("₹", "Indian Rupee"),
		stripe::Currency::ISK => ("kr", "Icelandic Króna"),
		stripe::Currency::JMD => ("JM$", "Jamaican Dollar"),
		stripe::Currency::JPY => ("¥", "Japanese Yen"),
		stripe::Currency::KES => ("KSh", "Kenyan Shilling"),
		stripe::Currency::KGS => ("лв", "Kyrgyzstani Som"),
		stripe::Currency::KHR => ("៛", "Cambodian Riel"),
		stripe::Currency::KMF => ("CF", "Comorian Franc"),
		stripe::Currency::KRW => ("₩", "South Korean Won"),
		stripe::Currency::KYD => ("KY$", "Cayman Islands Dollar"),
		stripe::Currency::KZT => ("₸", "Kazakhstani Tenge"),
		stripe::Currency::LAK => ("₭", "Lao Kip"),
		stripe::Currency::LBP => ("ل.ل", "Lebanese Pound"),
		stripe::Currency::LKR => ("Rs", "Sri Lankan Rupee"),
		stripe::Currency::LRD => ("LR$", "Liberian Dollar"),
		stripe::Currency::LSL => ("L", "Lesotho Loti"),
		stripe::Currency::LTL => ("Lt", "Lithuanian Litas (pre-Euro)"),
		stripe::Currency::LVL => ("Ls", "Latvian Lats (pre-Euro)"),
		stripe::Currency::MAD => ("DH", "Moroccan Dirham"),
		stripe::Currency::MDL => ("L", "Moldovan Leu"),
		stripe::Currency::MGA => ("Ar", "Malagasy Ariary"),
		stripe::Currency::MKD => ("ден", "Macedonian Denar"),
		stripe::Currency::MNT => ("₮", "Mongolian Tögrög"),
		stripe::Currency::MOP => ("P", "Macanese Pataca"),
		stripe::Currency::MRO => ("UM", "Mauritanian Ouguiya"),
		stripe::Currency::MUR => ("₨", "Mauritian Rupee"),
		stripe::Currency::MVR => ("Rf", "Maldivian Rufiyaa"),
		stripe::Currency::MWK => ("MK", "Malawian Kwacha"),
		stripe::Currency::MXN => ("MX$", "Mexican Peso"),
		stripe::Currency::MYR => ("RM", "Malaysian Ringgit"),
		stripe::Currency::MZN => ("MT", "Mozambican Metical"),
		stripe::Currency::NAD => ("NA$", "Namibian Dollar"),
		stripe::Currency::NGN => ("₦", "Nigerian Naira"),
		stripe::Currency::NIO => ("C$", "Nicaraguan Córdoba"),
		stripe::Currency::NOK => ("kr", "Norwegian Krone"),
		stripe::Currency::NPR => ("₨", "Nepalese Rupee"),
		stripe::Currency::NZD => ("NZ$", "New Zealand Dollar"),
		stripe::Currency::PAB => ("B/.", "Panamanian Balboa"),
		stripe::Currency::PEN => ("S/", "Peruvian Nuevo Sol"),
		stripe::Currency::PGK => ("K", "Papua New Guinean Kina"),
		stripe::Currency::PHP => ("₱", "Philippine Peso"),
		stripe::Currency::PKR => ("₨", "Pakistani Rupee"),
		stripe::Currency::PLN => ("zł", "Polish Złoty"),
		stripe::Currency::PYG => ("₲", "Paraguayan Guaraní"),
		stripe::Currency::QAR => ("ر.ق", "Qatari Riyal"),
		stripe::Currency::RON => ("lei", "Romanian Leu"),
		stripe::Currency::RSD => ("дин", "Serbian Dinar"),
		stripe::Currency::RUB => ("₽", "Russian Ruble"),
		stripe::Currency::RWF => ("FRw", "Rwandan Franc"),
		stripe::Currency::SAR => ("﷼", "Saudi Riyal"),
		stripe::Currency::SBD => ("SI$", "Solomon Islands Dollar"),
		stripe::Currency::SCR => ("₨", "Seychellois Rupee"),
		stripe::Currency::SEK => ("kr", "Swedish Krona"),
		stripe::Currency::SGD => ("SG$", "Singapore Dollar"),
		stripe::Currency::SHP => ("£", "Saint Helenian Pound"),
		stripe::Currency::SLL => ("Le", "Sierra Leonean Leone"),
		stripe::Currency::SOS => ("S", "Somali Shilling"),
		stripe::Currency::SRD => ("SR$", "Surinamese Dollar"),
		stripe::Currency::STD => ("Db", "São Tomé and Príncipe Dobra"),
		stripe::Currency::SVC => ("SV$", "Salvadoran Colón"),
		stripe::Currency::SZL => ("L", "Swazi Lilangeni"),
		stripe::Currency::THB => ("฿", "Thai Baht"),
		stripe::Currency::TJS => ("ЅМ", "Tajikistani Somoni"),
		stripe::Currency::TOP => ("T$", "Tongan Paʻanga"),
		stripe::Currency::TRY => ("₺", "Turkish Lira"),
		stripe::Currency::TTD => ("TT$", "Trinidad and Tobago Dollar"),
		stripe::Currency::TWD => ("NT$", "New Taiwan Dollar"),
		stripe::Currency::TZS => ("TSh", "Tanzanian Shilling"),
		stripe::Currency::UAH => ("₴", "Ukrainian Hryvnia"),
		stripe::Currency::UGX => ("USh", "Ugandan Shilling"),
		stripe::Currency::USD => ("$", "United States Dollar"),
		stripe::Currency::UYU => ("UY$", "Uruguayan Peso"),
		stripe::Currency::UZS => ("лв", "Uzbekistani Som"),
		stripe::Currency::VEF => ("Bs.", "Venezuelan Bolívar"),
		stripe::Currency::VND => ("₫", "Vietnamese Đồng"),
		stripe::Currency::VUV => ("VT", "Vanuatu Vatu"),
		stripe::Currency::WST => ("WS$", "Samoan Tala"),
		stripe::Currency::XAF => ("FCFA", "Central African CFA Franc"),
		stripe::Currency::XCD => ("EC$", "East Caribbean Dollar"),
		stripe::Currency::XOF => ("CFA", "West African CFA Franc"),
		stripe::Currency::XPF => ("₣", "CFP Franc"),
		stripe::Currency::YER => ("﷼", "Yemeni Rial"),
		stripe::Currency::ZAR => ("R", "South African Rand"),
		stripe::Currency::ZMW => ("ZK", "Zambian Kwacha"),
	}
}

impl Plan {
	pub fn from_variant(
		value: SubscriptionProductVariant,
		regional_currency: Option<stripe::Currency>,
		default_currency: stripe::Currency,
	) -> Option<Self> {
		let (interval_unit, discount) = match value.kind {
			SubscriptionProductKind::Monthly => (SubscriptionCycleUnit::Month, None),
			SubscriptionProductKind::Yearly => (SubscriptionCycleUnit::Year, Some(0.2)),
		};

		let (currency, price) =
			if let Some(price) = regional_currency.and_then(|currency| value.currency_prices.get(&currency)) {
				(regional_currency.unwrap(), *price)
			} else if let Some(price) = value.currency_prices.get(&default_currency) {
				(default_currency, *price)
			} else {
				return None;
			};

		let (currency_symbol, currency_name) = currency_symbol(currency);

		Some(Self {
			interval_unit,
			interval: 1,
			price: price.max(0) as u64,
			currency,
			discount,
			currency_symbol,
			currency_name,
		})
	}
}
