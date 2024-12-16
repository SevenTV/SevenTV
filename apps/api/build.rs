use std::process::Command;

fn main() {
	// Include the current commit hash
	if let Some(hash) = Command::new("git")
		.args(["rev-parse", "HEAD"])
		.output()
		.ok()
		.and_then(|o| String::from_utf8(o.stdout).ok())
	{
		println!("cargo:rustc-env=GIT_HASH={}", hash);
	}
}
