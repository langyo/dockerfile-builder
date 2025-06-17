use dockerfile_builder::instruction_builder::{EnvBuilder, FromBuilder, LabelBuilder};
use dockerfile_builder::Dockerfile;

use anyhow::{anyhow, Result};
use bollard::query_parameters::BuildImageOptions;
use bollard::{body_full, Docker};
use flate2::write::GzEncoder;
use flate2::Compression;
use futures::StreamExt;
use std::io::Write;
use std::time::Duration;
use tar::Builder;

async fn docker_build(docker_tar: Vec<u8>) -> Result<()> {
    let mut docker = Docker::connect_with_defaults()?;
    docker.set_timeout(Duration::from_secs(600));

    let build_image_options = BuildImageOptions {
        dockerfile: "Dockerfile".to_string(),
        t: Some("test-image".to_string()),
        //version: BuilderVersion::BuilderBuildKit,
        //rm: true,
        //q: false,
        ..Default::default()
    };

    let mut build_results = docker.build_image(
        build_image_options,
        None,
        Some(body_full(docker_tar.into())),
    );
    while let Some(msg) = build_results.next().await {
        if msg.is_err() {
            eprintln!("{:#?}", &msg);
            return Err(anyhow!("{:#?}", &msg));
        }
    }

    Ok(())
}

fn create_docker_tar(docker_file: String) -> Result<Vec<u8>> {
    let mut header = tar::Header::new_gnu();
    header.set_path("Dockerfile")?;
    header.set_size(docker_file.clone().len() as u64);
    header.set_mode(0o755);
    header.set_cksum();
    let mut tar = Builder::new(Vec::new());
    tar.append(&header, docker_file.as_bytes())?;
    let uncompressed = tar.into_inner()?;
    let mut c = GzEncoder::new(Vec::new(), Compression::default());
    c.write_all(&uncompressed)?;
    let compressed = c.finish()?;
    Ok(compressed)
}

#[tokio::test]
async fn test_docker_image_build() {
    let mut dockerfile = Dockerfile::new();
    let escape_check = EnvBuilder::builder()
        .key("PHP_ERROR_REPORTING")
        .value("E_ERROR | E_WARNING | E_PARSE")
        .build()
        .unwrap();
    let from = FromBuilder::builder().image("scratch").build().unwrap();
    dockerfile.push(from).push(escape_check);
    let tar = create_docker_tar(dockerfile.to_string()).unwrap();
    assert!(docker_build(tar).await.is_ok());
}

#[tokio::test]
async fn test_docker_image_fail() {
    let mut dockerfile = Dockerfile::new();
    let escape_check = LabelBuilder::builder()
        .key("JKHASD||2&!")
        .value("E_ERROR | E_WARNING | E_PARSE")
        .build()
        .unwrap();
    let from = FromBuilder::builder().image("scratch").build().unwrap();
    dockerfile.push(from).push(escape_check);
    let tar = create_docker_tar(dockerfile.to_string()).unwrap();
    assert!(docker_build(tar).await.is_err());
}
