use crate::utils::get_node;
use crate::hyperbeam::Hyperbeam;
use crate::wallet::SignerTypes;
use axum::{extract::Path, response::Json};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use axum::response::IntoResponse;
use axum::{http::StatusCode};
use pulldown_cmark::{Parser, Options, html};


fn init_hb_client(node_endpoint: String) -> Hyperbeam {
    Hyperbeam::new(node_endpoint, SignerTypes::Arweave("test_key.json".to_string())).unwrap()
}

pub async fn status_handler() -> Json<Value> {
    Json(json!({"status": "running"}))
}

pub async fn node_info_handler(Path(id): Path<String>) -> Json<Value> {
    let node_obj = get_node(&id);
    let hb = init_hb_client(node_obj.node_url);
    let res = hb.meta_info().await.unwrap();

    Json(serde_json::to_value(res).unwrap())
}

pub async fn node_routes_handler(Path(id): Path<String>) -> Json<Value> {
    let node_obj = get_node(&id);
    let hb = init_hb_client(node_obj.node_url);
    let res = hb.router_routes().await.unwrap();
    
    Json(serde_json::to_value(res).unwrap())
}

pub async fn node_metrics_handler(
    Path(id): Path<String>,
) -> impl IntoResponse {
    let node_obj = get_node(&id);
    let hb = init_hb_client(node_obj.node_url);
    let markdown_content = hb.hyperbuddy_metrics().await.unwrap();

    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);

    let parser = Parser::new_ext(&markdown_content, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    let styled_html = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Node Metrics</title>
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
            line-height: 1.6;
            color: #333;
            max-width: 900px;
            margin: 0 auto;
            padding: 20px;
        }}
        pre {{
            background-color: #f6f8fa;
            border-radius: 3px;
            padding: 16px;
            overflow: auto;
        }}
        code {{
            font-family: SFMono-Regular, Consolas, "Liberation Mono", Menlo, monospace;
            font-size: 85%;
        }}
        table {{
            border-collapse: collapse;
            width: 100%;
            margin-bottom: 20px;
        }}
        th, td {{
            border: 1px solid #ddd;
            padding: 8px;
            text-align: left;
        }}
        th {{
            background-color: #f2f2f2;
        }}
        h1, h2, h3, h4, h5, h6 {{
            margin-top: 24px;
            margin-bottom: 16px;
            font-weight: 600;
            line-height: 1.25;
        }}
        h1 {{
            font-size: 2em;
            border-bottom: 1px solid #eaecef;
            padding-bottom: .3em;
        }}
        h2 {{
            font-size: 1.5em;
            border-bottom: 1px solid #eaecef;
            padding-bottom: .3em;
        }}
    </style>
</head>
<body>
    {html_output}
</body>
</html>"#
    );

    axum::response::Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "text/html; charset=UTF-8")
        .header("accept-ranges", "bytes")
        .header("cache-control", "public, max-age=31536000")
        .body(styled_html)
        .unwrap()
        .into_response()
}
