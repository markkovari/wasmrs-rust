use std::collections::VecDeque;

use futures::StreamExt;
use wasmrs::{Metadata, RSocket, RawPayload};
use wasmrs_codec::messagepack::*;
use wasmrs_host::WasiParams;
use wasmrs_wasmtime::WasmtimeBuilder;

static MODULE_BYTES: &[u8] = include_bytes!("../../../build/reqres_component.wasm");

#[test_log::test(tokio::test)]
async fn test_req_stream() -> anyhow::Result<()> {
  let engine = WasmtimeBuilder::new()
    .with_module_bytes("reqres_component", MODULE_BYTES)
    .wasi_params(WasiParams::default())
    .build()?;
  let host = wasmrs_host::Host::new(engine)?;
  let context = host.new_context(64 * 1024, 64 * 1024)?;
  let op = context.get_export("suite.test", "chars")?;

  let mbytes = Metadata::new(op).encode();

  #[derive(serde::Serialize)]
  struct Input {
    input: String,
  }
  let input = Input {
    input: "HELLO WORLD".to_owned(),
  };

  let bytes = serialize(&input).unwrap();

  let payload = RawPayload::new(mbytes, bytes.into());

  let mut response = context.request_stream(payload.clone());
  let mut outputs: VecDeque<String> = input.input.chars().map(|i| i.to_string()).collect();
  while let Some(response) = response.next().await {
    match response {
      Ok(v) => {
        let bytes = v.data.unwrap();
        let val: String = deserialize(&bytes).unwrap();
        println!("{}", val);
        let next = outputs.pop_front().unwrap();
        assert_eq!(val, next);
      }
      Err(e) => {
        panic!("Error: {:?}", e);
      }
    }
  }
  assert!(outputs.is_empty());

  Ok(())
}
