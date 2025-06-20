use dockerfile_builder::instruction::{Instruction, ADD, CMD, FROM, ONBUILD};
use dockerfile_builder::instruction_builder::{
    AddBuilder, AddGitBuilder, AddHttpBuilder, ArgBuilder, CmdBuilder, CmdExecBuilder, CopyBuilder,
    EntrypointBuilder, EntrypointExecBuilder, EnvBuilder, ExposeBuilder, FromBuilder,
    HealthcheckBuilder, OnbuildBuilder, RunExecBuilder, StopsignalBuilder, UserBuilder,
    VolumeBuilder, WorkdirBuilder,
};
use expect_test::expect;
use std::time::Duration;

#[test]
fn from() {
    let from = FromBuilder::builder().image("cargo-chef").build().unwrap();
    let expected = expect!["FROM cargo-chef"];
    expected.assert_eq(&from.to_string());

    let from = FromBuilder::builder()
        .image("cargo-chef")
        .platform("linux/arm64")
        .build()
        .unwrap();
    let expected = expect!["FROM --platform=linux/arm64 cargo-chef"];
    expected.assert_eq(&from.to_string());

    let from = FromBuilder::builder()
        .image("cargo-chef")
        .name("chef")
        .tag("latest")
        .build()
        .unwrap();
    let expected = expect!["FROM cargo-chef:latest AS chef"];
    expected.assert_eq(&from.to_string());

    let from = FromBuilder::builder()
        .image("cargo-chef")
        .name("chef")
        .digest("sha256")
        .build()
        .unwrap();
    let expected = expect!["FROM cargo-chef@sha256 AS chef"];
    expected.assert_eq(&from.to_string());
}

#[test]
fn run() {
    let run = RunExecBuilder::builder()
        .executable("mybin.exe")
        .build()
        .unwrap();
    let expected = expect![r#"RUN ["mybin.exe"]"#];
    expected.assert_eq(&run.to_string());
}

#[test]
fn cmd() {
    let cmd = CmdBuilder::builder()
        .command("some command")
        .build()
        .unwrap();
    let expected = expect!["CMD some command"];
    expected.assert_eq(&cmd.to_string());

    let cmd = CmdExecBuilder::builder()
        .executable("some command")
        .build()
        .unwrap();
    let expected = expect![r#"CMD ["some command"]"#];
    expected.assert_eq(&cmd.to_string());
}

#[test]
fn from_err() {
    let from = FromBuilder::builder()
        .image("cargo-chef")
        .tag("t")
        .digest("d")
        .build();
    match from {
        Ok(_) => panic!("Both tag and digest are set. Expect test to fail"),
        Err(e) => assert_eq!(
            e.to_string(),
            "Dockerfile image can only have tag OR digest".to_string(),
        ),
    }
}

#[test]
fn expose() {
    let expose = ExposeBuilder::builder().port(80).build().unwrap();
    let expected = expect!["EXPOSE 80"];
    expected.assert_eq(&expose.to_string());
}

#[test]
fn add() {
    let add = AddBuilder::builder()
        .src("file_1")
        .dest("/mydir/")
        .build()
        .unwrap();
    let expected = expect!["ADD file_1 /mydir/"];
    expected.assert_eq(&add.to_string());
}

#[test]
fn add_http() {
    let add = AddHttpBuilder::builder()
        .checksum("sha256::123")
        .src("http://example.com/foobar")
        .dest("/")
        .build()
        .unwrap();
    let expected = expect!["ADD --checksum=sha256::123 http://example.com/foobar /"];
    expected.assert_eq(&add.to_string());
}

#[test]
fn add_git() {
    let add = AddGitBuilder::builder()
        .keep_git_dir(true)
        .git_ref("https://github.com/moby/buildkit.git#v0.10.1")
        .dir("/buildkit")
        .build()
        .unwrap();
    let expected =
        expect!["ADD --keep-git-dir=true https://github.com/moby/buildkit.git#v0.10.1 /buildkit"];
    expected.assert_eq(&add.to_string());
}

#[test]
fn copy() {
    let copy = CopyBuilder::builder()
        .link(true)
        .src("foo/")
        .dest("bar/")
        .build()
        .unwrap();
    let expected = expect!["COPY --link foo/ bar/"];
    expected.assert_eq(&copy.to_string());
}

#[test]
fn entrypoint() {
    let entrypoint = EntrypointBuilder::builder()
        .command("some command")
        .build()
        .unwrap();
    let expected = expect![r#"ENTRYPOINT some command"#];
    expected.assert_eq(&entrypoint.to_string());

    let entrypoint = EntrypointExecBuilder::builder()
        .executable("mybin.exe")
        .build()
        .unwrap();
    let expected = expect![r#"ENTRYPOINT ["mybin.exe"]"#];
    expected.assert_eq(&entrypoint.to_string());

    let entrypoint = EntrypointExecBuilder::builder()
        .executable("top")
        .param("-b")
        .build()
        .unwrap();
    let expected = expect![r#"ENTRYPOINT ["top", "-b"]"#];
    expected.assert_eq(&entrypoint.to_string());
}

#[test]
fn volume() {
    let volume = VolumeBuilder::builder().path("/myvol1").build().unwrap();
    let expected = expect!["VOLUME /myvol1"];
    expected.assert_eq(&volume.to_string());
}

#[test]
fn user() {
    let user = UserBuilder::builder().user("myuser").build().unwrap();
    let expected = expect!["USER myuser"];
    expected.assert_eq(&user.to_string());

    let user = UserBuilder::builder()
        .user("myuser")
        .group("mygroup")
        .build()
        .unwrap();
    let expected = expect!["USER myuser:mygroup"];
    expected.assert_eq(&user.to_string());
}

#[test]
fn workdir() {
    let workdir = WorkdirBuilder::builder()
        .path("/path/to/workdir")
        .build()
        .unwrap();
    let expected = expect!["WORKDIR /path/to/workdir"];
    expected.assert_eq(&workdir.to_string());
}

#[test]
fn arg() {
    let arg = ArgBuilder::builder().name("user1").build().unwrap();
    let expected = expect!["ARG user1"];
    expected.assert_eq(&arg.to_string());

    let arg = ArgBuilder::builder()
        .name("user1")
        .value("someuser")
        .build()
        .unwrap();
    let expected = expect!["ARG user1=\"someuser\""];
    expected.assert_eq(&arg.to_string());
}

#[test]
fn onbuild() {
    let onbuild = OnbuildBuilder::builder()
        .instruction(Instruction::ADD(ADD::from(". /app/src")))
        .build()
        .unwrap();
    let expected = expect!["ONBUILD ADD . /app/src"];
    expected.assert_eq(&onbuild.to_string());
}

#[test]
fn onbuild_err() {
    let onbuild = OnbuildBuilder::builder()
        .instruction(Instruction::ONBUILD(ONBUILD::from("RUN somecommand")))
        .build();
    match onbuild {
        Ok(_) => panic!("Chaining Onbuild instructions. Expect test to fail"),
        Err(e) => assert_eq!(
            e.to_string(),
            "Chaining ONBUILD instructions using ONBUILD ONBUILD isn’t allowed".to_string(),
        ),
    }

    let onbuild = OnbuildBuilder::builder()
        .instruction(Instruction::FROM(FROM::from("someimage")))
        .build();
    match onbuild {
        Ok(_) => {
            panic!("ONBUILD instruction may not trigger FROM instruction. Expect test to fail")
        }
        Err(e) => assert_eq!(
            e.to_string(),
            "ONBUILD instruction may not trigger FROM instruction".to_string(),
        ),
    }
}

#[test]
fn stopsignal() {
    let stopsignal = StopsignalBuilder::builder()
        .signal("SIGKILL")
        .build()
        .unwrap();
    let expected = expect!["STOPSIGNAL SIGKILL"];
    expected.assert_eq(&stopsignal.to_string());
}

#[test]
fn healthcheck() {
    let healthcheck = HealthcheckBuilder::builder()
        .cmd(CMD::from("curl -f http://localhost/"))
        .build()
        .unwrap();
    let expected = expect!["HEALTHCHECK CMD curl -f http://localhost/"];
    expected.assert_eq(&healthcheck.to_string());

    let healthcheck = HealthcheckBuilder::builder()
        .cmd(CMD::from("curl -f http://localhost/"))
        .interval(Duration::from_secs(15))
        .timeout(Duration::from_secs(200))
        .start_period(Duration::from_secs(5))
        .retries(5)
        .build()
        .unwrap();
    let expected = expect!["HEALTHCHECK --interal=15 --timeout=200 --start-period=5 --retries=5 CMD curl -f http://localhost/"];
    expected.assert_eq(&healthcheck.to_string());
}

#[test]
fn env_builder_escape() {
    let escape_check = EnvBuilder::builder()
        .key("PHP_ERROR_REPORTING")
        .value("E_ERROR | E_WARNING | E_PARSE")
        .build()
        .unwrap();

    let expected = expect!["ENV PHP_ERROR_REPORTING=\"E_ERROR | E_WARNING | E_PARSE\""];
    expected.assert_eq(&escape_check.to_string());
}

#[test]
fn add_mult_src() {
    let add = AddBuilder::builder()
        .src("file_1")
        .src("file_2")
        .dest("/mydir/")
        .build()
        .unwrap();
    let expected = expect!["ADD file_1 file_2 /mydir/"];
    expected.assert_eq(&add.to_string());

    let add = AddBuilder::builder()
        .sources(vec!["file_1", "file_2"])
        .dest("/mydir/")
        .build()
        .unwrap();
    let expected = expect!["ADD file_1 file_2 /mydir/"];
    expected.assert_eq(&add.to_string());
}

#[test]
fn copy_mult_src() {
    let add = CopyBuilder::builder()
        .src("file_1")
        .src("file_2")
        .dest("/mydir/")
        .build()
        .unwrap();
    let expected = expect!["COPY file_1 file_2 /mydir/"];
    expected.assert_eq(&add.to_string());

    let add = CopyBuilder::builder()
        .sources(vec!["file_1", "file_2"])
        .dest("/mydir/")
        .build()
        .unwrap();
    let expected = expect!["COPY file_1 file_2 /mydir/"];
    expected.assert_eq(&add.to_string());
}

#[test]
fn volume_mult_src() {
    let volume = VolumeBuilder::builder()
        .path("/myvol1")
        .path("/myvol2")
        .build()
        .unwrap();
    let expected = expect!["VOLUME /myvol1 /myvol2"];
    expected.assert_eq(&volume.to_string());

    let volume = VolumeBuilder::builder()
        .paths(vec!["/myvol1", "/myvol2"])
        .build()
        .unwrap();
    let expected = expect!["VOLUME /myvol1 /myvol2"];
    expected.assert_eq(&volume.to_string());
}

#[test]
fn healthcheck_none() {
    let healthcheck = HealthcheckBuilder::builder().build().unwrap();
    let expected = expect!["HEALTHCHECK NONE"];
    expected.assert_eq(&healthcheck.to_string());

    let healthcheck = HealthcheckBuilder::set_none().unwrap();
    let expected = expect!["HEALTHCHECK NONE"];
    expected.assert_eq(&healthcheck.to_string());
}

#[test]
fn healthcheck_start_interval() {
    let healthcheck = HealthcheckBuilder::builder()
        .cmd(CMD::from("curl -f http://localhost/"))
        .interval(Duration::from_secs(15))
        .timeout(Duration::from_secs(200))
        .start_period(Duration::from_secs(5))
        .start_interval(Duration::from_secs(5))
        .retries(5)
        .build()
        .unwrap();
    let expected = expect!["HEALTHCHECK --interal=15 --timeout=200 --start-period=5 --start-interval=5 --retries=5 CMD curl -f http://localhost/"];
    expected.assert_eq(&healthcheck.to_string());
}

#[test]
fn add_git_url_parse_failure() {
    let add = AddGitBuilder::builder()
        .keep_git_dir(true)
        .git_ref("!@#~~https://github.com/moby/buildkit.git#v0.10.1")
        .dir("/buildkit")
        .build();
    assert!(add.is_err());
}

#[test]
fn add_http_url_parse_failure() {
    let add = AddHttpBuilder::builder()
        .src("!@#~~https://github.com/moby/buildkit.git#v0.10.1")
        .dest("/buildkit")
        .build();
    assert!(add.is_err());
}

#[test]
fn additional_urls() {
    let add = AddGitBuilder::builder()
        .keep_git_dir(true)
        .git_ref("https://github.com/rust-lang/rust/issues?labels=E-easy&state=open")
        .dir("/buildkit")
        .build();
    if add.is_err() {
        eprintln!("{:#?}", &add);
    }
    assert!(add.is_ok());

    //Data url
    let add = AddGitBuilder::builder()
        .keep_git_dir(true)
        .git_ref("data:text/plain,Hello?World#")
        .dir("/buildkit")
        .build();
    if add.is_err() {
        eprintln!("{:#?}", &add);
    }
    assert!(add.is_ok());
}

#[test]
fn add_http_percent_encoding() {
    let add = AddGitBuilder::builder()
        .keep_git_dir(true)
        .git_ref("https://foobar.com/foo%20%3Cbar%3E")
        .dir("/buildkit")
        .build();
    if add.is_err() {
        eprintln!("{:#?}", &add);
    }
    assert!(add.is_ok());
}
