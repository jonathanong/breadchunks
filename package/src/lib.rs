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

fn run_batch(
    inputs: &[String],
    options: &Option<breadchunks::ChunkOptions>,
) -> Result<Vec<Vec<Chunk>>> {
    inputs
        .iter()
        .map(|text| {
            breadchunks::chunk(text, options.clone())
                .into_iter()
                .map(|c| {
                    if c.length > u32::MAX as usize {
                        return Err(Error::from_reason(
                            "chunk length exceeds u32::MAX; docs >4 GiB unsupported on Node binding",
                        ));
                    }
                    Ok(Chunk {
                        level: c.level,
                        header: c.header,
                        headers: c.headers,
                        breadcrumb: c.breadcrumb,
                        text: c.text,
                        length: c.length as u32,
                    })
                })
                .collect::<Result<Vec<Chunk>>>()
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
        run_batch(&decoded?, &self.options)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_options_none() {
        assert!(map_options(None).is_none());
    }

    #[test]
    fn test_map_options_some() {
        let input = ChunkOptions {
            min_length: Some(10),
            max_length: Some(100),
            phase: Some(2),
            title: Some("Test Title".to_string()),
        };

        let result = map_options(Some(input)).expect("Expected Some");
        assert_eq!(result.min_length, Some(10));
        assert_eq!(result.max_length, Some(100));
        assert_eq!(result.phase, Some(2));
        assert_eq!(result.title.as_deref(), Some("Test Title"));
    }
}
