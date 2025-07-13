use std::error::Error;

fn main() {
	build_detours().unwrap();
}

const CPP_FILES: [&str; 5] = [
	"./ext/detours/src/creatwth.cpp",
	"./ext/detours/src/detours.cpp",
	"./ext/detours/src/disasm.cpp",
	"./ext/detours/src/image.cpp",
	"./ext/detours/src/modules.cpp",
];

fn build_detours() -> Result<(), Box<dyn Error>> {
	add_target_options(
		cc::Build::new()
			.include("./ext/detours/src")
			.files(&CPP_FILES),
	)
	.try_compile("detours")?;
	Ok(())
}
fn add_target_options(build: &mut cc::Build) -> &mut cc::Build {
	if std::env::var("CARGO_CFG_TARGET_ENV").unwrap() != "msvc" {
		build
			.compiler("clang")
			.cpp(true)
			.flag("-fms-extensions")
			.flag("-Wno-everything")
	} else {
		build
	}
}
