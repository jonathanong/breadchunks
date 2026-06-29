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
#[derive(Debug)]
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

fn map_chunk(c: breadchunks::Chunk) -> std::result::Result<Chunk, &'static str> {
    let length = u32::try_from(c.length)
        .map_err(|_| "chunk length exceeds u32::MAX; docs >4 GiB unsupported on Node binding")?;
    Ok(Chunk {
        level: c.level,
        header: c.header.map(std::sync::Arc::unwrap_or_clone),
        headers: std::sync::Arc::unwrap_or_clone(c.headers),
        breadcrumb: std::sync::Arc::unwrap_or_clone(c.breadcrumb),
        text: c.text,
        length,
    })
}

fn run_batch(
    inputs: &[String],
    options: &Option<breadchunks::ChunkOptions>,
) -> std::result::Result<Vec<Vec<Chunk>>, &'static str> {
    inputs
        .iter()
        .map(|text| {
            breadchunks::chunk(text, options.as_ref())
                .into_iter()
                .map(map_chunk)
                .collect::<std::result::Result<Vec<Chunk>, &'static str>>()
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
        run_batch(&decoded?, &self.options).map_err(Error::from_reason)
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

    #[test]
    fn test_map_chunk_valid_length() {
        let mut chunks = breadchunks::chunk("# Test\n\nHello, world!", None);
        let chunk = chunks.remove(0);

        let result = map_chunk(chunk).unwrap();
        assert_eq!(result.level, 1);
        assert_eq!(result.header, Some("Test".to_string()));
        assert_eq!(result.breadcrumb, "Test");
        assert_eq!(result.text, "Hello, world!");
        assert_eq!(result.length, 18);
    }

    #[test]
    #[cfg(target_pointer_width = "64")]
    fn test_map_chunk_exceeds_u32_max() {
        let mut chunks = breadchunks::chunk("Too long", None);
        let mut chunk = chunks.remove(0);
        chunk.length = (u32::MAX as usize) + 1;
        let err = map_chunk(chunk).unwrap_err();
        assert_eq!(
            err,
            "chunk length exceeds u32::MAX; docs >4 GiB unsupported on Node binding"
        );
    }

    #[test]
    fn test_run_batch_empty() {
        let inputs: Vec<String> = vec![];
        let result = run_batch(&inputs, &None).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_run_batch_single_input() {
        let inputs = vec!["# Title\n\nContent".to_string()];
        let result = run_batch(&inputs, &None).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].len(), 1);
        assert_eq!(result[0][0].text, "Content");
        assert_eq!(result[0][0].breadcrumb, "Title");
    }

    #[test]
    fn test_run_batch_multiple_inputs() {
        let inputs = vec![
            "# Title 1\n\nContent 1".to_string(),
            "# Title 2\n\nContent 2".to_string(),
        ];
        let result = run_batch(&inputs, &None).unwrap();
        assert_eq!(result.len(), 2);

        assert_eq!(result[0].len(), 1);
        assert_eq!(result[0][0].text, "Content 1");
        assert_eq!(result[0][0].breadcrumb, "Title 1");

        assert_eq!(result[1].len(), 1);
        assert_eq!(result[1][0].text, "Content 2");
        assert_eq!(result[1][0].breadcrumb, "Title 2");
    }

    #[test]
    fn test_run_batch_with_options() {
        let inputs = vec!["Just content without title".to_string()];
        let options = Some(breadchunks::ChunkOptions {
            title: Some("Fallback Title".to_string()),
            ..Default::default()
        });

        let result = run_batch(&inputs, &options).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].len(), 1);
        assert_eq!(result[0][0].text, "Just content without title");
        assert_eq!(result[0][0].breadcrumb, "Fallback Title");
    }
}
