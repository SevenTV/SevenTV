// #[tokio::main]
// async fn main() {
// 	tracing_subscriber::fmt::init();

// 	let mut client = scuffle_image_processor_proto::image_processor_client::ImageProcessorClient::connect("http://localhost:50051").await.unwrap();

// 	let req = scuffle_image_processor_proto::ProcessImageRequest {
// 		priority: 0,
// 		task: Some(scuffle_image_processor_proto::Task {
// 			input: Some(scuffle_image_processor_proto::Input {
// 				metadata: None,
// 				path: None,
// 				..Default::default()
// 			}),
// 			output: Some(scuffle_image_processor_proto::Output {
// 				drive_path: Some(scuffle_image_processor_proto::DrivePath {
// 					drive: "seventv-cdn".to_string(),
// 					path: "{id}/{width}x{height}{static}.{ext}".to_string(),
// 					acl: None,
// 				}),
// 				formats: vec![
// 					scuffle_image_processor_proto::OutputFormatOptions {
// 						format: scuffle_image_processor_proto::OutputFormat::PngStatic as i32,
// 						quality: scuffle_image_processor_proto::OutputQuality::Auto as i32,
// 						name: None,
// 					},
// 					scuffle_image_processor_proto::OutputFormatOptions {
// 						format: scuffle_image_processor_proto::OutputFormat::WebpStatic as i32,
// 						quality: scuffle_image_processor_proto::OutputQuality::Auto as i32,	
// 						name: None,
// 					},
// 					scuffle_image_processor_proto::OutputFormatOptions {
// 						format: scuffle_image_processor_proto::OutputFormat::AvifStatic as i32,
// 						quality: scuffle_image_processor_proto::OutputQuality::Auto as i32,
// 						name: None,
// 					},
// 					scuffle_image_processor_proto::OutputFormatOptions {
// 						format: scuffle_image_processor_proto::OutputFormat::WebpAnim as i32,
// 						quality: scuffle_image_processor_proto::OutputQuality::Auto as i32,
// 						name: None,
// 					},
// 					scuffle_image_processor_proto::OutputFormatOptions {
// 						format: scuffle_image_processor_proto::OutputFormat::GifAnim as i32,
// 						quality: scuffle_image_processor_proto::OutputQuality::Auto as i32,	
// 						name: None,
// 					},
// 					scuffle_image_processor_proto::OutputFormatOptions {
// 						format: scuffle_image_processor_proto::OutputFormat::AvifAnim as i32,
// 						quality: scuffle_image_processor_proto::OutputQuality::Auto as i32,
// 						name: None,
// 					},
// 				],
// 				upscale: true,
// 				skip_impossible_formats: true,
// 				resize: Some(scuffle_image_processor_proto::output::Resize::Scaling(scuffle_image_processor_proto::Scaling{
// 					base: Some(scuffle_image_processor_proto::scaling::Base::BaseHeight(32)),
// 					scales: vec![1, 2, 3, 4],
// 				})),
// 				..Default::default()
// 			}),
// 			events: Some(scuffle_image_processor_proto::Events {
// 				on_success: Some(scuffle_image_processor_proto::EventQueue {
// 					name: "nats-json".to_string(),
// 					topic: "temp.success".to_string(),
// 				}),
// 				on_start: Some(scuffle_image_processor_proto::EventQueue {
// 					name: "nats-json".to_string(),
// 					topic: "temp.start".to_string(),
// 				}),
// 				on_failure: Some(scuffle_image_processor_proto::EventQueue {
// 					name: "nats-json".to_string(),
// 					topic: "temp.failure".to_string(),
// 				}),
// 				on_cancel: Some(scuffle_image_processor_proto::EventQueue {
// 					name: "nats-json".to_string(),
// 					topic: "temp.cancel".to_string(),
// 				}),
// 				metadata: Default::default(),
// 			}),
// 			..Default::default()
// 		}),
// 		input_upload: Some(scuffle_image_processor_proto::InputUpload {
// 			binary: std::fs::read("local/image/MothSanityCheck.avif").unwrap(),
// 			drive_path: Some(scuffle_image_processor_proto::DrivePath {
// 				drive: "seventv-cdn-private".to_string(),
// 				path: "{id}".to_string(),
// 				acl: None,
// 			}),
// 			..Default::default()
// 		}),
// 		..Default::default()
// 	};

// 	let res = client.process_image(req).await.unwrap();
// 	println!("{:?}", res);
// }

use std::net::SocketAddr;

fn ip_mode(addr: SocketAddr) -> std::io::Result<socket2::Domain> {
	if addr.ip().is_ipv4() {
		Ok(socket2::Domain::IPV4)
	} else if addr.ip().is_ipv6() {
		Ok(socket2::Domain::IPV6)
	} else {
		Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid ip address"))
	}
}

fn make_tcp_listener(addr: SocketAddr) -> std::io::Result<tokio::net::TcpListener> {
	let socket = socket2::Socket::new(ip_mode(addr)?, socket2::Type::STREAM, Some(socket2::Protocol::TCP))?;

	socket.set_nonblocking(true)?;
	socket.set_reuse_address(true)?;
	socket.set_reuse_port(true)?;
	socket.bind(&socket2::SockAddr::from(addr))?;
	socket.listen(1024)?;

	tokio::net::TcpListener::from_std(socket.into())
}

#[tokio::main]
async fn main() {
	make_tcp_listener("[::]:8080".parse().unwrap()).unwrap();
}
