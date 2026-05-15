#![deny(clippy::all)]

use napi_derive::napi;

#[napi(object)]
pub struct ChunkOptions {
    pub min_length: Option<u32>,
    pub max_length: Option<u32>,
    pub phase: Option<u32>,
    pub title: Option<String>,
}

#[napi(object)]
pub struct Chunk {
    pub level: u32,
    pub header: Option<String>,
    pub headers: Vec<Option<String>>,
    pub breadcrumb: String,
    pub text: String,
    pub length: u32,
}

#[napi]
pub fn chunk(text: String, options: Option<ChunkOptions>) -> Vec<Chunk> {
    let opts = options.map(|o| breadchunks::ChunkOptions {
        min_length: o.min_length,
        max_length: o.max_length,
        phase: o.phase,
        title: o.title,
    });
    breadchunks::chunk(&text, opts)
        .into_iter()
        .map(|c| Chunk {
            level: c.level,
            header: c.header,
            headers: c.headers,
            breadcrumb: c.breadcrumb,
            text: c.text,
            length: c.length,
        })
        .collect()
}
