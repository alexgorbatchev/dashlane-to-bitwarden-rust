use cli_command_kit::Mode;
use dashlane_to_bitwarden::convert;
use dashlane_to_bitwarden::{command, convert_to_file, entry, help_options};
use std::fs;
use std::{
    ffi::OsString,
    io::{self, Write},
    path::Path,
};

#[test]
fn preserves_secret_whitespace_and_all_category_relationships() {
    let dir = tempfile::tempdir_in(".tmp").unwrap();
    fs::write(dir.path().join("credentials.csv"), "title,username,password,note,url,username2,username3,otpUrl\nExample, user ,  secret  ,\" note\nsecond line \",https://example.com,other,third,otpauth://test\n").unwrap();
    fs::write(dir.path().join("payments.csv"), "type,name,account_name,account_holder,issuing_bank,cc_number,expiration_month,expiration_year,code,routing_number,account_number,country,note\npayment_card,Card,,Ada,Visa,4111111111111111,1,2030,123,,,,card note\nbank,,Checking,Ada,Bank,,,,,0001,0002,US,bank note\n").unwrap();
    fs::write(dir.path().join("personalInfo.csv"), "type,first_name,last_name,middle_name,address,city,state,zip,country,email,phone_number,login,title\nname,Ada,Lovelace,Byron,Street,London,State,123,GB,a@example.com,555,ada,Ms\n").unwrap();
    fs::write(dir.path().join("ids.csv"), "type,number,name,issue_date,expiration_date,place_of_issue,state\npassport,AB123,Travel,2020,2030,GB,State\nsocial_security,SSN,,,,,\nlicense,DL,,,,,\n").unwrap();
    fs::write(
        dir.path().join("securenotes.csv"),
        "title,note\nPrivate,\"  preserve note  \"\n",
    )
    .unwrap();
    let export = convert(dir.path()).unwrap();
    let value = serde_json::to_value(export).unwrap();
    assert_eq!(value["items"].as_array().unwrap().len(), 8);
    let items = value["items"].as_array().unwrap();
    assert_eq!(items[0]["login"]["password"], "  secret  ");
    assert_eq!(items[0]["notes"], " note\nsecond line ");
    assert_eq!(items[0]["login"]["username"], " user ");
    assert_eq!(items[0]["login"]["uris"][0]["uri"], "https://example.com");
    assert_eq!(items[0]["fields"][1]["value"], "third");
    assert_eq!(items[1]["card"]["number"], "4111111111111111");
    assert_eq!(items[2]["fields"][1]["type"], 1);
    assert_eq!(items[2]["fields"][2]["value"], "0002");
    assert_eq!(items[3]["name"], "Ada Lovelace");
    assert_eq!(items[3]["identity"]["address1"], "Street");
    assert_eq!(items[4]["identity"]["passportNumber"], "AB123");
    assert_eq!(items[5]["identity"]["ssn"], "SSN");
    assert_eq!(items[6]["identity"]["licenseNumber"], "DL");
    assert_eq!(items[7]["notes"], "  preserve note  ");
    let folders = value["folders"].as_array().unwrap();
    for item in items {
        let folder = folders
            .iter()
            .find(|folder| folder["id"] == item["folderId"])
            .unwrap();
        assert!(folder["name"].is_string());
        assert!(
            uuid::Uuid::parse_str(item["id"].as_str().unwrap())
                .unwrap()
                .get_version_num()
                == 4
        );
    }
}

#[test]
fn malformed_csv_and_missing_input_are_failures() {
    let dir = tempfile::tempdir_in(".tmp").unwrap();
    fs::write(
        dir.path().join("credentials.csv"),
        "title,password\nExample,value,extra\n",
    )
    .unwrap();
    assert!(convert(dir.path()).is_err());
    assert!(convert(&dir.path().join("missing")).is_err());
    fs::write(
        dir.path().join("credentials.csv"),
        "password,password\na,b\n",
    )
    .unwrap();
    assert!(convert(dir.path()).is_err());
}

fn archive(path: &Path, entries: &[(&str, &[u8])]) {
    let mut archive = zip::ZipWriter::new(fs::File::create(path).unwrap());
    for (name, bytes) in entries {
        archive
            .start_file(*name, zip::write::SimpleFileOptions::default())
            .unwrap();
        archive.write_all(bytes).unwrap();
    }
    archive.finish().unwrap();
}

#[test]
fn nested_archives_and_name_fallbacks_preserve_source_contract() {
    let dir = tempfile::tempdir_in(".tmp").unwrap();
    let path = dir.path().join("export.ZIP");
    archive(&path, &[
        ("export/credentials.csv", b"title,url,password\n,https://example.com,secret\n,, \n"),
        ("export/payments.csv", b"type,name,account_name\npayment_card,,My card\npayment_card,,\nbank,,\nother,,ignored\n"),
        ("export/personalInfo.csv", b"type,item_name\nname,\nname,Fallback\nemail,ignored\n"),
        ("export/ids.csv", b"type,number\npassport,AB\nunknown,XY\n"),
        ("export/securenotes.csv", b"title,note\n,content\n"),
        ("export/unrelated.txt", b"ignored"),
    ]);
    let value = serde_json::to_value(convert(&path).unwrap()).unwrap();
    let items = value["items"].as_array().unwrap();
    assert_eq!(
        items
            .iter()
            .map(|item| item["name"].as_str().unwrap())
            .collect::<Vec<_>>(),
        [
            "https://example.com",
            "Login",
            "My card",
            "Card",
            "Bank Account",
            "Identity",
            "Fallback",
            "passport - AB",
            "unknown - XY",
            "Secure Note"
        ]
    );
    assert_eq!(items[1]["login"]["password"], " ");
    assert_eq!(items[1]["login"]["uris"], serde_json::json!([]));
    assert!(items[8]["identity"]["passportNumber"].is_null());
    assert_eq!(items[8]["fields"][0]["value"], "unknown");
    assert_eq!(items[8]["fields"][1]["value"], "XY");
    assert_eq!(items[8]["fields"][1]["type"], 1);
    archive(
        &path,
        &[
            ("a/credentials.csv", b"password\nx\n"),
            ("b/credentials.csv", b"password\ny\n"),
        ],
    );
    assert!(
        convert(&path)
            .err()
            .unwrap()
            .to_string()
            .contains("ambiguous")
    );
    fs::write(&path, "not a ZIP").unwrap();
    assert!(convert(&path).is_err());
    let plain = dir.path().join("plain.csv");
    fs::write(&plain, "header\nx\n").unwrap();
    assert!(convert(&plain).is_err());
}

#[test]
fn missing_empty_and_bom_csv_are_supported_but_read_failures_abort() {
    let dir = tempfile::tempdir_in(".tmp").unwrap();
    assert!(convert(dir.path()).unwrap().items.is_empty());
    fs::write(dir.path().join("credentials.csv"), "").unwrap();
    assert!(convert(dir.path()).unwrap().items.is_empty());
    fs::write(
        dir.path().join("credentials.csv"),
        "\u{feff}title,password\r\nExample,secret\r\n",
    )
    .unwrap();
    let value = serde_json::to_value(convert(dir.path()).unwrap()).unwrap();
    assert_eq!(value["items"][0]["name"], "Example");
    fs::write(
        dir.path().join("credentials.csv"),
        b"title,password\nExample,\xff\n",
    )
    .unwrap();
    assert!(convert(dir.path()).is_err());
    fs::remove_file(dir.path().join("credentials.csv")).unwrap();
    fs::create_dir(dir.path().join("credentials.csv")).unwrap();
    assert!(convert(dir.path()).is_err());
    #[cfg(unix)]
    {
        use std::os::unix::net::UnixListener;
        let socket = dir.path().join("socket");
        let _listener = UnixListener::bind(&socket).unwrap();
        assert!(convert(&socket).is_err());
    }
}

#[test]
fn output_is_private_atomic_and_cannot_replace_a_source_file() {
    let dir = tempfile::tempdir_in(".tmp").unwrap();
    let source = dir.path().join("source");
    fs::create_dir(&source).unwrap();
    let csv = source.join("credentials.csv");
    fs::write(&csv, "title,password\nExample,secret\n").unwrap();
    let output = dir.path().join("nested/output.json");
    assert_eq!(convert_to_file(&source, &output).unwrap().items.len(), 1);
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&fs::read(&output).unwrap()).unwrap()["items"]
            [0]["login"]["password"],
        "secret"
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            output.metadata().unwrap().permissions().mode() & 0o777,
            0o600
        );
    }
    let before = fs::read(&output).unwrap();
    fs::write(&csv, "title,password\nExample,secret,extra\n").unwrap();
    assert!(convert_to_file(&source, &output).is_err());
    assert_eq!(fs::read(&output).unwrap(), before);
    fs::write(&csv, "title,password\nExample,secret\n").unwrap();
    assert!(convert_to_file(&source, &csv).is_err());
    let zip = dir.path().join("source.zip");
    archive(
        &zip,
        &[("credentials.csv", b"title,password\nExample,secret\n")],
    );
    assert!(convert_to_file(&zip, &zip).is_err());
    let block = dir.path().join("block");
    fs::write(&block, "old").unwrap();
    assert!(convert_to_file(&source, &block.join("file.json")).is_err());
    let occupied = dir.path().join("directory");
    fs::create_dir(&occupied).unwrap();
    assert!(convert_to_file(&source, &occupied).is_err());
    assert!(occupied.is_dir());
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 5);
    convert_to_file(&source, &output).unwrap();
}

fn invoke(args: &[&str], mode: Mode) -> (u8, String, String) {
    let mut out = Vec::new();
    let mut err = Vec::new();
    let code = entry(args.iter().map(OsString::from), &mut out, &mut err, mode);
    (
        code,
        String::from_utf8(out).unwrap(),
        String::from_utf8(err).unwrap(),
    )
}

#[test]
fn cli_conversion_metadata_and_validation_work_in_both_modes() {
    let dir = tempfile::tempdir_in(".tmp").unwrap();
    let source = dir.path().join("source");
    fs::create_dir(&source).unwrap();
    fs::write(source.join("securenotes.csv"), "title,note\nExample,note\n").unwrap();
    let output = dir.path().join("output.json");
    let source = source.to_str().unwrap();
    let output = output.to_str().unwrap();
    for mode in [Mode::Human, Mode::Agent] {
        for args in [
            vec!["dashlane-to-bitwarden", source, output],
            vec!["dashlane-to-bitwarden", "export", "convert", source, output],
        ] {
            let (code, out, err) = invoke(&args, mode);
            assert_eq!(code, 0, "{err}");
            assert!(err.is_empty());
            if mode == Mode::Agent {
                assert_eq!(
                    serde_json::from_str::<serde_json::Value>(&out).unwrap()["secure_notes"],
                    1
                );
            } else {
                assert!(out.contains("Secure Notes: 1"));
            }
        }
        assert_eq!(
            invoke(&["dashlane-to-bitwarden", "--version"], mode),
            (0, "0.1.0\n".into(), String::new())
        );
        assert_eq!(invoke(&["dashlane-to-bitwarden", "-v"], mode).1, "0.1.0\n");
        let skill = invoke(&["dashlane-to-bitwarden", "skill"], mode);
        assert_eq!(skill, (0, "Install the dashlane-to-bitwarden skill using the Skills CLI:\n\n  npx skills add alexgorbatchev/dashlane-to-bitwarden --skill dashlane-to-bitwarden\n\nSkill:\n\n  https://github.com/alexgorbatchev/dashlane-to-bitwarden/tree/main/skills/dashlane-to-bitwarden\n".into(), String::new()));
        for args in [
            vec!["dashlane-to-bitwarden", "--help"],
            vec!["dashlane-to-bitwarden", "export", "--help"],
            vec!["dashlane-to-bitwarden", "export", "convert", "--help"],
            vec!["dashlane-to-bitwarden", "skill", "--help"],
            vec!["dashlane-to-bitwarden", "help"],
            vec!["dashlane-to-bitwarden", "help", "skill"],
            vec!["dashlane-to-bitwarden", "help", "export", "convert"],
            vec!["dashlane-to-bitwarden", "help", "help"],
            vec!["dashlane-to-bitwarden", "export", "help"],
            vec!["dashlane-to-bitwarden", "export", "help", "convert"],
            vec!["dashlane-to-bitwarden", "export", "help", "help"],
        ] {
            let (code, out, err) = invoke(&args, mode);
            assert_eq!(code, 0, "{args:?}: {err}");
            assert_eq!(out.starts_with("ALERT:"), mode == Mode::Agent);
        }
        for args in [
            vec!["dashlane-to-bitwarden"],
            vec!["dashlane-to-bitwarden", source],
            vec!["dashlane-to-bitwarden", "--bad"],
            vec!["dashlane-to-bitwarden", "skill", "extra"],
            vec!["dashlane-to-bitwarden", source, output, "extra"],
            vec!["dashlane-to-bitwarden", "export"],
        ] {
            let (code, out, err) = invoke(&args, mode);
            assert_eq!(code, 1, "{args:?}: {out}");
            assert!(out.is_empty());
            assert!(!err.is_empty());
        }
    }
    let doc = fs::read_to_string("skills/dashlane-to-bitwarden/SKILL.md").unwrap();
    assert!(doc.contains("name: dashlane-to-bitwarden"));
    let omissions = cli_command_kit::skill_omissions(&command(), &help_options(), &doc);
    assert!(omissions.is_empty(), "{omissions:?}");
}

struct Broken;
impl Write for Broken {
    fn write(&mut self, _: &[u8]) -> io::Result<usize> {
        Err(io::Error::other("writer failed"))
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[test]
fn output_failures_and_executable_errors_are_reported_once_without_secrets() {
    for args in [
        vec!["dashlane-to-bitwarden", "--help"],
        vec!["dashlane-to-bitwarden", "--version"],
        vec!["dashlane-to-bitwarden", "skill"],
    ] {
        let mut err = Vec::new();
        assert_eq!(
            entry(
                args.into_iter().map(OsString::from),
                &mut Broken,
                &mut err,
                Mode::Agent
            ),
            1
        );
        assert!(String::from_utf8(err).unwrap().contains("writer failed"));
    }
    assert_eq!(
        entry(
            [OsString::from("dashlane-to-bitwarden")],
            &mut Vec::new(),
            &mut Broken,
            Mode::Human
        ),
        1
    );
    let dir = tempfile::tempdir_in(".tmp").unwrap();
    fs::write(
        dir.path().join("credentials.csv"),
        "title,password\nExample,secret,extra\n",
    )
    .unwrap();
    let output = dir.path().join("output.json");
    let result = std::process::Command::new(env!("CARGO_BIN_EXE_dashlane-to-bitwarden"))
        .arg(dir.path())
        .arg(&output)
        .env("AGENT", "1")
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(1));
    assert!(result.stdout.is_empty());
    let err = String::from_utf8(result.stderr).unwrap();
    assert_eq!(err.lines().count(), 1);
    assert!(!err.contains("secret"));
    assert!(!output.exists());
    fs::write(
        dir.path().join("credentials.csv"),
        "title,password\nExample,secret\n",
    )
    .unwrap();
    let result = std::process::Command::new(env!("CARGO_BIN_EXE_dashlane-to-bitwarden"))
        .arg(dir.path())
        .arg(&output)
        .env("AGENT", "1")
        .output()
        .unwrap();
    assert!(result.status.success());
    assert!(result.stderr.is_empty());
    let summary: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(summary["logins"], 1);
    let mut err = Vec::new();
    assert_eq!(
        entry(
            [
                OsString::from("dashlane-to-bitwarden"),
                dir.path().as_os_str().into(),
                output.as_os_str().into()
            ],
            &mut Broken,
            &mut err,
            Mode::Human
        ),
        1
    );
    assert!(output.exists());
    assert!(String::from_utf8(err).unwrap().contains("writer failed"));
    let mut err = Vec::new();
    assert_eq!(
        entry(
            [
                OsString::from("dashlane-to-bitwarden"),
                dir.path().as_os_str().into(),
                output.as_os_str().into()
            ],
            &mut Broken,
            &mut err,
            Mode::Agent
        ),
        1
    );
    assert!(String::from_utf8(err).unwrap().contains("writer failed"));
}

#[test]
fn native_relative_destinations_and_home_folded_paths_are_correct() {
    let dir = tempfile::tempdir_in(".tmp").unwrap();
    let root = dir.path().canonicalize().unwrap();
    fs::create_dir(root.join("source")).unwrap();
    let result = std::process::Command::new(env!("CARGO_BIN_EXE_dashlane-to-bitwarden"))
        .current_dir(&root)
        .args(["source", "output.json"])
        .env("AGENT", "0")
        .output()
        .unwrap();
    assert!(result.status.success());
    assert!(root.join("output.json").is_file());
    assert!(
        String::from_utf8(result.stdout)
            .unwrap()
            .starts_with("Created: output.json\n")
    );
    let input = root.join("source");
    let output = root.join("absolute.json");
    let (code, out, err) = invoke(
        &[
            "dashlane-to-bitwarden",
            input.to_str().unwrap(),
            output.to_str().unwrap(),
        ],
        Mode::Agent,
    );
    assert_eq!(code, 0, "{err}");
    let summary: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(summary["output"], cli_command_kit::display_path(&output));
    let home = std::env::home_dir().unwrap();
    let missing = home.join(format!("missing-{}", uuid::Uuid::new_v4()));
    let (code, out, err) = invoke(
        &[
            "dashlane-to-bitwarden",
            missing.to_str().unwrap(),
            output.to_str().unwrap(),
        ],
        Mode::Human,
    );
    assert_eq!(code, 1);
    assert!(out.is_empty());
    assert!(err.contains("~/missing-"));
    assert!(!err.contains(home.to_str().unwrap()));
}
