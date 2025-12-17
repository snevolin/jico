use anyhow::Result;
use assert_cmd::prelude::*;
use httpmock::prelude::*;
use serde_json::{Value, json};
use std::process::Command;

fn base_env(server: &MockServer) -> Vec<(&'static str, String)> {
    vec![
        ("JIRA_BASE_URL", server.base_url()),
        ("JIRA_EMAIL", "user@example.com".to_string()),
        ("JIRA_API_TOKEN", "token".to_string()),
    ]
}

#[test]
fn cli_create_with_new_fields() -> Result<()> {
    let server = MockServer::start();
    let expected_body = json!({
        "fields": {
            "project": { "key": "ACME" },
            "summary": "Title",
            "issuetype": { "name": "Task" },
            "description": {
                "type": "doc",
                "version": 1,
                "content": [{
                    "type": "paragraph",
                    "content": [{
                        "type": "text",
                        "text": "Desc"
                    }]
                }]
            },
            "labels": ["bug", "ui"],
            "priority": { "name": "High" },
            "assignee": { "accountId": "abc" }
        }
    });
    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/rest/api/3/issue")
            .json_body(expected_body.clone());
        then.status(201)
            .json_body(json!({ "id": "10000", "key": "ACME-1" }));
    });

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("jico"));
    for (key, val) in base_env(&server) {
        cmd.env(key, val);
    }
    let assert = cmd
        .arg("create")
        .arg("Title")
        .arg("--description")
        .arg("Desc")
        .arg("--project")
        .arg("ACME")
        .arg("--issue-type")
        .arg("Task")
        .arg("--labels")
        .arg("bug,ui")
        .arg("--priority")
        .arg("High")
        .arg("--assignee")
        .arg("abc")
        .assert()
        .success();

    mock.assert();
    let stdout = String::from_utf8(assert.get_output().stdout.clone())?;
    let value: Value = serde_json::from_str(&stdout)?;
    assert_eq!(value["id"], "10000");
    assert_eq!(value["key"], "ACME-1");
    Ok(())
}

#[test]
fn cli_update_with_new_fields() -> Result<()> {
    let server = MockServer::start();
    let expected_body = json!({
        "fields": {
            "summary": "New summary",
            "labels": ["backend"],
            "priority": { "name": "Medium" },
            "assignee": { "accountId": "xyz" }
        }
    });
    let mock = server.mock(|when, then| {
        when.method(PUT)
            .path("/rest/api/3/issue/ACME-1")
            .json_body(expected_body.clone());
        then.status(200).json_body(json!({ "ok": true }));
    });

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("jico"));
    for (key, val) in base_env(&server) {
        cmd.env(key, val);
    }
    let assert = cmd
        .arg("update")
        .arg("ACME-1")
        .arg("--summary")
        .arg("New summary")
        .arg("--labels")
        .arg("backend")
        .arg("--priority")
        .arg("Medium")
        .arg("--assignee")
        .arg("xyz")
        .assert()
        .success();

    mock.assert();
    let stdout = String::from_utf8(assert.get_output().stdout.clone())?;
    let value: Value = serde_json::from_str(&stdout)?;
    assert_eq!(value["ok"], true);
    Ok(())
}
