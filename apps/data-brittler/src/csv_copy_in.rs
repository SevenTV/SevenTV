// use std::pin::Pin;

// use bytes::Bytes;
// use futures::SinkExt;
// use postgres_from_row::tokio_postgres::CopyInSink;
// use tokio_util::{compat::TokioAsyncWriteCompatExt, io::{CopyToBytes,
// SinkWriter}};

// pub type CsvCopyIn = csv_async::AsyncSerializer<Pin<Box<dyn
// futures::AsyncWrite + Send + Sync>>>;

// pub fn new(copy_in: CopyInSink<Bytes>) -> CsvCopyIn {
//     tracing::trace!("creating csv copy in");

//     let sink = copy_in.sink_map_err(|e|
// std::io::Error::new(std::io::ErrorKind::Other, e));

//     let sink = CopyToBytes::new(sink);

//     let sink_writer = Box::pin(SinkWriter::new(sink).compat_write()) as
// Pin<Box<dyn futures::AsyncWrite + Send + Sync>>;
//     csv_async::AsyncWriterBuilder::new().
// quote_style(csv_async::QuoteStyle::Necessary).create_serializer(sink_writer)
// }
