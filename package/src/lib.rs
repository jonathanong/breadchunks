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

// Defers UTF-8 validation to the worker thread for the async path.
enum TaskInput {
    Bytes(Vec<u8>),
    String(String),
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
                    length: {
                        assert!(
                            c.length <= u32::MAX as usize,
                            "chunk length exceeds u32::MAX; docs >4 GiB unsupported on Node binding"
                        );
                        c.length as u32 // usize→u32 narrowing for napi; docs >4 GiB unsupported on Node binding
                    },
                })
                .collect()
        })
        .collect()
}

pub struct ChunkTask {
    inputs: Vec<TaskInput>,
    options: Option<breadchunks::ChunkOptions>,
}

impl Task for ChunkTask {
    type Output = Vec<Vec<Chunk>>;
    type JsValue = Vec<Vec<Chunk>>;

    fn compute(&mut self) -> Result<Self::Output> {
        let inputs = std::mem::take(&mut self.inputs);
        let decoded: Result<Vec<String>> = inputs
            .into_iter()
            .map(|i| match i {
                TaskInput::Bytes(b) => String::from_utf8(b)
                    .map_err(|e| Error::from_reason(format!("Buffer is not valid UTF-8: {e}"))),
                TaskInput::String(s) => Ok(s),
            })
            .collect();
        Ok(run_batch(&decoded?, &self.options))
    }

    fn resolve(&mut self, _env: Env, output: Self::Output) -> Result<Self::JsValue> {
        Ok(output)
    }
}

#[napi(ts_return_type = "Promise<Array<Array<Chunk>>>")]
pub fn chunk(
    inputs: Vec<Either<Buffer, String>>,
    options: Option<ChunkOptions>,
) -> AsyncTask<ChunkTask> {
    let task_inputs = inputs
        .into_iter()
        .map(|i| match i {
            Either::A(buf) => TaskInput::Bytes(buf.to_vec()),
            Either::B(s) => TaskInput::String(s),
        })
        .collect();
    AsyncTask::new(ChunkTask {
        inputs: task_inputs,
        options: map_options(options),
    })
}

#[napi(js_name = "chunkSync")]
pub fn chunk_sync(
    inputs: Vec<Either<Buffer, String>>,
    options: Option<ChunkOptions>,
) -> Result<Vec<Vec<Chunk>>> {
    let decoded: Result<Vec<String>> = inputs
        .into_iter()
        .map(|i| match i {
            Either::A(buf) => std::str::from_utf8(&buf)
                .map(|s| s.to_owned())
                .map_err(|e| Error::from_reason(format!("Buffer is not valid UTF-8: {e}"))),
            Either::B(s) => Ok(s),
        })
        .collect();
    Ok(run_batch(&decoded?, &map_options(options)))
}
