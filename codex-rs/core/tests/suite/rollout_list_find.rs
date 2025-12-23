#![allow(clippy::unwrap_used, clippy::expect_used)]
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

use codex_core::find_conversation_path_by_id_str;
use pretty_assertions::assert_eq;
use tempfile::TempDir;
use uuid::Uuid;

/// Create sessions/YYYY/MM/DD and write a minimal rollout file containing the
/// provided conversation id in the SessionMeta line. Returns the absolute path.
fn write_minimal_rollout_with_id(codex_home: &Path, id: Uuid) -> PathBuf {
    let sessions = codex_home.join("sessions/2024/01/01");
    std::fs::create_dir_all(&sessions).unwrap();

    let file = sessions.join(format!("rollout-2024-01-01T00-00-00-{id}.jsonl"));
    let mut f = std::fs::File::create(&file).unwrap();
    // Minimal first line: session_meta with the id so content search can find it
    writeln!(
        f,
        "{}",
        serde_json::json!({
            "timestamp": "2024-01-01T00:00:00.000Z",
            "type": "session_meta",
            "payload": {
                "id": id,
                "timestamp": "2024-01-01T00:00:00Z",
                "instructions": null,
                "cwd": ".",
                "originator": "test",
                "cli_version": "test",
                "model_provider": "test-provider"
            }
        })
    )
    .unwrap();

    file
}

#[tokio::test]
async fn find_locates_rollout_file_by_id() {
    let home = TempDir::new().unwrap();
    let id = Uuid::new_v4();
    let expected = write_minimal_rollout_with_id(home.path(), id);

    let found = find_conversation_path_by_id_str(home.path(), &id.to_string())
        .await
        .unwrap();

    assert_eq!(found, Some(expected));
}

#[tokio::test]
async fn find_handles_gitignore_covering_codex_home_directory() {
    let repo = TempDir::new().unwrap();
    let codex_home = repo.path().join(".codex");
    std::fs::create_dir_all(&codex_home).unwrap();
    std::fs::write(repo.path().join(".gitignore"), ".codex/**\n").unwrap();
    let id = Uuid::new_v4();
    let expected = write_minimal_rollout_with_id(&codex_home, id);

    let found = find_conversation_path_by_id_str(&codex_home, &id.to_string())
        .await
        .unwrap();

    assert_eq!(found, Some(expected));
}

#[tokio::test]
async fn find_supports_unique_prefix_match() {
    let home = TempDir::new().unwrap();
    let id = Uuid::parse_str("019b4b3f-0c97-7601-9a6e-8cd48e294ff2").unwrap();
    let expected = write_minimal_rollout_with_id(home.path(), id);

    let found = find_conversation_path_by_id_str(home.path(), "019b")
        .await
        .unwrap();

    assert_eq!(found, Some(expected));
}

#[tokio::test]
async fn find_returns_newest_match_for_prefix() {
    let home = TempDir::new().unwrap();
    let id1 = Uuid::parse_str("abcd0000-0000-0000-0000-000000000001").unwrap();
    let id2 = Uuid::parse_str("abcd0000-0000-0000-0000-000000000002").unwrap();
    write_minimal_rollout_with_id(home.path(), id1);
    let newer = write_minimal_rollout_with_id(home.path(), id2);

    let found = find_conversation_path_by_id_str(home.path(), "abcd")
        .await
        .unwrap();

    assert_eq!(found, Some(newer));
}

#[tokio::test]
async fn find_ignores_granular_gitignore_rules() {
    let home = TempDir::new().unwrap();
    let id = Uuid::new_v4();
    let expected = write_minimal_rollout_with_id(home.path(), id);
    std::fs::write(home.path().join("sessions/.gitignore"), "*.jsonl\n").unwrap();

    let found = find_conversation_path_by_id_str(home.path(), &id.to_string())
        .await
        .unwrap();

    assert_eq!(found, Some(expected));
}
