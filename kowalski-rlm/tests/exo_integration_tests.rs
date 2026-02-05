use httpmock::prelude::*;
use kowalski_rlm::exo_cluster_manager::{ExoClusterManager, ExoClusterState, ExoDeviceInfo, ExoModelListResponse, ExoModelInfo};
use serde_json::json;

#[tokio::test]
async fn test_exo_cluster_discovery_and_models() {
    let server = MockServer::start();

    let devices = ExoClusterState {
        devices: vec![ExoDeviceInfo {
            id: "device-1".to_string(),
            address: "127.0.0.1:9999".to_string(),
            capabilities: Default::default(),
        }],
    };

    let _state_mock = server.mock(|when, then| {
        when.method(GET).path("/state");
        then.status(200).json_body_obj(&devices);
    });

    let models = ExoModelListResponse {
        models: vec![ExoModelInfo {
            name: "llama3.2".to_string(),
            size: Some(1234),
            digest: Some("abc".to_string()),
        }],
    };

    let _models_mock = server.mock(|when, then| {
        when.method(GET).path("/models");
        then.status(200).json_body_obj(&models);
    });

    let manager = ExoClusterManager::new(server.url(""))
        .await
        .expect("Failed to init exo cluster manager");

    let discovered = manager.list_devices().await.expect("List devices failed");
    assert_eq!(discovered.len(), 1);
    assert_eq!(discovered[0].id, "device-1");

    let available_models = manager.list_models().await.expect("List models failed");
    assert_eq!(available_models.len(), 1);
    assert_eq!(available_models[0].name, "llama3.2");
}

#[tokio::test]
async fn test_exo_remote_repl_request() {
    let server = MockServer::start();

    let _state_mock = server.mock(|when, then| {
        when.method(GET).path("/state");
        then.status(200)
            .json_body_obj(&ExoClusterState { devices: vec![] });
    });

    let _repl_mock = server.mock(|when, then| {
        when.method(POST).path("/api/repl/execute");
        then.status(200).json_body(json!({
            "stdout": "hello",
            "stderr": "",
            "exit_code": 0
        }));
    });

    let manager = ExoClusterManager::new(server.url(""))
        .await
        .expect("Failed to init exo cluster manager");

    let response = manager
        .send_repl_request(
            "device-1",
            kowalski_rlm::exo_cluster_manager::REPLRequest {
                language: "python".to_string(),
                code: "print('hello')".to_string(),
                timeout_ms: 1000,
                max_output_bytes: 10000,
            },
        )
        .await
        .expect("REPL request failed");

    assert_eq!(response.stdout.trim(), "hello");
    assert_eq!(response.exit_code, 0);
}
