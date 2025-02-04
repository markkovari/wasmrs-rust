use std::collections::VecDeque;

use futures::StreamExt;
use wasmrs::{Metadata, RSocket, RawPayload};
use wasmrs_codec::messagepack::*;
use wasmrs_host::WasiParams;
use wasmrs_rx::*;
use wasmrs_wasmtime::WasmtimeBuilder;

static MODULE_BYTES: &[u8] = include_bytes!("../../../build/reqres_component.wasm");

#[test_log::test(tokio::test)]
async fn test_req_channel_single() -> anyhow::Result<()> {
  let engine = WasmtimeBuilder::new()
    .with_module_bytes("reqres_component", MODULE_BYTES)
    .wasi_params(WasiParams::default())
    .build()?;
  let host = wasmrs_host::Host::new(engine)?;
  let context = host.new_context(64 * 1024, 64 * 1024)?;
  let op = context.get_export("suite.test", "reverse")?;

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

  let stream = FluxChannel::new();
  stream.send(payload.clone())?;
  stream.complete();
  let mut response = context.request_channel(Box::pin(stream));
  let mut outputs: VecDeque<String> = vec!["DLROW OLLEH".to_owned()].into();
  while let Some(response) = response.next().await {
    println!("response: {:?}", response);
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

#[test_log::test(tokio::test)]
async fn test_req_channel_multi() -> anyhow::Result<()> {
  let engine = WasmtimeBuilder::new()
    .with_module_bytes("reqres_component", MODULE_BYTES)
    .wasi_params(WasiParams::default())
    .build()?;
  let host = wasmrs_host::Host::new(engine)?;
  let context = host.new_context(64 * 1024, 64 * 1024)?;
  let op = context.get_export("suite.test", "wrap")?;

  let mbytes = Metadata::new(op).encode();

  #[derive(serde::Serialize)]
  struct Input1 {
    left: String,
    right: String,
  }
  #[derive(serde::Serialize)]
  struct Input2 {
    input: String,
  }
  let stream = FluxChannel::new();

  let input = Input1 {
    left: "<<<".to_owned(),
    right: ">>>".to_owned(),
  };
  let bytes = serialize(&input).unwrap();
  let payload = RawPayload::new(mbytes.clone(), bytes.into());
  stream.send(payload.clone())?;

  let input = Input2 {
    input: "Hello world".to_owned(),
  };
  let bytes = serialize(&input).unwrap();
  let payload = RawPayload::new(mbytes, bytes.into());
  stream.send(payload.clone())?;

  stream.complete();
  let mut response = context.request_channel(Box::pin(stream));
  let mut outputs: VecDeque<String> = vec!["<<<Hello world>>>".to_owned()].into();
  while let Some(response) = response.next().await {
    println!("response: {:?}", response);
    match response {
      Ok(v) => {
        let bytes = v.data.unwrap();
        let val: String = deserialize(&bytes).unwrap();
        println!("Got: {}", val);
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
