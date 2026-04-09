use std::io::Write;

use flate2::write::GzEncoder;

use super::WriteTo;
use crate::{Compression, PmtError, PmtResult};

/// Trait for compression implementations.
/// Implement this to provide custom compression behavior.
pub trait Compressor {
    /// Returns the compression type for the `PMTiles` header.
    fn compression(&self) -> Compression;

    /// Compress `data` through an encoder wrapping `writer`, then finalize.
    ///
    /// # Errors
    ///
    /// Returns an error if writing to `writer` fails or the compression fails.
    fn compress<D: WriteTo + ?Sized, W: Write>(
        &self,
        data: &D,
        writer: &mut W,
    ) -> PmtResult<()>;
}

/// Passthrough (no compression).
pub struct NoCompression;

impl Compressor for NoCompression {
    fn compression(&self) -> Compression {
        Compression::None
    }

    fn compress<D: WriteTo + ?Sized, W: Write>(
        &self,
        data: &D,
        writer: &mut W,
    ) -> PmtResult<()> {
        data.write_to(writer)?;
        Ok(())
    }
}

/// Gzip compression. Wraps [`flate2::Compression`] for level configuration.
#[derive(Default)]
pub struct GzipCompressor(pub flate2::Compression);

impl Compressor for GzipCompressor {
    fn compression(&self) -> Compression {
        Compression::Gzip
    }

    fn compress<D: WriteTo + ?Sized, W: Write>(
        &self,
        data: &D,
        writer: &mut W,
    ) -> PmtResult<()> {
        let mut encoder = GzEncoder::new(writer, self.0);
        data.write_to(&mut encoder)?;
        encoder.finish()?;
        Ok(())
    }
}

/// Brotli compression. Wraps [`brotli::enc::BrotliEncoderParams`].
#[cfg(feature = "brotli")]
#[derive(Default)]
pub struct BrotliCompressor(pub brotli::enc::BrotliEncoderParams);

#[cfg(feature = "brotli")]
impl Compressor for BrotliCompressor {
    fn compression(&self) -> Compression {
        Compression::Brotli
    }

    fn compress<D: WriteTo + ?Sized, W: Write>(
        &self,
        data: &D,
        writer: &mut W,
    ) -> PmtResult<()> {
        let mut encoder = brotli::CompressorWriter::with_params(writer, 4096, &self.0);
        data.write_to(&mut encoder)?;
        Ok(())
    }
}

/// Zstd compression with configurable level.
#[cfg(feature = "zstd")]
pub struct ZstdCompressor(pub i32);

#[cfg(feature = "zstd")]
impl Compressor for ZstdCompressor {
    fn compression(&self) -> Compression {
        Compression::Zstd
    }

    fn compress<D: WriteTo + ?Sized, W: Write>(
        &self,
        data: &D,
        writer: &mut W,
    ) -> PmtResult<()> {
        let mut encoder = zstd::stream::Encoder::new(writer, self.0)?;
        data.write_to(&mut encoder)?;
        encoder.finish()?;
        Ok(())
    }
}

#[cfg(feature = "zstd")]
impl Default for ZstdCompressor {
    fn default() -> Self {
        Self(zstd::DEFAULT_COMPRESSION_LEVEL)
    }
}

/// Stub compressor for codecs whose feature is disabled.
/// Returns an error when compression is attempted.
pub struct UnsupportedCompressor(pub(crate) Compression);

impl Compressor for UnsupportedCompressor {
    fn compression(&self) -> Compression {
        self.0
    }

    fn compress<D: WriteTo + ?Sized, W: Write>(
        &self,
        _data: &D,
        _writer: &mut W,
    ) -> PmtResult<()> {
        Err(PmtError::UnsupportedCompression(self.0))
    }
}
