use std::path::PathBuf;
use std::str::FromStr;

use shared::database::{
	badge::{Badge, BadgeId},
	image_set::ImageSet, user::UserId,
};

#[derive(Debug)]
pub struct BadgeReprocessJob {
	pub id: BadgeId,
	pub input: PathBuf,
}

#[derive(Debug)]
pub struct NewBadgeJob {
	pub badge: Badge,
	pub input: PathBuf,
}

pub fn new_badges() -> Vec<NewBadgeJob> {
	vec![
		NewBadgeJob {
			badge: Badge {
				id: BadgeId::from_str("01JAT9B20KQK8DK8FT13XV83YC").unwrap(),
				name: "7TV Subscriber - 3 Years".to_string(),
				description: Some("7TV Subscriber (3 Years)".to_string()),
				tags: vec!["sub36".to_string()],
				image_set: ImageSet {
					input: shared::database::image_set::ImageSetInput::Pending {
						task_id: String::new(),
						path: String::new(),
						mime: String::new(),
						size: 0,
					},
					outputs: vec![],
				},
				created_by_id: UserId::nil(),
				updated_at: chrono::Utc::now(),
				search_updated_at: None,
			},
			input: "./local/badges/sub36.avif".into(),
		},
		NewBadgeJob {
			badge: Badge {
				id: BadgeId::from_str("01JAT9BASQDPE5VV0PMXRYND9E").unwrap(),
				name: "7TV Subscriber - 3 Years & a Quarter".to_string(),
				description: Some("7TV Subscriber (3 Years & a Quarter)".to_string()),
				tags: vec!["sub39".to_string()],
				image_set: ImageSet {
					input: shared::database::image_set::ImageSetInput::Pending {
						task_id: String::new(),
						path: String::new(),
						mime: String::new(),
						size: 0,
					},
					outputs: vec![],
				},
				created_by_id: UserId::nil(),
				updated_at: chrono::Utc::now(),
				search_updated_at: None,
			},
			input: "./local/badges/sub39.avif".into(),
		},
		NewBadgeJob {
			badge: Badge {
				id: BadgeId::from_str("01JAT9BH1BBRB8Q45AJVVND54R").unwrap(),
				name: "7TV Subscriber - 3 Years & a Half".to_string(),
				description: Some("7TV Subscriber (3 Years & a Half)".to_string()),
				tags: vec!["sub42".to_string()],
				image_set: ImageSet {
					input: shared::database::image_set::ImageSetInput::Pending {
						task_id: String::new(),
						path: String::new(),
						mime: String::new(),
						size: 0,
					},
					outputs: vec![],
				},
				created_by_id: UserId::nil(),
				updated_at: chrono::Utc::now(),
				search_updated_at: None,
			},
			input: "./local/badges/sub42.avif".into(),
		},
		NewBadgeJob {
			badge: Badge {
				id: BadgeId::from_str("01JAT9BPCRSDK1BQSM17278GJN").unwrap(),
				name: "7TV Subscriber - 3 Years & Three Quarters".to_string(),
				description: Some("7TV Subscriber (3 Years & Three Quarters)".to_string()),
				tags: vec!["sub45".to_string()],
				image_set: ImageSet {
					input: shared::database::image_set::ImageSetInput::Pending {
						task_id: String::new(),
						path: String::new(),
						mime: String::new(),
						size: 0,
					},
					outputs: vec![],
				},
				created_by_id: UserId::nil(),
				updated_at: chrono::Utc::now(),
				search_updated_at: None,
			},
			input: "./local/badges/sub45.avif".into(),
		},
	]
}

pub fn jobs() -> Vec<BadgeReprocessJob> {
	vec![
		BadgeReprocessJob {
			id: BadgeId::from_str("01829e8c-7388-0000-e46e-b00e438a696a").unwrap(),
			input: "./local/badges/sub1.png".into(),
		},
		BadgeReprocessJob {
			id: BadgeId::from_str("01829e92-ff50-0000-e46e-b00e438a696b").unwrap(),
			input: "./local/badges/sub2.png".into(),
		},
		BadgeReprocessJob {
			id: BadgeId::from_str("01829e94-91a8-0000-e46e-b00e438a696c").unwrap(),
			input: "./local/badges/sub3.png".into(),
		},
		BadgeReprocessJob {
			id: BadgeId::from_str("01829e95-e968-0000-e46e-b00e438a696d").unwrap(),
			input: "./local/badges/sub6.png".into(),
		},
		BadgeReprocessJob {
			id: BadgeId::from_str("01829e97-1630-0000-e46e-b00e438a696e").unwrap(),
			input: "./local/badges/sub9.png".into(),
		},
		BadgeReprocessJob {
			id: BadgeId::from_str("01829e98-79a8-0000-e46e-b00e438a696f").unwrap(),
			input: "./local/badges/sub12.png".into(),
		},
		BadgeReprocessJob {
			id: BadgeId::from_str("0184a18e-81f0-0000-2863-630a2d06e27a").unwrap(),
			input: "./local/badges/sub15.png".into(),
		},
		BadgeReprocessJob {
			id: BadgeId::from_str("0184a191-77c0-0000-2863-630a2d06e27b").unwrap(),
			input: "./local/badges/sub18.png".into(),
		},
		BadgeReprocessJob {
			id: BadgeId::from_str("01876c56-d638-0000-cda6-36a6910a265f").unwrap(),
			input: "./local/badges/sub21.png".into(),
		},
		BadgeReprocessJob {
			id: BadgeId::from_str("018a0abc-9e00-0000-2040-c6754787d929").unwrap(),
			input: "./local/badges/sub24.avif".into(),
		},
		BadgeReprocessJob {
			id: BadgeId::from_str("018a0ad2-aa88-0000-2040-c6754787d92a").unwrap(),
			input: "./local/badges/sub27.avif".into(),
		},
		BadgeReprocessJob {
			id: BadgeId::from_str("018a0ae6-1730-0000-2040-c6754787d92f").unwrap(),
			input: "./local/badges/sub30.avif".into(),
		},
		BadgeReprocessJob {
			id: BadgeId::from_str("018a0ae7-a1b8-0000-2040-c6754787d930").unwrap(),
			input: "./local/badges/sub33.avif".into(),
		},
		// ---
		BadgeReprocessJob {
			id: BadgeId::from_str("0190eae2-1ef0-0000-58ae-9fae9addffdb").unwrap(),
			input: "./local/badges/owner.png".into(),
		},
		BadgeReprocessJob {
			id: BadgeId::from_str("01829f0a-bf90-0000-e46e-b00e438a6982").unwrap(),
			input: "./local/badges/kathy.png".into(),
		},
		BadgeReprocessJob {
			id: BadgeId::from_str("01829ea9-b3d0-0000-e46e-b00e438a6971").unwrap(),
			input: "./local/badges/admin.png".into(),
		},
		BadgeReprocessJob {
			id: BadgeId::from_str("01829f0d-76e0-0000-e46e-b00e438a6984").unwrap(),
			input: "./local/badges/translator.png".into(),
		},
		BadgeReprocessJob {
			id: BadgeId::from_str("01829eac-7ac0-0000-e46e-b00e438a6972").unwrap(),
			input: "./local/badges/mod.png".into(),
		},
		BadgeReprocessJob {
			id: BadgeId::from_str("01829f08-69e8-0000-e46e-b00e438a6981").unwrap(),
			input: "./local/badges/contributor.png".into(),
		},
		BadgeReprocessJob {
			id: BadgeId::from_str("0183c3e5-b5f8-0000-ddb1-2c4c4f707a3f").unwrap(),
			input: "./local/badges/event.png".into(),
		},
		BadgeReprocessJob {
			id: BadgeId::from_str("018c5e43-7380-0000-ffc9-d968e5102164").unwrap(),
			input: "./local/badges/nnys.png".into(),
		},
		BadgeReprocessJob {
			id: BadgeId::from_str("01854211-7330-0000-919e-3d301c52fa84").unwrap(),
			input: "./local/badges/xmas.png".into(),
		},
		BadgeReprocessJob {
			id: BadgeId::from_str("018aaa82-ea48-0000-55de-b74f50368f40").unwrap(),
			input: "./local/badges/subtember23.png".into(),
		},
		BadgeReprocessJob {
			id: BadgeId::from_str("01829ea2-1a80-0000-e46e-b00e438a6970").unwrap(),
			input: "./local/badges/founder.png".into(),
		},
		BadgeReprocessJob {
			id: BadgeId::from_str("0190eb05-db60-0000-58ae-9fae9addffe1").unwrap(),
			input: "./local/badges/troy.avif".into(),
		},
	]
}
