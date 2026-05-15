#![deny(clippy::all)]

use napi::bindgen_prelude::{AsyncTask, Buffer, Either};
use napi::{Env, Error, Result, Task};
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

fn decode_input(input: Either<Buffer, String>) -> Result<String> {
    match input {
        Either::A(buf) => std::str::from_utf8(&buf)
            .map(|s| s.to_owned())
            .map_err(|e| Error::from_reason(format!("Buffer is not valid UTF-8: {e}"))),
        Either::B(s) => Ok(s),
    }
}

fn map_options(options: Option<ChunkOptions>) -> Option<breadchunks::ChunkOptions> {
    options.map(|o| breadchunks::ChunkOptions {
        min_length: o.min_length,
        max_length: o.max_length,
        phase: o.phase,
        title: o.title,
    })
}

fn run_batch(inputs: &[String], options: &Option<breadchunks::ChunkOptions>) -> Vec<Vec<Chunk>> {
    inputs
        .iter()
        .map(|text| {
            breadchunks::chunk(text, options.clone())
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
        })
        .collect()
}

pub struct ChunkTask {
    inputs: Vec<String>,
    options: Option<breadchunks::ChunkOptions>,
}

impl Task for ChunkTask {
    type Output = Vec<Vec<Chunk>>;
    type JsValue = Vec<Vec<Chunk>>;

    fn compute(&mut self) -> Result<Self::Output> {
        Ok(run_batch(&self.inputs, &self.options))
    }

    fn resolve(&mut self, _env: Env, output: Self::Output) -> Result<Self::JsValue> {
        Ok(output)
    }
}

#[napi(ts_return_type = "Promise<Array<Array<Chunk>>>")]
pub fn chunk(
    inputs: Vec<Either<Buffer, String>>,
    options: Option<ChunkOptions>,
) -> Result<AsyncTask<ChunkTask>> {
    let inputs: Result<Vec<String>> = inputs.into_iter().map(decode_input).collect();
    Ok(AsyncTask::new(ChunkTask {
        inputs: inputs?,
        options: map_options(options),
    }))
}

#[napi(js_name = "chunkSync")]
pub fn chunk_sync(
    inputs: Vec<Either<Buffer, String>>,
    options: Option<ChunkOptions>,
) -> Result<Vec<Vec<Chunk>>> {
    let inputs: Result<Vec<String>> = inputs.into_iter().map(decode_input).collect();
    Ok(run_batch(&inputs?, &map_options(options)))
}
