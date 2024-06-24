use anyhow::Error;

fn main() -> Result<(), Error> {
    #[cfg(not(feature = "rari"))]
    {
        let package_path = std::path::Path::new("@webref/css");
        rari_deps::webref_css::update_webref_css(package_path)?;
    }
    println!("cargo::rerun-if-changed=build.rs");
    Ok(())
}
