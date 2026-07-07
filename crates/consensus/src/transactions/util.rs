use alloy_primitives::{Bytes, FixedBytes};
use bytes::Buf;

/// Decodes `N` raw bytes from the current buffer and converts them into `F`.
pub fn decode<const N: usize, F: From<FixedBytes<N>>>(buf: &mut &[u8]) -> alloy_rlp::Result<F> {
    if buf.len() < N {
        return Err(alloy_rlp::Error::InputTooShort);
    }
    let data: FixedBytes<N> = FixedBytes::from(
        &buf[..N]
            .try_into()
            .map_err(|_| alloy_rlp::Error::InputTooShort)?,
    );
    buf.advance(N);
    Ok(F::from(data))
}

/// Consumes and returns the remaining buffer contents as [`Bytes`].
pub fn decode_rest(buf: &mut &[u8]) -> Bytes {
    // read the rest of the buffer as Bytes
    let data = Bytes::from(buf.to_vec());
    *buf = &[];
    data
}
