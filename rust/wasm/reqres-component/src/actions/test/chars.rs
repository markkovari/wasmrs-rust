use crate::actions::test_service::chars::*;

pub(crate) async fn task(input: Input) -> Result<Output, crate::Error> {
  let stream = Flux::new();
  for c in input.input.chars() {
    stream.send(c.to_string()).unwrap();
  }
  stream.complete();
  Ok(stream.take_rx()?)
}
