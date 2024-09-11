use std::collections::HashMap;

use anyhow::Context;
use bytes::Bytes;
use image_processor::{OutputFormat, OutputFormatOptions, OutputQuality};
use scuffle_image_processor_proto::image_processor_client::ImageProcessorClient;
use scuffle_image_processor_proto::{self as image_processor};

use crate::config::ImageProcessorConfig;
use crate::database::badge::BadgeId;
use crate::database::emote::EmoteId;
use crate::database::paint::{PaintId, PaintLayerId};
use crate::database::user::profile_picture::UserProfilePictureId;
use crate::database::user::UserId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Subject {
	Emote(EmoteId),
	ProfilePicture(UserProfilePictureId),
	PaintLayer(PaintId, PaintLayerId),
	Badge(BadgeId),
}

impl Subject {
	pub fn wildcard(prefix: &str) -> String {
		format!("{prefix}.>")
	}

	pub fn to_string(&self, prefix: &str) -> String {
		let mut parts: Vec<String> = Vec::new();

		if !prefix.is_empty() {
			parts.push(prefix.to_string());
		}

		match self {
			Self::Emote(id) => {
				parts.push("emote".to_string());
				parts.push(id.to_string());
			}
			Self::ProfilePicture(id) => {
				parts.push("profile-picture".to_string());
				parts.push(id.to_string());
			}
			Self::PaintLayer(id, layer_id) => {
				parts.push("paint-layer".to_string());
				parts.push(id.to_string());
				parts.push(layer_id.to_string());
			}
			Self::Badge(id) => {
				parts.push("badge".to_string());
				parts.push(id.to_string());
			}
		}

		parts.join(".")
	}

	pub fn from_string(s: &str, prefix: &str) -> anyhow::Result<Self> {
		let s = s.strip_prefix(prefix).context("missing prefix")?.trim_start_matches('.');

		let parts = s.split('.').collect::<Vec<_>>();

		match parts.as_slice() {
			["emote", id] => Ok(Self::Emote(id.parse()?)),
			["profile-picture", id] => Ok(Self::ProfilePicture(id.parse()?)),
			["paint-layer", id, layer_id] => Ok(Self::PaintLayer(id.parse()?, layer_id.parse()?)),
			["badge", id] => Ok(Self::Badge(id.parse()?)),
			_ => anyhow::bail!("invalid subject"),
		}
	}
}

pub struct ImageProcessor {
	client: ImageProcessorClient<tonic::transport::Channel>,
	input_drive_name: String,
	output_drive_name: String,
	event_queue_name: String,
	event_queue_topic_prefix: String,
}

impl ImageProcessor {
	pub async fn new(config: &ImageProcessorConfig) -> anyhow::Result<Self> {
		let channel =
			crate::grpc::make_channel(config.address.clone(), config.resolve_interval, None).context("make channel")?;

		let client = ImageProcessorClient::new(channel);

		Ok(Self {
			client,
			input_drive_name: config.input_drive_name.clone(),
			output_drive_name: config.output_drive_name.clone(),
			event_queue_name: config.event_queue_name.clone(),
			event_queue_topic_prefix: config.event_queue_topic_prefix.clone(),
		})
	}

	pub async fn send_req(
		&self,
		req: image_processor::ProcessImageRequest,
	) -> tonic::Result<image_processor::ProcessImageResponse> {
		Ok(self.client.clone().process_image(req).await?.into_inner())
	}

	pub fn make_request(
		&self,
		input_upload: Option<image_processor::InputUpload>,
		task: image_processor::Task,
	) -> image_processor::ProcessImageRequest {
		image_processor::ProcessImageRequest {
			input_upload,
			task: Some(task),
			priority: 5,
			..Default::default()
		}
	}

	pub fn make_task(&self, output: image_processor::Output, events: image_processor::Events) -> image_processor::Task {
		image_processor::Task {
			output: Some(output),
			events: Some(events),
			limits: Some(image_processor::Limits {
				max_input_frame_count: Some(1000),
				max_input_width: Some(1000),
				max_input_height: Some(1000),
				..Default::default()
			}),
			..Default::default()
		}
	}

	pub fn make_input_upload(&self, input_path: String, data: Bytes) -> image_processor::InputUpload {
		image_processor::InputUpload {
			drive_path: Some(image_processor::DrivePath {
				drive: self.input_drive_name.clone(),
				path: input_path,
				acl: Some("private".to_string()),
			}),
			binary: data.to_vec(),
			..Default::default()
		}
	}

	pub fn make_output(&self, output_path: String) -> image_processor::Output {
		image_processor::Output {
			drive_path: Some(image_processor::DrivePath {
				drive: self.output_drive_name.clone(),
				path: output_path,
				acl: Some("public-read".to_string()),
			}),
			input_reupload_path: None,
			formats: vec![
				OutputFormatOptions {
					format: OutputFormat::WebpAnim as i32,
					quality: OutputQuality::Auto as i32,
					name: None,
				},
				OutputFormatOptions {
					format: OutputFormat::WebpStatic as i32,
					quality: OutputQuality::Auto as i32,
					name: None,
				},
				OutputFormatOptions {
					format: OutputFormat::AvifAnim as i32,
					quality: OutputQuality::Auto as i32,
					name: None,
				},
				OutputFormatOptions {
					format: OutputFormat::AvifStatic as i32,
					quality: OutputQuality::Auto as i32,
					name: None,
				},
				OutputFormatOptions {
					format: OutputFormat::GifAnim as i32,
					quality: OutputQuality::Auto as i32,
					name: None,
				},
				OutputFormatOptions {
					format: OutputFormat::PngStatic as i32,
					quality: OutputQuality::Auto as i32,
					name: None,
				},
			],
			upscale: true,
			skip_impossible_formats: true,
			min_aspect_ratio: None,
			max_aspect_ratio: Some(3.0),
			resize_method: image_processor::ResizeMethod::Fit as i32,
			resize_algorithm: image_processor::ResizeAlgorithm::Lanczos3 as i32,
			resize: Some(image_processor::output::Resize::Scaling(image_processor::Scaling {
				base: Some(image_processor::scaling::Base::BaseHeight(32)),
				scales: vec![1, 2, 3, 4],
			})),
			..Default::default()
		}
	}

	pub fn make_events(&self, topic: Subject, metadata: HashMap<String, String>) -> image_processor::Events {
		let topic = topic.to_string(&self.event_queue_topic_prefix);

		image_processor::Events {
			on_success: Some(image_processor::EventQueue {
				name: self.event_queue_name.clone(),
				topic: topic.clone(),
			}),
			on_start: Some(image_processor::EventQueue {
				name: self.event_queue_name.clone(),
				topic: topic.clone(),
			}),
			on_failure: Some(image_processor::EventQueue {
				name: self.event_queue_name.clone(),
				topic: topic.clone(),
			}),
			on_cancel: Some(image_processor::EventQueue {
				name: self.event_queue_name.clone(),
				topic: topic.clone(),
			}),
			metadata,
		}
	}

	pub async fn upload_emote(
		&self,
		id: EmoteId,
		data: Bytes,
		upload_ip: Option<std::net::IpAddr>,
	) -> tonic::Result<image_processor::ProcessImageResponse> {
		let req = self.make_request(
			Some(self.make_input_upload(format!("/emote/{id}/input.{{ext}}"), data)),
			self.make_task(
				self.make_output(format!("/emote/{id}/{{scale}}x{{static}}.{{ext}}")),
				self.make_events(Subject::Emote(id), {
					let mut map = std::collections::HashMap::new();
					map.insert("emote_id".to_string(), id.to_string());
					if let Some(ip) = upload_ip {
						map.insert("upload_ip".to_string(), ip.to_string());
					}

					map
				}),
			),
		);

		self.send_req(req).await
	}

	pub async fn upload_profile_picture(
		&self,
		id: UserProfilePictureId,
		user_id: UserId,
		data: Bytes,
		upload_ip: Option<std::net::IpAddr>,
	) -> tonic::Result<image_processor::ProcessImageResponse> {
		let req = self.make_request(
			Some(self.make_input_upload(format!("/user/{user_id}/profile-picture/{id}/input.{{ext}}"), data)),
			self.make_task(
				self.make_output(format!("/user/{user_id}/profile-picture/{id}/{{scale}}x{{static}}.{{ext}}")),
				self.make_events(Subject::ProfilePicture(id), {
					let mut map = std::collections::HashMap::new();
					map.insert("user_id".to_string(), user_id.to_string());
					if let Some(ip) = upload_ip {
						map.insert("upload_ip".to_string(), ip.to_string());
					}

					map
				}),
			),
		);

		self.send_req(req).await
	}

	pub async fn upload_paint_layer(
		&self,
		id: PaintId,
		layer_id: PaintLayerId,
		data: Bytes,
	) -> tonic::Result<image_processor::ProcessImageResponse> {
		let req = self.make_request(
			Some(self.make_input_upload(format!("/paint/{id}/layer/{layer_id}/input.{{ext}}"), data)),
			image_processor::Task {
				limits: Some(image_processor::Limits {
					max_input_frame_count: Some(1000),
					max_input_width: Some(1500),
					max_input_height: Some(1500),
					..Default::default()
				}),
				..self.make_task(
					scuffle_image_processor_proto::Output {
						max_aspect_ratio: None,
						..self.make_output(format!("/paint/{id}/layer/{layer_id}/{{scale}}x{{static}}.{{ext}}"))
					},
					self.make_events(
						Subject::PaintLayer(id, layer_id),
						[
							("paint_id".to_string(), id.to_string()),
							("layer_id".to_string(), layer_id.to_string()),
						]
						.into_iter()
						.collect(),
					),
				)
			},
		);

		self.send_req(req).await
	}

	pub async fn upload_badge(&self, id: BadgeId, data: Bytes) -> tonic::Result<image_processor::ProcessImageResponse> {
		let req = self.make_request(
			Some(self.make_input_upload(format!("/badge/{id}/input.{{ext}}"), data)),
			self.make_task(
				self.make_output(format!("/badge/{id}/{{scale}}x{{static}}.{{ext}}")),
				self.make_events(
					Subject::Badge(id),
					[("badge_id".to_string(), id.to_string())].into_iter().collect(),
				),
			),
		);

		self.send_req(req).await
	}
}
