use std::{path::PathBuf, str::FromStr};

use shared::database::badge::BadgeId;

struct Badge {
	id: BadgeId,
	input: PathBuf,
}

pub fn badges() -> Vec<Badge> {
	vec![
		Badge {
			id: BadgeId::from_str("01876c56-d638-0000-cda6-36a6910a265f").unwrap(),
			input: "./badges/sub21.avif".into(),
		},
		Badge {
			id: BadgeId::from_str("01829e94-91a8-0000-e46e-b00e438a696c").unwrap(),
			input: "./badges/sub3.avif".into(),
		},
		Badge {
			id: BadgeId::from_str("0190eae2-1ef0-0000-58ae-9fae9addffdb").unwrap(),
			input: "./badges/owner.avif".into(),
		},
		Badge {
			id: BadgeId::from_str("01829ea9-b3d0-0000-e46e-b00e438a6971").unwrap(),
			input: "./badges/admin.avif".into(),
		},
		Badge {
			id: BadgeId::from_str("0184a191-77c0-0000-2863-630a2d06e27b").unwrap(),
			input: "./badges/sub18.avif".into(),
		},
		Badge {
			id: BadgeId::from_str("01829f0d-76e0-0000-e46e-b00e438a6984").unwrap(),
			input: "./badges/translator.avif".into(),
		},
		Badge {
			id: BadgeId::from_str("01829eac-7ac0-0000-e46e-b00e438a6972").unwrap(),
			input: "./badges/mod.avif".into(),
		},
		Badge {
			id: BadgeId::from_str("01829f08-69e8-0000-e46e-b00e438a6981").unwrap(),
			input: "./badges/contributor.avif".into(),
		},
		Badge {
			id: BadgeId::from_str("0184a18e-81f0-0000-2863-630a2d06e27a").unwrap(),
			input: "./badges/sub15.avif".into(),
		},
		Badge {
			id: BadgeId::from_str("01829e95-e968-0000-e46e-b00e438a696d").unwrap(),
			input: "./badges/sub6.avif".into(),
		},
		Badge {
			id: BadgeId::from_str("0183c3e5-b5f8-0000-ddb1-2c4c4f707a3f").unwrap(),
			input: "./badges/events.avif".into(),
		},
		Badge {
			id: BadgeId::from_str("01829e92-ff50-0000-e46e-b00e438a696b").unwrap(),
			input: "./badges/sub2.avif".into(),
		},
		Badge {
			id: BadgeId::from_str("01829e97-1630-0000-e46e-b00e438a696e").unwrap(),
			input: "./badges/sub9.avif".into(),
		},
		Badge {
			id: BadgeId::from_str("018a0ad2-aa88-0000-2040-c6754787d92a").unwrap(),
			input: "./badges/sub27.avif".into(),
		},
		Badge {
			id: BadgeId::from_str("01829e98-79a8-0000-e46e-b00e438a696f").unwrap(),
			input: "./badges/sub12.avif".into(),
		},
		Badge {
			id: BadgeId::from_str("01829e8c-7388-0000-e46e-b00e438a696a").unwrap(),
			input: "./badges/sub1.avif".into(),
		},
		Badge {
			id: BadgeId::from_str("018a0ae6-1730-0000-2040-c6754787d92f").unwrap(),
			input: "./badges/sub30.avif".into(),
		},
		Badge {
			id: BadgeId::from_str("018a0ae7-a1b8-0000-2040-c6754787d930").unwrap(),
			input: "./badges/sub33.avif".into(),
		},
		Badge {
			id: BadgeId::from_str("018c5e43-7380-0000-ffc9-d968e5102164").unwrap(),
			input: "./badges/nnys.avif".into(),
		},
		Badge {
			id: BadgeId::from_str("018a0abc-9e00-0000-2040-c6754787d929").unwrap(),
			input: "./badges/sub24.avif".into(),
		},
		Badge {
			id: BadgeId::from_str("01854211-7330-0000-919e-3d301c52fa84").unwrap(),
			input: "./badges/xmas.avif".into(),
		},
		Badge {
			id: BadgeId::from_str("018aaa82-ea48-0000-55de-b74f50368f40").unwrap(),
			input: "./badges/subtember23.avif".into(),
		},
		Badge {
			id: BadgeId::from_str("01829ea2-1a80-0000-e46e-b00e438a6970").unwrap(),
			input: "./badges/founder.avif".into(),
		},
		Badge {
			id: BadgeId::from_str("0190eb05-db60-0000-58ae-9fae9addffe1").unwrap(),
			input: "./badges/troy.avif".into(),
		},
	]
}
