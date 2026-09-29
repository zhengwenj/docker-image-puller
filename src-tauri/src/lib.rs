use flate2::read::GzDecoder;
use reqwest::blocking::{Client, ClientBuilder, Response};
use reqwest::header::{ACCEPT, ACCEPT_ENCODING, CONTENT_TYPE, WWW_AUTHENTICATE};
use reqwest::{Method, Proxy};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, Cursor, Read};
use std::path::{Path, PathBuf};
use tar::{Builder as TarBuilder, Header as TarHeader};
use tauri::Manager;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProxyConfig {
    http_proxy: Option<String>,
    https_proxy: Option<String>,
    no_proxy: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RegistryAuth {
    username: Option<String>,
    password: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SearchImagesRequest {
    keyword: String,
    limit: Option<usize>,
    proxy: Option<ProxyConfig>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SearchImageResult {
    full_name: String,
    description: String,
    stars: u64,
    pulls: u64,
    is_official: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PullImageRequest {
    registry: Option<String>,
    repository: String,
    tag: Option<String>,
    platform_os: Option<String>,
    platform_architecture: Option<String>,
    output_dir: String,
    tar_file_name: Option<String>,
    proxy: Option<ProxyConfig>,
    auth: Option<RegistryAuth>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PullImageResult {
    image_ref: String,
    tar_path: String,
    layer_count: usize,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TestRegistryAuthRequest {
    registry: Option<String>,
    repository: Option<String>,
    tag: Option<String>,
    proxy: Option<ProxyConfig>,
    auth: Option<RegistryAuth>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct TestRegistryAuthResult {
    image_ref: Option<String>,
    message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PersistedConfig {
    output_dir: String,
    default_tag: String,
    platform_os: String,
    platform_architecture: String,
    http_proxy: String,
    https_proxy: String,
    no_proxy: String,
    username: String,
    password: String,
}

#[derive(Debug, Deserialize)]
struct ManifestList {
    manifests: Vec<ManifestListEntry>,
}

#[derive(Debug, Deserialize)]
struct ManifestListEntry {
    digest: String,
    platform: Option<ManifestPlatform>,
}

#[derive(Debug, Deserialize)]
struct ManifestPlatform {
    architecture: Option<String>,
    os: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ImageManifest {
    config: ManifestDescriptor,
    layers: Vec<ManifestDescriptor>,
}

#[derive(Debug, Deserialize)]
struct ManifestDescriptor {
    #[serde(rename = "mediaType")]
    media_type: Option<String>,
    digest: String,
}

#[derive(Debug)]
struct AuthChallenge {
    realm: String,
    service: Option<String>,
    scope: Option<String>,
}

#[derive(Debug, Clone)]
struct LayerDownloadJob {
    digest: String,
    media_type: Option<String>,
    expected_diff_digest: String,
    blob_tar_path: String,
    temp_blob_path: PathBuf,
}

#[tauri::command]
async fn search_images(request: SearchImagesRequest) -> Result<Vec<SearchImageResult>, String> {
    tauri::async_runtime::spawn_blocking(move || search_images_impl(request))
        .await
        .map_err(|e| format!("后台任务执行失败: {e}"))?
}

fn search_images_impl(request: SearchImagesRequest) -> Result<Vec<SearchImageResult>, String> {
    let keyword = request.keyword.trim();
    if keyword.is_empty() {
        return Ok(Vec::new());
    }

    let limit = request.limit.unwrap_or(20).clamp(1, 100);
    let client = build_http_client(request.proxy.as_ref(), "hub.docker.com")?;
    let page_size = limit.to_string();

    let response = client
        .get("https://hub.docker.com/v2/search/repositories/")
        .query(&[("query", keyword), ("page_size", page_size.as_str())])
        .header(ACCEPT, "application/json")
        // 部分代理会返回损坏的压缩流，显式要求 identity 可显著降低解码失败概率。
        .header(ACCEPT_ENCODING, "identity")
        .send()
        .map_err(format_reqwest_err)?;

    parse_dockerhub_search_response(response)
}

fn parse_dockerhub_search_response(response: Response) -> Result<Vec<SearchImageResult>, String> {
    let status = response.status();
    let content_type = response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("-")
        .to_string();

    let body = response.text().map_err(format_reqwest_err)?;
    if !status.is_success() {
        return Err(format!(
            "搜索镜像失败，状态码: {}，内容类型: {}，响应片段: {}",
            status,
            content_type,
            preview_text(&body, 220)
        ));
    }

    let json: Value = serde_json::from_str(&body).map_err(|err| {
        format!(
            "解析 Docker Hub 搜索响应失败: {}，内容类型: {}，响应片段: {}",
            err,
            content_type,
            preview_text(&body, 220)
        )
    })?;

    let results = json
        .get("results")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            format!(
                "Docker Hub 响应结构异常（缺少 results），内容类型: {}，响应片段: {}",
                content_type,
                preview_text(&body, 220)
            )
        })?;

    let mut items = Vec::with_capacity(results.len());
    for raw in results {
        let full_name = raw
            .get("repo_name")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
            .or_else(|| {
                let namespace = raw.get("namespace").and_then(Value::as_str)?.trim();
                let name = raw.get("name").and_then(Value::as_str)?.trim();
                if namespace.is_empty() || name.is_empty() {
                    None
                } else {
                    Some(format!("{namespace}/{name}"))
                }
            })
            .unwrap_or_else(|| "unknown/unknown".to_string());

        let description = raw
            .get("description")
            .and_then(Value::as_str)
            .or_else(|| raw.get("short_description").and_then(Value::as_str))
            .unwrap_or("")
            .to_string();

        let stars = value_to_u64(raw.get("star_count"));
        let pulls = value_to_u64(raw.get("pull_count"));
        let is_official = raw
            .get("is_official")
            .and_then(Value::as_bool)
            .unwrap_or(false);

        items.push(SearchImageResult {
            full_name,
            description,
            stars,
            pulls,
            is_official,
        });
    }

    Ok(items)
}

#[tauri::command]
async fn load_persisted_config(app: tauri::AppHandle) -> Result<PersistedConfig, String> {
    tauri::async_runtime::spawn_blocking(move || load_persisted_config_impl(app))
        .await
        .map_err(|e| format!("后台任务执行失败: {e}"))?
}

fn load_persisted_config_impl(app: tauri::AppHandle) -> Result<PersistedConfig, String> {
    let conn = open_config_db(&app)?;
    let mut config = default_persisted_config();

    let keys = [
        "output_dir",
        "default_tag",
        "platform_os",
        "platform_architecture",
        "http_proxy",
        "https_proxy",
        "no_proxy",
        "username",
        "password",
    ];
    for key in keys {
        if let Some(value) = load_config_value(&conn, key)? {
            match key {
                "output_dir" => config.output_dir = value,
                "default_tag" => config.default_tag = value,
                "platform_os" => config.platform_os = value,
                "platform_architecture" => config.platform_architecture = value,
                "http_proxy" => config.http_proxy = value,
                "https_proxy" => config.https_proxy = value,
                "no_proxy" => config.no_proxy = value,
                "username" => config.username = value,
                "password" => config.password = value,
                _ => {}
            }
        }
    }

    Ok(config)
}

#[tauri::command]
async fn save_persisted_config(app: tauri::AppHandle, config: PersistedConfig) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || save_persisted_config_impl(app, config))
        .await
        .map_err(|e| format!("后台任务执行失败: {e}"))?
}

fn save_persisted_config_impl(app: tauri::AppHandle, config: PersistedConfig) -> Result<(), String> {
    let mut conn = open_config_db(&app)?;
    let tx = conn.transaction().map_err(format_sqlite_err)?;

    upsert_config_value(&tx, "output_dir", &config.output_dir)?;
    upsert_config_value(&tx, "default_tag", &config.default_tag)?;
    upsert_config_value(&tx, "platform_os", &config.platform_os)?;
    upsert_config_value(
        &tx,
        "platform_architecture",
        &config.platform_architecture,
    )?;
    upsert_config_value(&tx, "http_proxy", &config.http_proxy)?;
    upsert_config_value(&tx, "https_proxy", &config.https_proxy)?;
    upsert_config_value(&tx, "no_proxy", &config.no_proxy)?;
    upsert_config_value(&tx, "username", &config.username)?;
    upsert_config_value(&tx, "password", &config.password)?;

    tx.commit().map_err(format_sqlite_err)?;
    Ok(())
}

#[tauri::command]
async fn pull_image_as_tar(request: PullImageRequest) -> Result<PullImageResult, String> {
    tauri::async_runtime::spawn_blocking(move || pull_image_as_tar_impl(request))
        .await
        .map_err(|e| format!("后台任务执行失败: {e}"))?
}

fn pull_image_as_tar_impl(request: PullImageRequest) -> Result<PullImageResult, String> {
    let registry = normalize_registry(request.registry.as_deref());
    let api_registry = to_api_registry(&registry);
    let (repo, tag) = resolve_repository_and_tag(
        &registry,
        request.repository.trim(),
        request.tag.as_deref(),
    )?;
    let image_ref = if registry == "docker.io" {
        format!("{}:{}", repo, tag)
    } else {
        format!("{}/{}:{}", registry, repo, tag)
    };

    let output_dir = request.output_dir.trim();
    if output_dir.is_empty() {
        return Err("输出目录不能为空".to_string());
    }
    let output_dir = PathBuf::from(output_dir);
    fs::create_dir_all(&output_dir).map_err(|e| format!("创建输出目录失败: {e}"))?;

    let tar_file_name = request
        .tar_file_name
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| format!("{}-{}.tar", sanitize_file_name(&repo), tag));
    let tar_path = output_dir.join(tar_file_name);

    let client = build_http_client(request.proxy.as_ref(), &api_registry)?;
    let auth = request.auth.unwrap_or(RegistryAuth {
        username: None,
        password: None,
    });
    let manifest_accept = [
        "application/vnd.docker.distribution.manifest.list.v2+json",
        "application/vnd.docker.distribution.manifest.v2+json",
        "application/vnd.oci.image.index.v1+json",
        "application/vnd.oci.image.manifest.v1+json",
    ]
    .join(", ");
    let mut token: Option<String> = None;
    let base = format!("https://{}", api_registry);

    let mut manifest_response = request_registry(
        &client,
        Method::GET,
        &format!("{}/v2/{}/manifests/{}", base, repo, tag),
        Some(&manifest_accept),
        &auth,
        &mut token,
    )?;
    let content_type = manifest_response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .to_string();
    let manifest_bytes = read_response_bytes(&mut manifest_response)?;
    let selected_manifest = if content_type.contains("manifest.list.v2+json")
        || content_type.contains("image.index.v1+json")
    {
        let manifest_list: ManifestList = serde_json::from_slice(&manifest_bytes)
            .map_err(|e| format!("解析镜像索引失败: {e}"))?;
        let selected_entry = select_manifest_entry(
            &manifest_list,
            request.platform_os.as_deref(),
            request.platform_architecture.as_deref(),
        )
        .ok_or_else(|| "镜像索引没有可用的 manifest".to_string())?;
        let mut response = request_registry(
            &client,
            Method::GET,
            &format!("{}/v2/{}/manifests/{}", base, repo, selected_entry.digest),
            Some(&manifest_accept),
            &auth,
            &mut token,
        )?;
        let bytes = read_response_bytes(&mut response)?;
        serde_json::from_slice::<ImageManifest>(&bytes)
            .map_err(|e| format!("解析镜像 manifest 失败: {e}"))?
    } else {
        serde_json::from_slice::<ImageManifest>(&manifest_bytes)
            .map_err(|e| format!("解析镜像 manifest 失败: {e}"))?
    };

    let config_blob_path = digest_to_blob_tar_path(&selected_manifest.config.digest)?;
    let config_blob = download_blob_bytes(
        &client,
        &base,
        &repo,
        &selected_manifest.config.digest,
        &auth,
        &mut token,
    )?;
    let config_json: Value =
        serde_json::from_slice(&config_blob).map_err(|e| format!("解析镜像 config 失败: {e}"))?;
    let layer_diff_ids = resolve_layer_diff_ids(&config_json, selected_manifest.layers.len())?;

    let tar_file = File::create(&tar_path).map_err(|e| format!("创建 tar 文件失败: {e}"))?;
    let mut tar_builder = TarBuilder::new(tar_file);

    append_bytes_entry(&mut tar_builder, &config_blob_path, &config_blob)?;

    let mut layer_jobs = Vec::with_capacity(selected_manifest.layers.len());
    for (index, layer) in selected_manifest.layers.iter().enumerate() {
        let expected_diff_digest = layer_diff_ids[index].clone();
        let layer_digest_key = digest_to_key(&expected_diff_digest)?;
        layer_jobs.push(LayerDownloadJob {
            digest: layer.digest.clone(),
            media_type: layer.media_type.clone(),
            expected_diff_digest: expected_diff_digest.clone(),
            blob_tar_path: digest_to_blob_tar_path(&expected_diff_digest)?,
            temp_blob_path: output_dir.join(format!(".tmp-layer-{index}-{layer_digest_key}.blob")),
        });
    }

    download_layers_in_parallel(
        request.proxy.as_ref(),
        &api_registry,
        &base,
        &repo,
        &auth,
        &layer_jobs,
    )?;

    let repo_tag = if registry == "docker.io" {
        format!("{}:{}", repo, tag)
    } else {
        format!("{}/{}:{}", registry, repo, tag)
    };

    let layer_descriptors = layer_jobs
        .iter()
        .map(|layer| {
            let layer_size = fs::metadata(&layer.temp_blob_path)
                .map_err(|e| format!("读取镜像层文件大小失败: {e}"))?
                .len();
            Ok(json!({
                "mediaType": "application/vnd.oci.image.layer.v1.tar",
                "digest": layer.expected_diff_digest,
                "size": layer_size
            }))
        })
        .collect::<Result<Vec<Value>, String>>()?;

    let manifest_payload = json!({
        "schemaVersion": 2,
        "mediaType": "application/vnd.oci.image.manifest.v1+json",
        "config": {
            "mediaType": "application/vnd.oci.image.config.v1+json",
            "digest": selected_manifest.config.digest,
            "size": config_blob.len()
        },
        "layers": layer_descriptors
    });
    let manifest_bytes =
        serde_json::to_vec(&manifest_payload).map_err(|e| format!("序列化 manifest 失败: {e}"))?;
    let manifest_digest = build_sha256_digest(&manifest_bytes);
    let manifest_blob_path = digest_to_blob_tar_path(&manifest_digest)?;
    append_bytes_entry(&mut tar_builder, &manifest_blob_path, &manifest_bytes)?;

    let layer_paths = layer_jobs
        .iter()
        .map(|layer| layer.blob_tar_path.clone())
        .collect::<Vec<String>>();
    let mut layer_sources = serde_json::Map::new();
    for layer in &layer_jobs {
        let layer_size = fs::metadata(&layer.temp_blob_path)
            .map_err(|e| format!("读取镜像层文件大小失败: {e}"))?
            .len();
        layer_sources.insert(
            layer.expected_diff_digest.clone(),
            json!({
                "mediaType": "application/vnd.oci.image.layer.v1.tar",
                "size": layer_size,
                "digest": layer.expected_diff_digest
            }),
        );
    }
    append_json_entry(
        &mut tar_builder,
        "manifest.json",
        &json!([{
            "Config": config_blob_path,
            "RepoTags": [repo_tag],
            "Layers": layer_paths,
            "LayerSources": layer_sources
        }]),
    )?;

    let repo_name = if registry == "docker.io" {
        repo.clone()
    } else {
        format!("{registry}/{repo}")
    };
    if layer_diff_ids.is_empty() {
        return Err("镜像不包含可导出的文件层".to_string());
    }
    let last_layer_key = digest_to_key(&layer_diff_ids[layer_diff_ids.len().saturating_sub(1)])?;
    let mut tag_map = serde_json::Map::new();
    tag_map.insert(tag.clone(), Value::String(last_layer_key));
    let mut repositories_map = serde_json::Map::new();
    repositories_map.insert(repo_name, Value::Object(tag_map));
    append_json_entry(
        &mut tar_builder,
        "repositories",
        &Value::Object(repositories_map),
    )?;

    let compat_entries = build_compat_layer_jsons(&selected_manifest, &layer_diff_ids, &config_json);
    for compat in compat_entries {
        let compat_digest = build_sha256_digest(&compat);
        let compat_blob_path = digest_to_blob_tar_path(&compat_digest)?;
        append_bytes_entry(&mut tar_builder, &compat_blob_path, &compat)?;
    }

    for layer in &layer_jobs {
        append_file_entry(&mut tar_builder, &layer.blob_tar_path, &layer.temp_blob_path)?;
        fs::remove_file(&layer.temp_blob_path).map_err(|e| format!("清理临时层文件失败: {e}"))?;
    }

    append_json_entry(
        &mut tar_builder,
        "oci-layout",
        &json!({
            "imageLayoutVersion": "1.0.0"
        }),
    )?;
    append_json_entry(
        &mut tar_builder,
        "index.json",
        &json!({
            "schemaVersion": 2,
            "mediaType": "application/vnd.oci.image.index.v1+json",
            "manifests": [{
                "mediaType": "application/vnd.oci.image.manifest.v1+json",
                "digest": manifest_digest,
                "size": manifest_bytes.len(),
                "annotations": {
                    "io.containerd.image.name": repo_tag,
                    "org.opencontainers.image.ref.name": tag
                }
            }]
        }),
    )?;

    tar_builder
        .finish()
        .map_err(|e| format!("完成 tar 打包失败: {e}"))?;

    Ok(PullImageResult {
        image_ref,
        tar_path: tar_path.to_string_lossy().to_string(),
        layer_count: selected_manifest.layers.len(),
    })
}

#[tauri::command]
async fn test_registry_auth(
    request: TestRegistryAuthRequest,
) -> Result<TestRegistryAuthResult, String> {
    tauri::async_runtime::spawn_blocking(move || test_registry_auth_impl(request))
        .await
        .map_err(|e| format!("后台任务执行失败: {e}"))?
}

fn test_registry_auth_impl(request: TestRegistryAuthRequest) -> Result<TestRegistryAuthResult, String> {
    let registry = normalize_registry(request.registry.as_deref());
    let api_registry = to_api_registry(&registry);
    let client = build_http_client(request.proxy.as_ref(), &api_registry)?;
    let auth = request.auth.unwrap_or(RegistryAuth {
        username: None,
        password: None,
    });
    let mut token: Option<String> = None;
    let base = format!("https://{}", api_registry);

    let normalized_ref = request
        .repository
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(|value| resolve_repository_and_tag(&registry, value, request.tag.as_deref()))
        .transpose()?;

    if let Some((repo, tag)) = normalized_ref {
        let manifest_accept = [
            "application/vnd.docker.distribution.manifest.list.v2+json",
            "application/vnd.docker.distribution.manifest.v2+json",
            "application/vnd.oci.image.index.v1+json",
            "application/vnd.oci.image.manifest.v1+json",
        ]
        .join(", ");
        let head_result = request_registry(
            &client,
            Method::HEAD,
            &format!("{base}/v2/{repo}/manifests/{tag}"),
            Some(&manifest_accept),
            &auth,
            &mut token,
        );
        if let Err(head_err) = head_result {
            let fallback_to_get = head_err.contains("状态码: 503") || head_err.contains("状态码: 405");
            if fallback_to_get {
                request_registry(
                    &client,
                    Method::GET,
                    &format!("{base}/v2/{repo}/manifests/{tag}"),
                    Some(&manifest_accept),
                    &auth,
                    &mut token,
                )
                .map_err(|get_err| format!("HEAD 校验失败后尝试 GET 仍失败: {get_err}"))?;
            } else {
                return Err(head_err);
            }
        }

        let image_ref = if registry == "docker.io" {
            format!("{repo}:{tag}")
        } else {
            format!("{registry}/{repo}:{tag}")
        };
        return Ok(TestRegistryAuthResult {
            image_ref: Some(image_ref),
            message: "账号密码可用，且目标镜像可访问".to_string(),
        });
    }

    request_registry(
        &client,
        Method::GET,
        &format!("{base}/v2/"),
        None,
        &auth,
        &mut token,
    )?;

    Ok(TestRegistryAuthResult {
        image_ref: None,
        message: "仓库连通与鉴权流程正常（未指定镜像，跳过镜像访问校验）".to_string(),
    })
}

fn build_http_client(proxy: Option<&ProxyConfig>, target_host: &str) -> Result<Client, String> {
    let mut builder = ClientBuilder::new().user_agent(concat!(
        env!("CARGO_PKG_NAME"),
        "/",
        env!("CARGO_PKG_VERSION")
    ));
    if let Some(cfg) = proxy {
        let bypass = should_bypass_proxy(target_host, cfg.no_proxy.as_deref());
        if !bypass {
            if let Some(http_proxy) = clean_opt(cfg.http_proxy.as_deref()) {
                builder = builder.proxy(Proxy::http(http_proxy).map_err(format_reqwest_err)?);
            }
            if let Some(https_proxy) = clean_opt(cfg.https_proxy.as_deref()) {
                builder = builder.proxy(Proxy::https(https_proxy).map_err(format_reqwest_err)?);
            }
        }
    }
    builder.build().map_err(format_reqwest_err)
}

fn request_registry(
    client: &Client,
    method: Method,
    url: &str,
    accept: Option<&str>,
    auth: &RegistryAuth,
    token: &mut Option<String>,
) -> Result<Response, String> {
    let method_name = method.as_str().to_string();
    let mut req = client.request(method.clone(), url);
    if let Some(accept_value) = accept {
        req = req.header(ACCEPT, accept_value);
    }
    if let Some(existing_token) = token.clone() {
        req = req.bearer_auth(existing_token);
    } else if let (Some(username), Some(password)) = (
        clean_opt(auth.username.as_deref()),
        clean_opt(auth.password.as_deref()),
    ) {
        req = req.basic_auth(username, Some(password));
    }

    let response = req.send().map_err(format_reqwest_err)?;
    if response.status() != reqwest::StatusCode::UNAUTHORIZED {
        return ensure_success(response)
            .map_err(|err| format!("{err}，请求: {} {}", method_name, url));
    }

    let challenge = response
        .headers()
        .get(WWW_AUTHENTICATE)
        .and_then(|value| value.to_str().ok())
        .and_then(parse_auth_challenge);

    if let Some(challenge) = challenge {
        let fetched_token = fetch_bearer_token(client, &challenge, auth)?;
        *token = Some(fetched_token.clone());
        let mut retry = client.request(method, url).bearer_auth(fetched_token);
        if let Some(accept_value) = accept {
            retry = retry.header(ACCEPT, accept_value);
        }
        let retry_response = retry.send().map_err(format_reqwest_err)?;
        return ensure_success(retry_response)
            .map_err(|err| format!("{err}，请求: {} {}", method_name, url));
    }

    Err("镜像仓库鉴权失败，请检查用户名密码和仓库权限".to_string())
}

fn fetch_bearer_token(
    client: &Client,
    challenge: &AuthChallenge,
    auth: &RegistryAuth,
) -> Result<String, String> {
    let mut request = client.get(&challenge.realm);
    if let Some(service) = clean_opt(challenge.service.as_deref()) {
        request = request.query(&[("service", service)]);
    }
    if let Some(scope) = clean_opt(challenge.scope.as_deref()) {
        request = request.query(&[("scope", scope)]);
    }
    if let (Some(username), Some(password)) = (
        clean_opt(auth.username.as_deref()),
        clean_opt(auth.password.as_deref()),
    ) {
        request = request.basic_auth(username, Some(password));
    }

    let response = request.send().map_err(format_reqwest_err)?;
    if !response.status().is_success() {
        return Err(format!("获取访问 token 失败，状态码: {}", response.status()));
    }
    let body: Value = response.json().map_err(format_reqwest_err)?;
    if let Some(token) = body.get("token").and_then(Value::as_str) {
        return Ok(token.to_string());
    }
    if let Some(token) = body.get("access_token").and_then(Value::as_str) {
        return Ok(token.to_string());
    }
    Err("鉴权服务返回中不包含 token".to_string())
}

fn parse_auth_challenge(raw_header: &str) -> Option<AuthChallenge> {
    let header = raw_header.trim();
    if !header.to_ascii_lowercase().starts_with("bearer ") {
        return None;
    }
    let fields = &header[7..];
    let mut params = HashMap::new();
    for part in fields.split(',') {
        let mut pair = part.trim().splitn(2, '=');
        let key = pair.next()?.trim().to_ascii_lowercase();
        let value = pair.next()?.trim().trim_matches('"').to_string();
        params.insert(key, value);
    }
    Some(AuthChallenge {
        realm: params.get("realm")?.to_string(),
        service: params.get("service").cloned(),
        scope: params.get("scope").cloned(),
    })
}

fn download_blob_bytes(
    client: &Client,
    base: &str,
    repo: &str,
    digest: &str,
    auth: &RegistryAuth,
    token: &mut Option<String>,
) -> Result<Vec<u8>, String> {
    let mut response = request_registry(
        client,
        Method::GET,
        &format!("{base}/v2/{repo}/blobs/{digest}"),
        None,
        auth,
        token,
    )?;
    read_response_bytes(&mut response)
}

fn download_blob_to_file(
    client: &Client,
    base: &str,
    repo: &str,
    digest: &str,
    media_type: Option<&str>,
    expected_diff_digest: &str,
    auth: &RegistryAuth,
    token: &mut Option<String>,
    destination: &Path,
) -> Result<(), String> {
    let mut response = request_registry(
        client,
        Method::GET,
        &format!("{base}/v2/{repo}/blobs/{digest}"),
        None,
        auth,
        token,
    )?;
    let mut output = File::create(destination).map_err(|e| format!("创建临时层文件失败: {e}"))?;

    let expected_key = digest_to_key(expected_diff_digest)?;
    let source_key = digest_to_key(digest)?;
    let decode_gzip =
        media_type.map(|v| v.contains("gzip")).unwrap_or(false) || expected_key != source_key;

    if decode_gzip {
        let mut decoder = GzDecoder::new(response);
        io::copy(&mut decoder, &mut output).map_err(|e| format!("解压镜像层失败: {e}"))?;
    } else {
        response
            .copy_to(&mut output)
            .map_err(|e| format!("写入镜像层失败: {e}"))?;
    }

    let actual_digest = build_sha256_digest_from_file(destination)?;
    if actual_digest != expected_diff_digest {
        return Err(format!(
            "镜像层 digest 不匹配，期望: {expected_diff_digest}，实际: {actual_digest}"
        ));
    }
    Ok(())
}

fn download_layers_in_parallel(
    proxy: Option<&ProxyConfig>,
    api_registry: &str,
    base: &str,
    repo: &str,
    auth: &RegistryAuth,
    layer_jobs: &[LayerDownloadJob],
) -> Result<(), String> {
    if layer_jobs.is_empty() {
        return Ok(());
    }

    let client = build_http_client(proxy, api_registry)?;
    let max_parallel = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
        .clamp(2, 8);
    let parallel = std::cmp::min(max_parallel, layer_jobs.len());

    for chunk in layer_jobs.chunks(parallel) {
        let mut handles = Vec::with_capacity(chunk.len());
        for layer in chunk {
            let client = client.clone();
            let base = base.to_string();
            let repo = repo.to_string();
            let auth = auth.clone();
            let digest = layer.digest.clone();
            let media_type = layer.media_type.clone();
            let expected_diff_digest = layer.expected_diff_digest.clone();
            let destination = layer.temp_blob_path.clone();
            handles.push(std::thread::spawn(move || {
                let mut token: Option<String> = None;
                download_blob_to_file(
                    &client,
                    &base,
                    &repo,
                    &digest,
                    media_type.as_deref(),
                    &expected_diff_digest,
                    &auth,
                    &mut token,
                    &destination,
                )
            }));
        }

        for handle in handles {
            let result = handle
                .join()
                .map_err(|_| "下载镜像层线程异常退出".to_string())?;
            result?;
        }
    }

    Ok(())
}

fn select_manifest_entry<'a>(
    manifest_list: &'a ManifestList,
    preferred_os: Option<&str>,
    preferred_architecture: Option<&str>,
) -> Option<&'a ManifestListEntry> {
    let normalized_os = preferred_os
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .unwrap_or("linux")
        .to_ascii_lowercase();
    let normalized_arch = preferred_architecture
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .unwrap_or("amd64")
        .to_ascii_lowercase();

    let preferred = manifest_list.manifests.iter().find(|entry| {
        let platform = match &entry.platform {
            Some(p) => p,
            None => return false,
        };
        let os_ok = platform
            .os
            .as_deref()
            .map(|v| v.eq_ignore_ascii_case(&normalized_os))
            .unwrap_or(false);
        let arch_ok = platform
            .architecture
            .as_deref()
            .map(|v| v.eq_ignore_ascii_case(&normalized_arch))
            .unwrap_or(false);
        os_ok && arch_ok
    });
    preferred.or_else(|| manifest_list.manifests.first())
}

fn append_json_entry(
    tar_builder: &mut TarBuilder<File>,
    entry_path: &str,
    value: &Value,
) -> Result<(), String> {
    let bytes = serde_json::to_vec(value).map_err(|e| format!("序列化 JSON 失败: {e}"))?;
    append_bytes_entry(tar_builder, entry_path, &bytes)
}

fn append_bytes_entry(
    tar_builder: &mut TarBuilder<File>,
    entry_path: &str,
    bytes: &[u8],
) -> Result<(), String> {
    let mut header = TarHeader::new_gnu();
    header.set_size(bytes.len() as u64);
    header.set_mode(0o644);
    header.set_cksum();
    tar_builder
        .append_data(&mut header, entry_path, Cursor::new(bytes))
        .map_err(|e| format!("写入 tar 条目失败({entry_path}): {e}"))
}

fn append_file_entry(
    tar_builder: &mut TarBuilder<File>,
    entry_path: &str,
    file_path: &Path,
) -> Result<(), String> {
    tar_builder
        .append_path_with_name(file_path, entry_path)
        .map_err(|e| format!("写入 tar 文件条目失败({entry_path}): {e}"))
}

fn read_response_bytes(response: &mut Response) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    response
        .read_to_end(&mut bytes)
        .map_err(|e| format!("读取响应失败: {e}"))?;
    Ok(bytes)
}

fn ensure_success(response: Response) -> Result<Response, String> {
    if response.status().is_success() {
        Ok(response)
    } else {
        let status = response.status();
        let hint = if status == reqwest::StatusCode::NOT_FOUND {
            "（常见原因：仓库路径不完整、tag 不存在，或走了错误代理）"
        } else if status == reqwest::StatusCode::SERVICE_UNAVAILABLE {
            "（常见原因：代理链路不可用，建议将仓库域名加入 NO_PROXY）"
        } else {
            ""
        };
        Err(format!("请求镜像仓库失败，状态码: {}{}", status, hint))
    }
}

fn resolve_repository_and_tag(
    registry: &str,
    repository: &str,
    request_tag: Option<&str>,
) -> Result<(String, String), String> {
    if repository.is_empty() {
        return Err("镜像名称不能为空".to_string());
    }

    let mut normalized_repo = strip_registry_prefix(repository, registry).to_string();
    if let Some((manifest_repo, manifest_tag)) = parse_manifest_style_repo(&normalized_repo) {
        normalized_repo = manifest_repo;
        let tag = pick_effective_tag(request_tag, Some(manifest_tag.as_str()));
        let repo = normalize_repository(registry, normalized_repo.trim())?;
        return Ok((repo, tag));
    }

    let (repo_without_tag, parsed_tag) = split_repo_and_tag(&normalized_repo);
    let tag = pick_effective_tag(request_tag, parsed_tag.as_deref());
    let repo = normalize_repository(registry, repo_without_tag.trim())?;
    Ok((repo, tag))
}

fn pick_effective_tag(request_tag: Option<&str>, parsed_tag: Option<&str>) -> String {
    let explicit = request_tag.map(str::trim).filter(|v| !v.is_empty());
    let use_parsed_tag = parsed_tag.is_some() && explicit.map(|v| v == "latest").unwrap_or(true);
    if use_parsed_tag {
        return parsed_tag.unwrap_or("latest").to_string();
    }
    explicit.unwrap_or("latest").to_string()
}

fn strip_registry_prefix<'a>(repo_input: &'a str, registry: &str) -> &'a str {
    let mut value = repo_input.trim();
    if let Some(without_scheme) = value.strip_prefix("https://") {
        value = without_scheme;
    } else if let Some(without_scheme) = value.strip_prefix("http://") {
        value = without_scheme;
    }
    if let Some(stripped) = value.strip_prefix(&format!("{registry}/")) {
        return stripped;
    }
    value
}

fn parse_manifest_style_repo(repository: &str) -> Option<(String, String)> {
    let marker = "/manifest/";
    let (repo, tag) = repository.split_once(marker)?;
    let tag = tag.trim();
    if repo.trim().is_empty() || tag.is_empty() {
        return None;
    }
    Some((repo.to_string(), tag.to_string()))
}

fn split_repo_and_tag(repository: &str) -> (String, Option<String>) {
    if repository.contains("@sha256:") {
        return (repository.to_string(), None);
    }
    if let Some((repo, candidate_tag)) = repository.rsplit_once(':') {
        if !candidate_tag.contains('/') && !repo.trim().is_empty() && !candidate_tag.trim().is_empty()
        {
            return (repo.to_string(), Some(candidate_tag.to_string()));
        }
    }
    (repository.to_string(), None)
}

fn default_persisted_config() -> PersistedConfig {
    PersistedConfig {
        output_dir: String::new(),
        default_tag: "latest".to_string(),
        platform_os: "linux".to_string(),
        platform_architecture: "amd64".to_string(),
        http_proxy: String::new(),
        https_proxy: String::new(),
        no_proxy: String::new(),
        username: String::new(),
        password: String::new(),
    }
}

fn open_config_db(app: &tauri::AppHandle) -> Result<Connection, String> {
    let mut app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("获取应用数据目录失败: {e}"))?;
    app_data_dir.push("config");
    fs::create_dir_all(&app_data_dir).map_err(|e| format!("创建配置目录失败: {e}"))?;
    let db_path = app_data_dir.join("dockerhub-puller.db");

    let conn = Connection::open(&db_path).map_err(format_sqlite_err)?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS app_config (
            key TEXT PRIMARY KEY NOT NULL,
            value TEXT NOT NULL
        )",
        [],
    )
    .map_err(format_sqlite_err)?;
    Ok(conn)
}

fn load_config_value(conn: &Connection, key: &str) -> Result<Option<String>, String> {
    conn.query_row(
        "SELECT value FROM app_config WHERE key = ?1",
        params![key],
        |row| row.get(0),
    )
    .optional()
    .map_err(format_sqlite_err)
}

fn upsert_config_value(
    tx: &rusqlite::Transaction<'_>,
    key: &str,
    value: &str,
) -> Result<(), String> {
    tx.execute(
        "INSERT INTO app_config(key, value)
         VALUES(?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )
    .map_err(format_sqlite_err)?;
    Ok(())
}

fn normalize_registry(registry: Option<&str>) -> String {
    registry
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .unwrap_or("docker.io")
        .to_ascii_lowercase()
}

fn to_api_registry(registry: &str) -> String {
    match registry {
        "docker.io" | "index.docker.io" => "registry-1.docker.io".to_string(),
        other => other.to_string(),
    }
}

fn normalize_repository(registry: &str, repository: &str) -> Result<String, String> {
    if repository.is_empty() {
        return Err("镜像名称不能为空".to_string());
    }
    if registry == "docker.io" && !repository.contains('/') {
        return Ok(format!("library/{}", repository));
    }
    Ok(repository.to_string())
}

fn digest_to_key(digest: &str) -> Result<String, String> {
    digest
        .split_once(':')
        .map(|(_, v)| v.to_string())
        .ok_or_else(|| format!("无效 digest 格式: {digest}"))
}

fn digest_to_blob_tar_path(digest: &str) -> Result<String, String> {
    let (algorithm, encoded) = digest
        .split_once(':')
        .ok_or_else(|| format!("无效 digest 格式: {digest}"))?;
    Ok(format!("blobs/{algorithm}/{encoded}"))
}

fn build_sha256_digest(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let hash = hasher.finalize();
    format!("sha256:{hash:x}")
}

fn build_sha256_digest_from_file(path: &Path) -> Result<String, String> {
    let mut file = File::open(path).map_err(|e| format!("读取镜像层文件失败: {e}"))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|e| format!("读取镜像层文件失败: {e}"))?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    let hash = hasher.finalize();
    Ok(format!("sha256:{hash:x}"))
}

fn resolve_layer_diff_ids(config_json: &Value, layer_count: usize) -> Result<Vec<String>, String> {
    let rootfs = config_json
        .get("rootfs")
        .ok_or_else(|| "镜像 config 缺少 rootfs 字段".to_string())?;
    let diff_ids = rootfs
        .get("diff_ids")
        .and_then(Value::as_array)
        .ok_or_else(|| "镜像 config 缺少 rootfs.diff_ids 字段".to_string())?;
    let mut values = diff_ids
        .iter()
        .filter_map(Value::as_str)
        .map(str::to_string)
        .collect::<Vec<String>>();
    if values.len() < layer_count {
        return Err(format!(
            "镜像层数量不匹配：manifest 层数 {}，config diff_ids 数量 {}",
            layer_count,
            values.len()
        ));
    }
    if values.len() > layer_count {
        values = values.split_off(values.len() - layer_count);
    }
    Ok(values)
}

fn build_compat_layer_jsons(
    selected_manifest: &ImageManifest,
    layer_diff_ids: &[String],
    config_json: &Value,
) -> Vec<Vec<u8>> {
    if selected_manifest.layers.is_empty() || layer_diff_ids.is_empty() {
        return Vec::new();
    }

    let os = config_json
        .get("os")
        .and_then(Value::as_str)
        .unwrap_or("linux")
        .to_string();
    let architecture = config_json
        .get("architecture")
        .and_then(Value::as_str)
        .unwrap_or("amd64")
        .to_string();
    let created = config_json
        .get("created")
        .and_then(Value::as_str)
        .unwrap_or("1970-01-01T08:00:00+08:00")
        .to_string();

    let mut entries = Vec::with_capacity(layer_diff_ids.len());
    let mut parent_id: Option<String> = None;
    for (index, diff_digest) in layer_diff_ids.iter().enumerate() {
        let id = digest_to_key(diff_digest).unwrap_or_else(|_| sanitize_file_name(diff_digest));
        let mut obj = serde_json::Map::new();
        obj.insert("id".to_string(), Value::String(id.clone()));
        if let Some(parent) = parent_id.clone() {
            obj.insert("parent".to_string(), Value::String(parent));
        }
        obj.insert("created".to_string(), Value::String(created.clone()));
        obj.insert("container_config".to_string(), build_empty_container_config());
        obj.insert("os".to_string(), Value::String(os.clone()));

        if index + 1 == layer_diff_ids.len() {
            obj.insert("architecture".to_string(), Value::String(architecture.clone()));
            if let Some(config_value) = config_json.get("config") {
                obj.insert("config".to_string(), config_value.clone());
            }
            if let Some(container_config_value) = config_json.get("container_config") {
                obj.insert("container_config".to_string(), container_config_value.clone());
            }
        }

        if let Ok(bytes) = serde_json::to_vec(&Value::Object(obj)) {
            entries.push(bytes);
        }
        parent_id = Some(id);
    }
    entries
}

fn build_empty_container_config() -> Value {
    json!({
        "Hostname": "",
        "Domainname": "",
        "User": "",
        "AttachStdin": false,
        "AttachStdout": false,
        "AttachStderr": false,
        "Tty": false,
        "OpenStdin": false,
        "StdinOnce": false,
        "Env": Value::Null,
        "Cmd": Value::Null,
        "Image": "",
        "Volumes": Value::Null,
        "WorkingDir": "",
        "Entrypoint": Value::Null,
        "OnBuild": Value::Null,
        "Labels": Value::Null
    })
}

fn sanitize_file_name(input: &str) -> String {
    input.replace('/', "_").replace(':', "_")
}

fn should_bypass_proxy(target_host: &str, no_proxy: Option<&str>) -> bool {
    let Some(no_proxy) = clean_opt(no_proxy) else {
        return false;
    };
    no_proxy.split(',').any(|rule| {
        let item = rule.trim();
        if item.is_empty() {
            return false;
        }
        if item == "*" {
            return true;
        }
        if target_host == item {
            return true;
        }
        if let Some(stripped) = item.strip_prefix('.') {
            return target_host.ends_with(stripped);
        }
        target_host.ends_with(&format!(".{item}"))
    })
}

fn clean_opt(value: Option<&str>) -> Option<&str> {
    value.map(str::trim).filter(|v| !v.is_empty())
}

fn value_to_u64(value: Option<&Value>) -> u64 {
    let Some(value) = value else {
        return 0;
    };
    if let Some(number) = value.as_u64() {
        return number;
    }
    value
        .as_str()
        .and_then(|text| text.parse::<u64>().ok())
        .unwrap_or(0)
}

fn preview_text(text: &str, max_chars: usize) -> String {
    let mut preview = text
        .chars()
        .filter(|ch| !ch.is_control() || *ch == '\n' || *ch == '\r' || *ch == '\t')
        .take(max_chars)
        .collect::<String>();
    if text.chars().count() > max_chars {
        preview.push_str("...");
    }
    preview
}

fn format_reqwest_err(err: reqwest::Error) -> String {
    format!("网络请求失败: {err}")
}

fn format_sqlite_err(err: rusqlite::Error) -> String {
    format!("SQLite 操作失败: {err}")
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            search_images,
            pull_image_as_tar,
            test_registry_auth,
            load_persisted_config,
            save_persisted_config
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
