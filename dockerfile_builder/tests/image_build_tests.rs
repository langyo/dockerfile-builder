use dockerfile_builder::instruction::ENV;
use dockerfile_builder::instruction_builder::{
    EnvBuilder, ExposeBuilder, FromBuilder, UserBuilder, WorkdirBuilder,
};
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

fn connect_to_docker() -> Result<Docker> {
    let docker = Docker::connect_with_defaults()?;
    Ok(docker)
}

fn scratch_docker_file() -> Result<Dockerfile> {
    let mut df = Dockerfile::new();
    let from = FromBuilder::builder().image("scratch").build()?;
    df.push(from);
    Ok(df)
}

async fn docker_build(docker_tar: Vec<u8>) -> Result<()> {
    let mut docker = connect_to_docker()?;
    docker.set_timeout(Duration::from_secs(600));

    let build_image_options = BuildImageOptions {
        dockerfile: "Dockerfile".to_string(),
        t: Some("test-image".to_string()),
        rm: true,
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
async fn test_docker_connect() {
    let docker = connect_to_docker();
    assert!(docker.is_ok());
}

#[tokio::test]
async fn env_instr_build() -> Result<()> {
    let mut docker_one = scratch_docker_file()?;
    let env_builder = EnvBuilder::builder()
        .key("PHP_ERROR_REPORTING")
        .value("E_ERROR | E_WARNING | E_PARSE")
        .build()?;
    docker_one.push(env_builder);
    let tar = create_docker_tar(docker_one.to_string())?;
    assert!(docker_build(tar).await.is_ok());

    //Manual escape b/c using Env::from should be Ok
    let mut docker_two = scratch_docker_file()?;
    let env_str = ENV::from("PHP_ERROR_REPORTING=\"E_ERROR | E_WARNING | E_PARSE\"").to_string();
    docker_two.push(env_str);
    let tar = create_docker_tar(docker_two.to_string()).unwrap();
    assert!(docker_build(tar).await.is_ok());

    //No manual escape should be Err
    let mut docker_two = scratch_docker_file()?;
    let env_str = ENV::from("PHP_ERROR_REPORTING=E_ERROR | E_WARNING | E_PARSE").to_string();
    docker_two.push(env_str);
    let tar = create_docker_tar(docker_two.to_string()).unwrap();
    assert!(docker_build(tar).await.is_err());
    Ok(())
}

#[tokio::test]
async fn healthcheck_submillisecond_duration_fails_build() -> Result<()> {
    let mut docker = scratch_docker_file()?;
    docker.push("HEALTHCHECK --interval=0.0000002s CMD curl -f http://localhost/");
    let tar = create_docker_tar(docker.to_string())?;
    assert!(docker_build(tar).await.is_err());
    Ok(())
}

#[tokio::test]
async fn test_multiple_instr() -> Result<()> {
    let mut docker_file = Dockerfile::new();
    let from = FromBuilder::builder().image("scratch").build()?;
    let workdir = WorkdirBuilder::builder().path("/usr/local/app").build()?;
    let expose = ExposeBuilder::builder().port(5000).build()?;
    let user = UserBuilder::builder().user("app").build()?;
    docker_file.push(from).push(workdir).push(expose).push(user);
    let tar = create_docker_tar(docker_file.to_string())?;
    assert!(docker_build(tar).await.is_ok());

    let expected = r#"FROM scratch
WORKDIR /usr/local/app
EXPOSE 5000
USER app"#;
    assert_eq!(expected, docker_file.to_string());
    Ok(())
}
