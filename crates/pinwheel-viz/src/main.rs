//! 状態グラフ Generator（PoC）。
//!
//! A をカンマ区切りで受け取り、`pinwheel-core` のラベル付き状態グラフ `G_A`
//! （`LabeledGraph::explore`）を展開して SVG で表示する小さな web サーバ。
//! 描画レイアウトは純 Rust の `layout-rs`（Sugiyama 階層レイアウト）で行い、
//! JS / WASM を使わず Rust だけで完結する。
//!
//! - `GET /?a=2,3,5` : 入力フォーム＋インライン SVG の HTML
//! - `GET /svg?a=2,3,5` : SVG 単体（image/svg+xml）
//! - `GET /dot?a=2,3,5` : Graphviz DOT（text/plain、外部ビューア用）

use std::collections::HashMap;

use axum::{
    Router,
    extract::Query,
    http::{StatusCode, header},
    response::{Html, IntoResponse},
    routing::get,
};

use layout::backends::svg::SVGWriter;
use layout::core::base::Orientation;
use layout::core::geometry::Point;
use layout::core::style::StyleAttr;
use layout::std_shapes::shapes::{Arrow, Element, ShapeKind};
use layout::topo::layout::VisualGraph;

use pinwheel_core::instance::PinwheelInstance;
use pinwheel_core::labeled::graph::LabeledGraph;

const ADDR: &str = "127.0.0.1:3000";
const DEFAULT_A: &str = "2,3";

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(index))
        .route("/svg", get(svg))
        .route("/dot", get(dot));

    let listener = tokio::net::TcpListener::bind(ADDR).await.unwrap();
    println!("状態グラフ Generator: http://{ADDR}/");
    axum::serve(listener, app).await.unwrap();
}

// --- ハンドラ ---

async fn index(Query(q): Query<HashMap<String, String>>) -> Html<String> {
    let a = query_a(&q);
    let body = match make_graph(&a) {
        Ok((instance, graph)) => {
            let info = info_line(&instance, &graph);
            let svg = render_svg(&graph);
            format!("<p class=\"info\">{info}</p>\n<div class=\"graph\">{svg}</div>")
        }
        Err(e) => format!("<p class=\"err\">エラー: {}</p>", escape(&e)),
    };
    Html(page(&a, &body))
}

async fn svg(Query(q): Query<HashMap<String, String>>) -> impl IntoResponse {
    let a = query_a(&q);
    match make_graph(&a) {
        Ok((_, graph)) => (
            [(header::CONTENT_TYPE, "image/svg+xml; charset=utf-8")],
            render_svg(&graph),
        )
            .into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, e).into_response(),
    }
}

async fn dot(Query(q): Query<HashMap<String, String>>) -> impl IntoResponse {
    let a = query_a(&q);
    match make_graph(&a) {
        Ok((_, graph)) => (
            [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
            render_dot(&graph),
        )
            .into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, e).into_response(),
    }
}

// --- グラフ生成 ---

fn make_graph(a: &str) -> Result<(PinwheelInstance, LabeledGraph), String> {
    let periods = parse_periods(a)?;
    let instance = PinwheelInstance::new(periods);
    let graph = LabeledGraph::explore(&instance);
    Ok((instance, graph))
}

fn parse_periods(a: &str) -> Result<Vec<u32>, String> {
    let mut periods = Vec::new();
    for tok in a.split(',') {
        let t = tok.trim();
        if t.is_empty() {
            continue;
        }
        let n: u32 = t
            .parse()
            .map_err(|_| format!("'{t}' は整数ではありません"))?;
        if n < 1 {
            return Err(format!("周期は 1 以上にしてください: {n}"));
        }
        periods.push(n);
    }
    if periods.is_empty() {
        return Err("周期を 1 つ以上入力してください（例: 2,3,5）".into());
    }
    Ok(periods)
}

fn info_line(instance: &PinwheelInstance, graph: &LabeledGraph) -> String {
    let a = instance
        .periods()
        .iter()
        .map(u32::to_string)
        .collect::<Vec<_>>()
        .join(",");
    let edges: usize = (0..graph.len())
        .map(|v| graph.successors_of(v).len())
        .sum();
    format!("A = ({a})  /  頂点 {}  辺 {}", graph.len(), edges)
}

// --- 描画 ---

fn render_svg(graph: &LabeledGraph) -> String {
    let mut vg = VisualGraph::new(Orientation::TopToBottom);

    let mut handles = Vec::with_capacity(graph.len());
    for state in graph.states() {
        let shape = ShapeKind::new_box(&state_label(state));
        let element = Element::create(
            shape,
            StyleAttr::simple(),
            Orientation::LeftToRight,
            Point::new(56.0, 38.0),
        );
        handles.push(vg.add_node(element));
    }

    for v in 0..graph.len() {
        for &(task, dst) in graph.successors_of(v) {
            let arrow = Arrow::simple(&task_letter(task));
            vg.add_edge(arrow, handles[v], handles[dst]);
        }
    }

    let mut writer = SVGWriter::new();
    vg.do_it(false, false, false, &mut writer);
    writer.finalize()
}

fn render_dot(graph: &LabeledGraph) -> String {
    let mut s = String::from("digraph G_A {\n  rankdir=TB;\n  node [shape=box, fontname=\"monospace\"];\n");
    for (i, state) in graph.states().iter().enumerate() {
        s.push_str(&format!("  n{i} [label=\"{}\"];\n", state_label(state)));
    }
    for v in 0..graph.len() {
        for &(task, dst) in graph.successors_of(v) {
            s.push_str(&format!("  n{v} -> n{dst} [label=\"{}\"];\n", task_letter(task)));
        }
    }
    s.push_str("}\n");
    s
}

/// 頂点ラベル: 経過日数ベクトル x を "0,1,2" の形に。
fn state_label(state: &[u32]) -> String {
    state.iter().map(u32::to_string).collect::<Vec<_>>().join(",")
}

/// タスク添字をラベルへ: 0->A, 1->B, ..., 25->Z, 以降は T26 など。
fn task_letter(task: usize) -> String {
    if task < 26 {
        ((b'A' + task as u8) as char).to_string()
    } else {
        format!("T{task}")
    }
}

// --- HTML ---

fn query_a(q: &HashMap<String, String>) -> String {
    let a = q.get("a").map(String::as_str).unwrap_or(DEFAULT_A).trim();
    if a.is_empty() {
        DEFAULT_A.to_string()
    } else {
        a.to_string()
    }
}

fn page(a: &str, body: &str) -> String {
    let a_attr = escape(a);
    let a_query = a.replace(' ', "");
    format!(
        r#"<!doctype html>
<html lang="ja">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>状態グラフ Generator</title>
<style>
  body {{ font-family: sans-serif; margin: 1.5rem; color: #222; }}
  h1 {{ font-size: 1.3rem; }}
  form {{ margin: 0 0 .6rem; }}
  input[type=text] {{ font-size: 1rem; padding: .3rem .5rem; width: 16rem; }}
  button {{ font-size: 1rem; padding: .3rem .9rem; cursor: pointer; }}
  .info {{ color: #444; font-family: monospace; margin: .3rem 0; }}
  .err {{ color: #b00020; }}
  .links {{ font-size: .9rem; margin: .2rem 0 1rem; }}
  .links a {{ margin-right: 1rem; }}
  .graph {{ border: 1px solid #ddd; overflow: auto; padding: .5rem; }}
  .graph svg {{ max-width: 100%; height: auto; }}
</style>
</head>
<body>
<h1>状態グラフ Generator</h1>
<form method="get" action="/">
  <label>A（カンマ区切り、例 2,3,5）:
    <input type="text" name="a" value="{a_attr}" autofocus></label>
  <button type="submit">生成</button>
</form>
<p class="links"><a href="/svg?a={a_query}">SVG を開く</a><a href="/dot?a={a_query}">DOT を開く</a></p>
{body}
</body>
</html>"#
    )
}

/// HTML 属性 / 本文への最小エスケープ。
fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
