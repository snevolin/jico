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
fn cli_create_with_parent_defaults_to_subtask() -> Result<()> {
    let server = MockServer::start();
    let expected_body = json!({
        "fields": {
            "project": { "key": "ACME" },
            "summary": "Child issue",
            "issuetype": { "name": "Sub-task" },
            "description": null,
            "parent": { "key": "ACME-1" }
        }
    });
    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/rest/api/3/issue")
            .json_body(expected_body.clone());
        then.status(201).json_body(json!({ "id": "10001" }));
    });

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("jico"));
    for (key, val) in base_env(&server) {
        cmd.env(key, val);
    }
    let assert = cmd
        .arg("create")
        .arg("Child issue")
        .arg("--project")
        .arg("ACME")
        .arg("--parent")
        .arg("ACME-1")
        .assert()
        .success();

    mock.assert();
    let stdout = String::from_utf8(assert.get_output().stdout.clone())?;
    let value: Value = serde_json::from_str(&stdout)?;
    assert_eq!(value["id"], "10001");
    Ok(())
}

#[test]
fn cli_view_subtasks() -> Result<()> {
    let server = MockServer::start();
    let response_body = json!({
        "fields": {
            "subtasks": [
                { "id": "20001", "key": "ACME-2" },
                { "id": "20002", "key": "ACME-3" }
            ]
        }
    });
    let mock = server.mock(|when, then| {
        when.method(GET)
            .path("/rest/api/3/issue/ACME-1")
            .query_param("fields", "subtasks");
        then.status(200).json_body(response_body.clone());
    });

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("jico"));
    for (key, val) in base_env(&server) {
        cmd.env(key, val);
    }
    let assert = cmd
        .arg("view")
        .arg("ACME-1")
        .arg("--subtasks")
        .assert()
        .success();

    mock.assert();
    let stdout = String::from_utf8(assert.get_output().stdout.clone())?;
    let value: Value = serde_json::from_str(&stdout)?;
    assert_eq!(value, response_body["fields"]["subtasks"]);
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

#[test]
fn cli_update_with_parent_defaults_to_subtask() -> Result<()> {
    let server = MockServer::start();
    let expected_body = json!({
        "fields": {
            "issuetype": { "name": "Sub-task" },
            "parent": { "key": "ACME-1" }
        }
    });
    let mock = server.mock(|when, then| {
        when.method(PUT)
            .path("/rest/api/3/issue/ACME-2")
            .json_body(expected_body.clone());
        then.status(200).json_body(json!({ "ok": true }));
    });

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("jico"));
    for (key, val) in base_env(&server) {
        cmd.env(key, val);
    }
    let assert = cmd
        .arg("update")
        .arg("ACME-2")
        .arg("--parent")
        .arg("ACME-1")
        .assert()
        .success();

    mock.assert();
    let stdout = String::from_utf8(assert.get_output().stdout.clone())?;
    let value: Value = serde_json::from_str(&stdout)?;
    assert_eq!(value["ok"], true);
    Ok(())
}

#[test]
fn cli_link_blocked_by_creates_blocks_issue_link() -> Result<()> {
    let server = MockServer::start();
    let expected_body = json!({
        "type": { "name": "Blocks" },
        "outwardIssue": { "key": "MG-26" },
        "inwardIssue": { "key": "MG-3" }
    });
    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/rest/api/3/issueLink")
            .json_body(expected_body.clone());
        then.status(201);
    });

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("jico"));
    for (key, val) in base_env(&server) {
        cmd.env(key, val);
    }
    let assert = cmd
        .arg("link")
        .arg("MG-26")
        .arg("--to")
        .arg("MG-3")
        .arg("--relation")
        .arg("blocked-by")
        .assert()
        .success();

    mock.assert();
    let stdout = String::from_utf8(assert.get_output().stdout.clone())?;
    let value: Value = serde_json::from_str(&stdout)?;
    assert_eq!(value, json!({}));
    Ok(())
}

#[test]
fn cli_link_clones_creates_cloners_issue_link() -> Result<()> {
    let server = MockServer::start();
    let expected_body = json!({
        "type": { "name": "Cloners" },
        "outwardIssue": { "key": "MG-3" },
        "inwardIssue": { "key": "MG-26" }
    });
    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/rest/api/3/issueLink")
            .json_body(expected_body.clone());
        then.status(201);
    });

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("jico"));
    for (key, val) in base_env(&server) {
        cmd.env(key, val);
    }
    let assert = cmd
        .arg("link")
        .arg("MG-26")
        .arg("--to")
        .arg("MG-3")
        .arg("--relation")
        .arg("clones")
        .assert()
        .success();

    mock.assert();
    let stdout = String::from_utf8(assert.get_output().stdout.clone())?;
    let value: Value = serde_json::from_str(&stdout)?;
    assert_eq!(value, json!({}));
    Ok(())
}
